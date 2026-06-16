use chrono::{DateTime, Utc};
use grand_edge_domain::{ModelAccuracySnapshot, ModelVersion, Rate, StrategyId};
use grand_edge_storage::{Storage, StoredPaperBet, StoredPrediction};
use serde::{Deserialize, Serialize};

use crate::{
    CalibrationMetrics, ExecutionQualityMetrics, ForecastMetrics, MetricWindow, MetricsError,
    RiskAdjustedMetrics, TradingMetrics,
    calibration::{calibration_buckets, execution_quality},
    forecast::{brier_score, directional_accuracy, mean_absolute_error, root_mean_squared_error},
    graph::{
        BlastRadiusMetricSummary, BlastRadiusOutcome, GraphPathMetricSummary, GraphPathOutcome,
        summarize_blast_radius_outcomes, summarize_graph_path_outcomes,
    },
    risk::{max_drawdown, sharpe_ratio, sortino_ratio},
    trading::{average_loss, average_win, capital_efficiency, profit_factor, win_rate},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetricSummary {
    pub strategy_id: String,
    pub model_version: String,
    pub horizon_secs: i64,
    pub window: MetricWindow,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub sample_size: i64,
    pub forecast: ForecastMetrics,
    pub trading: TradingMetrics,
    pub risk: RiskAdjustedMetrics,
    pub calibration: CalibrationMetrics,
    pub execution: ExecutionQualityMetrics,
}

pub struct MetricsEngine {
    storage: Storage,
}

impl MetricsEngine {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn compute_for_strategy(
        &self,
        strategy_id: &str,
        model_version: &str,
        horizon_secs: i64,
        window: MetricWindow,
        as_of: DateTime<Utc>,
    ) -> Result<StrategyMetricSummary, MetricsError> {
        let (window_start, window_end) = window.bounds(as_of);
        let predictions = self
            .storage
            .strategies()
            .list_predictions_for_strategy(
                strategy_id,
                model_version,
                horizon_secs,
                window_start,
                window_end,
            )
            .await?;
        let paper_bets = self
            .storage
            .simulations()
            .list_paper_bets_for_strategy(strategy_id, model_version, window_start, window_end)
            .await?;

        let matched_predictions = match_predictions(&predictions, &paper_bets);
        let summary = build_summary(
            strategy_id,
            model_version,
            horizon_secs,
            window,
            window_start,
            window_end,
            &predictions,
            &paper_bets,
            &matched_predictions,
        );

        let snapshot = summary.to_accuracy_snapshot()?;
        self.storage
            .metrics()
            .insert_strategy_metric(
                &snapshot,
                horizon_secs,
                window.as_name(),
                window_start,
                window_end,
            )
            .await?;

        Ok(summary)
    }

    pub async fn latest_accuracy_snapshot(
        &self,
        strategy_id: &str,
        model_version: &str,
        window: MetricWindow,
    ) -> Result<Option<ModelAccuracySnapshot>, MetricsError> {
        Ok(self
            .storage
            .metrics()
            .latest_strategy_metric(strategy_id, model_version, window.as_name())
            .await?)
    }

    pub async fn persist_graph_path_summaries(
        &self,
        outcomes: &[GraphPathOutcome],
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
        min_sample_size: i64,
    ) -> Result<Vec<GraphPathMetricSummary>, MetricsError> {
        let summaries =
            summarize_graph_path_outcomes(outcomes, window_start, window_end, min_sample_size);
        for summary in &summaries {
            self.storage
                .metrics()
                .upsert_metric_payloads(
                    "__graph_path_metrics__",
                    &summary.graph_version,
                    summary.path_length as i64,
                    &format!(
                        "graph_path:{}:{}:{}:{}:{}",
                        serde_json::to_string(&summary.graph_action)
                            .map_err(MetricsError::from)?
                            .trim_matches('"'),
                        summary.edge_type,
                        summary.source_type,
                        summary.execution_mode,
                        summary.confidence_bucket,
                    ),
                    summary.window_start,
                    summary.window_end,
                    &[serde_json::to_value(summary).map_err(MetricsError::from)?],
                )
                .await?;
        }
        Ok(summaries)
    }

    pub async fn persist_blast_radius_summaries(
        &self,
        outcomes: &[BlastRadiusOutcome],
        min_sample_size: i64,
    ) -> Result<Vec<BlastRadiusMetricSummary>, MetricsError> {
        let summaries = summarize_blast_radius_outcomes(outcomes, min_sample_size);
        for summary in &summaries {
            self.storage
                .metrics()
                .upsert_metric_payloads(
                    "__blast_radius_metrics__",
                    &summary.graph_version,
                    summary.horizon_secs,
                    &format!("blast_radius:{}", summary.scenario_mode),
                    DateTime::<Utc>::UNIX_EPOCH,
                    DateTime::<Utc>::UNIX_EPOCH,
                    &[serde_json::to_value(summary).map_err(MetricsError::from)?],
                )
                .await?;
        }
        Ok(summaries)
    }
}

impl StrategyMetricSummary {
    pub fn to_accuracy_snapshot(&self) -> Result<ModelAccuracySnapshot, MetricsError> {
        Ok(ModelAccuracySnapshot {
            strategy_id: StrategyId::new(self.strategy_id.clone())?,
            model_version: ModelVersion::new(self.model_version.clone())?,
            lookback_window: self.window.as_name().to_string(),
            sample_size: self.sample_size,
            directional_accuracy: self
                .forecast
                .directional_accuracy
                .map(Rate::new)
                .transpose()?,
            brier_score: self.forecast.brier_score.map(Rate::new).transpose()?,
            avg_realized_roi: average_realized_roi(&self.trading),
            max_drawdown: self.risk.max_drawdown.map(Rate::new).transpose()?,
            calibration: serde_json::to_value(&self.calibration)?,
        })
    }
}

fn build_summary(
    strategy_id: &str,
    model_version: &str,
    horizon_secs: i64,
    window: MetricWindow,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
    predictions: &[StoredPrediction],
    paper_bets: &[StoredPaperBet],
    matched_predictions: &[Option<usize>],
) -> StrategyMetricSummary {
    let matched_pairs = paper_bets
        .iter()
        .enumerate()
        .filter_map(|(index, paper_bet)| {
            let prediction = matched_predictions
                .get(index)
                .and_then(|position| position.map(|position| &predictions[position]))?;
            let realized_roi = paper_bet.realized_roi?;
            Some((prediction, paper_bet, realized_roi))
        })
        .collect::<Vec<_>>();

    let actual_returns = matched_pairs
        .iter()
        .map(|(_, _, realized_roi)| *realized_roi)
        .collect::<Vec<_>>();
    let predicted_returns = matched_pairs
        .iter()
        .map(|(prediction, _, _)| prediction.expected_return.get())
        .collect::<Vec<_>>();
    let outcomes = matched_pairs
        .iter()
        .map(|(_, paper_bet, _)| paper_bet.realized_profit_gp.unwrap_or_default() > 0)
        .collect::<Vec<_>>();
    let probabilities = matched_pairs
        .iter()
        .map(|(prediction, _, _)| prediction.confidence.get())
        .collect::<Vec<_>>();
    let profits = paper_bets
        .iter()
        .filter_map(|paper_bet| paper_bet.realized_profit_gp)
        .collect::<Vec<_>>();
    let entry_notional = paper_bets
        .iter()
        .map(|paper_bet| paper_bet.entry_price.as_i64() as f64 * paper_bet.quantity as f64)
        .collect::<Vec<_>>();
    let forecast = ForecastMetrics {
        mae: mean_absolute_error(&actual_returns, &predicted_returns),
        rmse: root_mean_squared_error(&actual_returns, &predicted_returns),
        directional_accuracy: directional_accuracy(&actual_returns, &predicted_returns),
        brier_score: brier_score(&outcomes, &probabilities),
    };
    let risk = RiskAdjustedMetrics {
        sharpe: sharpe_ratio(&actual_returns),
        sortino: sortino_ratio(&actual_returns),
        max_drawdown: max_drawdown(&actual_returns),
        deflated_sharpe_ratio: None,
        probability_of_backtest_overfitting: None,
        placeholder_reason: Some(
            "Deflated Sharpe and PBO remain placeholders for MVP.".to_string(),
        ),
    };
    let trading = TradingMetrics {
        gross_profit_gp: profits.iter().copied().filter(|value| *value > 0).sum(),
        net_profit_gp: profits.iter().sum(),
        win_rate: win_rate(&profits),
        avg_win_gp: average_win(&profits),
        avg_loss_gp: average_loss(&profits),
        profit_factor: profit_factor(&profits),
        max_drawdown: risk.max_drawdown,
        capital_efficiency: capital_efficiency(&entry_notional, profits.iter().sum()),
    };
    let calibration = CalibrationMetrics {
        buckets: calibration_buckets(&outcomes, &probabilities),
        execution_confidence_buckets: Vec::new(),
    };
    let execution = execution_quality(predictions, paper_bets, matched_predictions);

    StrategyMetricSummary {
        strategy_id: strategy_id.to_string(),
        model_version: model_version.to_string(),
        horizon_secs,
        window,
        window_start,
        window_end,
        sample_size: matched_pairs.len() as i64,
        forecast,
        trading,
        risk,
        calibration,
        execution,
    }
}

fn match_predictions(
    predictions: &[StoredPrediction],
    paper_bets: &[StoredPaperBet],
) -> Vec<Option<usize>> {
    paper_bets
        .iter()
        .map(|paper_bet| {
            predictions
                .iter()
                .enumerate()
                .filter(|(_, prediction)| {
                    prediction.item_id == paper_bet.item_id
                        && prediction.as_of <= paper_bet.entry_time
                })
                .max_by_key(|(_, prediction)| prediction.as_of)
                .map(|(index, _)| index)
        })
        .collect()
}

fn average_realized_roi(trading: &TradingMetrics) -> Option<grand_edge_domain::Rate> {
    trading
        .capital_efficiency
        .map(Rate::new)
        .transpose()
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{Gp, ItemId, ModelVersion, Probability, Rate, SignalSide, StrategyId};
    use uuid::Uuid;

    use super::{MetricWindow, StrategyMetricSummary, build_summary, match_predictions};
    use crate::{
        CalibrationMetrics, ExecutionQualityMetrics, ForecastMetrics, RiskAdjustedMetrics,
        TradingMetrics,
    };
    use grand_edge_storage::{StoredPaperBet, StoredPrediction};

    #[test]
    fn model_accuracy_snapshot_can_be_produced_for_ui() {
        let summary = StrategyMetricSummary {
            strategy_id: "momentum_v1".to_string(),
            model_version: "v1".to_string(),
            horizon_secs: 21_600,
            window: MetricWindow::SevenDays,
            window_start: Utc.with_ymd_and_hms(2026, 6, 9, 12, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            sample_size: 3,
            forecast: ForecastMetrics {
                mae: Some(0.1),
                rmse: Some(0.2),
                directional_accuracy: Some(0.66),
                brier_score: Some(0.18),
            },
            trading: TradingMetrics {
                net_profit_gp: 1000,
                capital_efficiency: Some(0.05),
                ..TradingMetrics::default()
            },
            risk: RiskAdjustedMetrics {
                max_drawdown: Some(0.12),
                ..RiskAdjustedMetrics::default()
            },
            calibration: CalibrationMetrics::default(),
            execution: ExecutionQualityMetrics::default(),
        };

        let snapshot = summary.to_accuracy_snapshot().unwrap();
        assert_eq!(snapshot.lookback_window, "seven_days");
        assert_eq!(snapshot.sample_size, 3);
    }

    #[test]
    fn build_summary_uses_matched_prediction_samples() {
        let predictions = vec![prediction(
            Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            0.03,
            0.8,
        )];
        let paper_bets = vec![paper_bet(
            Utc.with_ymd_and_hms(2026, 6, 16, 13, 0, 0).unwrap(),
            0.02,
            1200,
        )];
        let matched = match_predictions(&predictions, &paper_bets);
        let summary = build_summary(
            "momentum_v1",
            "v1",
            21_600,
            MetricWindow::SevenDays,
            Utc.with_ymd_and_hms(2026, 6, 9, 12, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            &predictions,
            &paper_bets,
            &matched,
        );

        assert_eq!(summary.sample_size, 1);
        assert_eq!(summary.trading.net_profit_gp, 1200);
    }

    fn prediction(
        as_of: chrono::DateTime<Utc>,
        expected_return: f64,
        confidence: f64,
    ) -> StoredPrediction {
        StoredPrediction {
            strategy_id: StrategyId::new("momentum_v1").unwrap(),
            model_version: ModelVersion::new("v1").unwrap(),
            item_id: ItemId(4151),
            as_of,
            horizon_secs: 21_600,
            side: SignalSide::Buy,
            expected_return: Rate::new(expected_return).unwrap(),
            confidence: Probability::new(confidence).unwrap(),
            expected_net_gp_per_unit: Gp(1000),
            target_entry: None,
            target_exit: None,
            stop_loss: None,
            take_profit: None,
            max_quantity: None,
            explanation: serde_json::json!({}),
        }
    }

    fn paper_bet(
        entry_time: chrono::DateTime<Utc>,
        realized_roi: f64,
        realized_profit_gp: i64,
    ) -> StoredPaperBet {
        StoredPaperBet {
            bet_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            recommendation_id: None,
            strategy_id: StrategyId::new("momentum_v1").unwrap(),
            model_version: ModelVersion::new("v1").unwrap(),
            item_id: ItemId(4151),
            entry_time,
            entry_price: Gp(100_000),
            quantity: 10,
            target_exit: None,
            stop_loss: None,
            exit_time: Some(entry_time),
            exit_price: Some(Gp(101_000)),
            tax_paid: 2060,
            realized_profit_gp: Some(realized_profit_gp),
            realized_roi: Some(realized_roi),
            max_drawdown: Some(0.05),
            hit_reason: Some("take_profit".to_string()),
            status: "closed".to_string(),
            explanation: serde_json::json!({ "execution_mode": "conservative_instant" }),
        }
    }
}
