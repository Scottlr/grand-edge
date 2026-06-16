use grand_edge_domain::{IntervalPrice, UserPosition};

use crate::{
    SimulatorConfig, SimulatorError,
    fills::{entry_fill, exit_fill},
    orders::{PaperBetOutcome, SimulatedOrderRequest, SimulatedOrderSide, SimulatedOrderStatus},
    pnl::{realized_profit_gp, realized_roi, tax_on_sale},
};

pub fn replay_user_position(
    config: &SimulatorConfig,
    position: &UserPosition,
    history: &[IntervalPrice],
) -> Result<PaperBetOutcome, SimulatorError> {
    let request = SimulatedOrderRequest {
        run_id: uuid::Uuid::new_v4(),
        strategy_id: "user_position_replay".to_string(),
        model_version: "v1".to_string(),
        item_id: position.item_id.0,
        created_at: position.bought_at.unwrap_or_else(chrono::Utc::now),
        side: SimulatedOrderSide::Sell,
        quantity: position.quantity.as_i64(),
        limit_price: Some(position.avg_buy_price.as_i64()),
        target_exit: None,
        stop_loss: None,
        horizon_secs: config.default_horizon_secs,
    };
    let entry = entry_fill(config, &request, history)?;
    let exit = exit_fill(config, &entry, &request, history)?;
    let tax_paid =
        tax_on_sale(&config.market_rules, request.item_id, exit.fill_price) * entry.filled_quantity;
    let profit = realized_profit_gp(
        &config.market_rules,
        request.item_id,
        entry.fill_price,
        exit.fill_price,
        entry.filled_quantity,
    );

    Ok(PaperBetOutcome {
        bet_id: uuid::Uuid::new_v4(),
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
        hit_reason: exit
            .explanation
            .get("hit_reason")
            .and_then(|value| value.as_str())
            .map(ToString::to_string),
        status: SimulatedOrderStatus::Closed,
        explanation: serde_json::json!({
            "execution_mode": config.execution_mode,
            "replay": true,
            "entry": entry.explanation,
            "exit": exit.explanation,
        }),
    })
}
