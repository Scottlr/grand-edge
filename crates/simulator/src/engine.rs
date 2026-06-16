use grand_edge_domain::{
    IntervalPrice, ItemId, PaperBet, PaperBetStatus, StrategySignal, UserPosition,
};
use grand_edge_storage::Storage;
use uuid::Uuid;

use crate::{
    SimulatorConfig, SimulatorError,
    fills::{entry_fill, exit_fill},
    orders::{PaperBetOutcome, SimulatedOrderRequest, SimulatedOrderSide, SimulatedOrderStatus},
    pnl::{realized_profit_gp, realized_roi, tax_on_sale},
    replay::replay_user_position,
};

pub struct SimulationEngine {
    storage: Storage,
    config: SimulatorConfig,
}

impl SimulationEngine {
    pub fn new(storage: Storage, config: SimulatorConfig) -> Self {
        Self { storage, config }
    }

    pub async fn create_run(
        &self,
        name: &str,
        strategy_config: serde_json::Value,
    ) -> Result<Uuid, SimulatorError> {
        let run_id = Uuid::new_v4();
        self.storage
            .simulations()
            .insert_simulation_run(run_id, name, strategy_config, "created")
            .await?;
        Ok(run_id)
    }

    pub async fn simulate_signal(
        &self,
        run_id: Uuid,
        signal: &StrategySignal,
    ) -> Result<PaperBetOutcome, SimulatorError> {
        let history = self
            .storage
            .prices()
            .interval_history(
                signal.item_id,
                grand_edge_domain::PriceInterval::OneHour,
                256,
            )
            .await?;
        let request = SimulatedOrderRequest {
            run_id,
            strategy_id: signal.strategy_id.0.clone(),
            model_version: signal.model_version.0.clone(),
            item_id: signal.item_id.0,
            created_at: signal.as_of,
            side: match signal.side {
                grand_edge_domain::SignalSide::Sell | grand_edge_domain::SignalSide::Cashout => {
                    SimulatedOrderSide::Sell
                }
                _ => SimulatedOrderSide::Buy,
            },
            quantity: signal.max_quantity.map(|value| value.as_i64()).unwrap_or(1),
            limit_price: signal.target_entry.map(|value| value.as_i64()),
            target_exit: signal
                .target_exit
                .map(|value| value.as_i64())
                .or(signal.take_profit.map(|value| value.as_i64())),
            stop_loss: signal.stop_loss.map(|value| value.as_i64()),
            horizon_secs: signal.horizon_secs.as_i64(),
        };
        let outcome = self.simulate_from_history(request, &history)?;
        let paper_bet = PaperBet {
            paper_bet_id: grand_edge_domain::OrderId(outcome.bet_id),
            recommendation_id: None,
            item_id: ItemId(outcome.item_id),
            created_at: signal.as_of,
            status: match outcome.status {
                SimulatedOrderStatus::Expired => PaperBetStatus::Expired,
                SimulatedOrderStatus::Closed => PaperBetStatus::Closed,
                _ => PaperBetStatus::Filled,
            },
            execution_mode: self.config.execution_mode,
        };
        self.storage
            .simulations()
            .insert_paper_bets(&[paper_bet])
            .await?;
        Ok(outcome)
    }

    pub fn simulate_from_history(
        &self,
        request: SimulatedOrderRequest,
        history: &[IntervalPrice],
    ) -> Result<PaperBetOutcome, SimulatorError> {
        if request.quantity <= 0 {
            return Err(SimulatorError::InvalidRequest(
                "quantity must be positive".to_string(),
            ));
        }

        let entry = entry_fill(&self.config, &request, history)?;
        let exit = exit_fill(&self.config, &entry, &request, history)?;
        let tax_paid = tax_on_sale(&self.config.market_rules, request.item_id, exit.fill_price)
            * entry.filled_quantity;
        let profit = realized_profit_gp(
            &self.config.market_rules,
            request.item_id,
            entry.fill_price,
            exit.fill_price,
            entry.filled_quantity,
        );
        let hit_reason = exit
            .explanation
            .get("hit_reason")
            .and_then(|value| value.as_str())
            .map(ToString::to_string);
        let status = if hit_reason.as_deref() == Some("horizon_expiry") {
            SimulatedOrderStatus::Expired
        } else {
            SimulatedOrderStatus::Closed
        };

        Ok(PaperBetOutcome {
            bet_id: Uuid::new_v4(),
            strategy_id: request.strategy_id,
            item_id: request.item_id,
            entry_time: entry.filled_at,
            entry_price: entry.fill_price,
            quantity: entry.filled_quantity,
            target_exit: request.target_exit,
            stop_loss: request.stop_loss,
            exit_time: Some(exit.filled_at),
            exit_price: Some(exit.fill_price),
            tax_paid,
            realized_profit_gp: Some(profit),
            realized_roi: realized_roi(entry.fill_price, entry.filled_quantity, profit),
            max_drawdown: exit
                .explanation
                .get("max_drawdown")
                .and_then(|value| value.as_f64()),
            hit_reason,
            status,
            explanation: serde_json::json!({
                "execution_mode": self.config.execution_mode,
                "entry": entry.explanation,
                "exit": exit.explanation,
            }),
        })
    }

    pub fn replay_user_position(
        &self,
        position: &UserPosition,
        history: &[IntervalPrice],
    ) -> Result<PaperBetOutcome, SimulatorError> {
        replay_user_position(&self.config, position, history)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};
    use grand_edge_domain::{ExecutionMode, Gp, IntervalPrice, ItemId, PriceInterval, StrategyId};
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    use super::SimulationEngine;
    use crate::{
        SimulatorConfig,
        orders::{SimulatedOrderRequest, SimulatedOrderSide, SimulatedOrderStatus},
    };

    fn row(
        hours_after_creation: i64,
        high: i64,
        low: i64,
        high_volume: i64,
        low_volume: i64,
    ) -> IntervalPrice {
        IntervalPrice {
            item_id: ItemId(4151),
            bucket_start: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap()
                + Duration::hours(hours_after_creation),
            interval: PriceInterval::OneHour,
            avg_high_price: Some(Gp(high)),
            high_price_volume: high_volume,
            avg_low_price: Some(Gp(low)),
            low_price_volume: low_volume,
        }
    }

    fn request() -> SimulatedOrderRequest {
        SimulatedOrderRequest {
            run_id: Uuid::new_v4(),
            strategy_id: StrategyId("spread_edge_v1".to_string()).0,
            model_version: "v1".to_string(),
            item_id: 4151,
            created_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            side: SimulatedOrderSide::Buy,
            quantity: 20,
            limit_price: Some(100_000),
            target_exit: Some(103_000),
            stop_loss: Some(99_000),
            horizon_secs: 21_600,
        }
    }

    fn engine(mode: ExecutionMode) -> SimulationEngine {
        let storage = grand_edge_storage::Storage::new(
            PgPoolOptions::new()
                .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
                .unwrap(),
        );
        SimulationEngine::new(
            storage,
            SimulatorConfig {
                execution_mode: mode,
                worst_case_slippage_gp: 100,
                ..SimulatorConfig::default()
            },
        )
    }

    #[tokio::test]
    async fn conservative_instant_execution_buys_high_sells_low() {
        let outcome = engine(ExecutionMode::ConservativeInstant)
            .simulate_from_history(
                request(),
                &[
                    row(1, 103_000, 100_000, 250, 170),
                    row(2, 103_500, 100_500, 250, 170),
                ],
            )
            .unwrap();
        assert_eq!(outcome.entry_price, 103_000);
        assert_eq!(outcome.exit_price, Some(100_500));
    }

    #[tokio::test]
    async fn passive_buy_does_not_fill_from_creation_bucket() {
        let outcome = engine(ExecutionMode::PassiveEstimated)
            .simulate_from_history(
                request(),
                &[
                    row(0, 102_000, 99_000, 250, 170),
                    row(1, 103_000, 100_001, 250, 170),
                    row(2, 104_000, 99_000, 250, 170),
                    row(3, 103_500, 100_500, 250, 170),
                ],
            )
            .unwrap();
        assert_eq!(
            outcome.entry_time,
            row(2, 104_000, 99_000, 250, 170).bucket_start
        );
    }

    #[tokio::test]
    async fn passive_sell_fills_on_future_high_cross() {
        let mut sell_request = request();
        sell_request.side = SimulatedOrderSide::Sell;
        sell_request.limit_price = Some(103_000);
        let outcome = engine(ExecutionMode::PassiveEstimated)
            .simulate_from_history(
                sell_request,
                &[
                    row(1, 102_000, 100_000, 250, 170),
                    row(2, 103_500, 100_500, 250, 170),
                    row(8, 101_000, 98_000, 200, 100),
                ],
            )
            .unwrap();
        assert_eq!(outcome.entry_price, 103_000);
    }

    #[tokio::test]
    async fn haircut_passive_quantity_uses_participation_and_confidence_haircut() {
        let outcome = engine(ExecutionMode::HaircutPassive)
            .simulate_from_history(
                request(),
                &[
                    row(1, 101_000, 100_000, 200, 200),
                    row(8, 103_000, 102_000, 200, 200),
                ],
            )
            .unwrap();
        assert_eq!(outcome.quantity, 10);
    }

    #[tokio::test]
    async fn worst_case_mode_applies_slippage() {
        let outcome = engine(ExecutionMode::WorstCase)
            .simulate_from_history(
                request(),
                &[
                    row(1, 103_000, 100_000, 250, 170),
                    row(7, 104_000, 102_000, 250, 170),
                ],
            )
            .unwrap();
        assert_eq!(outcome.entry_price, 103_100);
        assert!(
            serde_json::to_string(&outcome.explanation)
                .unwrap()
                .contains("worst_case_slippage_gp")
        );
    }

    #[tokio::test]
    async fn stop_loss_closes_with_hit_reason() {
        let outcome = engine(ExecutionMode::PassiveEstimated)
            .simulate_from_history(
                request(),
                &[
                    row(1, 101_000, 100_000, 250, 170),
                    row(2, 100_500, 98_000, 250, 170),
                ],
            )
            .unwrap();
        assert_eq!(outcome.hit_reason.as_deref(), Some("stop_loss"));
    }

    #[tokio::test]
    async fn take_profit_closes_with_hit_reason() {
        let outcome = engine(ExecutionMode::PassiveEstimated)
            .simulate_from_history(
                request(),
                &[
                    row(1, 101_000, 100_000, 250, 170),
                    row(2, 103_500, 100_500, 250, 170),
                ],
            )
            .unwrap();
        assert_eq!(outcome.hit_reason.as_deref(), Some("take_profit"));
    }

    #[tokio::test]
    async fn horizon_expiry_marks_order_expired() {
        let outcome = engine(ExecutionMode::PassiveEstimated)
            .simulate_from_history(
                request(),
                &[
                    row(1, 101_000, 100_000, 250, 170),
                    row(7, 101_500, 100_500, 250, 170),
                ],
            )
            .unwrap();
        assert_eq!(outcome.status, SimulatedOrderStatus::Expired);
        assert_eq!(outcome.hit_reason.as_deref(), Some("horizon_expiry"));
    }

    #[tokio::test]
    async fn tax_fixture_matches_goal() {
        let outcome = engine(ExecutionMode::PassiveEstimated)
            .simulate_from_history(
                request(),
                &[
                    row(1, 101_000, 100_000, 250, 170),
                    row(2, 103_000, 100_500, 250, 170),
                ],
            )
            .unwrap();
        assert_eq!(outcome.tax_paid / outcome.quantity, 2_060);
        assert_eq!(outcome.realized_profit_gp.unwrap() / outcome.quantity, 940);
        assert!(
            (outcome.realized_roi.unwrap() - 0.0094).abs() < 0.000_01,
            "roi was {}",
            outcome.realized_roi.unwrap()
        );
    }
}
