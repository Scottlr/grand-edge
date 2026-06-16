use chrono::{DateTime, Utc};
use grand_edge_domain::{ExecutionMode, IntervalPrice};
use serde_json::json;

use crate::{
    SimulatorConfig, SimulatorError,
    orders::{SimulatedOrderRequest, SimulatedOrderSide},
};

#[derive(Debug, Clone)]
pub struct FillDecision {
    pub filled_at: DateTime<Utc>,
    pub fill_price: i64,
    pub filled_quantity: i64,
    pub explanation: serde_json::Value,
}

pub fn entry_fill(
    config: &SimulatorConfig,
    request: &SimulatedOrderRequest,
    history: &[IntervalPrice],
) -> Result<FillDecision, SimulatorError> {
    let future_rows = future_rows(request.created_at, history);
    match config.execution_mode {
        ExecutionMode::ConservativeInstant => conservative_instant_fill(request, &future_rows),
        ExecutionMode::PassiveEstimated => passive_estimated_fill(request, &future_rows),
        ExecutionMode::HaircutPassive => haircut_passive_fill(config, request, &future_rows),
        ExecutionMode::WorstCase => worst_case_fill(config, request, &future_rows),
        ExecutionMode::UserPositionReplay => passive_estimated_fill(request, &future_rows),
    }
}

pub fn exit_fill(
    config: &SimulatorConfig,
    entry: &FillDecision,
    request: &SimulatedOrderRequest,
    history: &[IntervalPrice],
) -> Result<FillDecision, SimulatorError> {
    let future_rows = future_rows(entry.filled_at, history);
    if matches!(config.execution_mode, ExecutionMode::ConservativeInstant) {
        return conservative_exit_fill(request, &future_rows);
    }
    let horizon_at = request.created_at + chrono::Duration::seconds(request.horizon_secs.max(0));

    let mut max_drawdown = 0.0_f64;
    for row in &future_rows {
        let low = row.avg_low_price.map(|value| value.as_i64());
        let high = row.avg_high_price.map(|value| value.as_i64());
        if let Some(low) = low {
            let drawdown = (entry.fill_price - low) as f64 / entry.fill_price as f64;
            max_drawdown = max_drawdown.max(drawdown);
        }

        if let Some(stop_loss) = request.stop_loss {
            if low.is_some_and(|value| value <= stop_loss) {
                return Ok(exit_decision(
                    row.bucket_start,
                    stop_loss.saturating_sub(config.emergency_exit_slippage_gp),
                    request.quantity,
                    "stop_loss",
                    config.execution_mode,
                    max_drawdown,
                ));
            }
        }
        if let Some(target_exit) = request.target_exit {
            if high.is_some_and(|value| value >= target_exit) {
                return Ok(exit_decision(
                    row.bucket_start,
                    target_exit,
                    request.quantity,
                    "take_profit",
                    config.execution_mode,
                    max_drawdown,
                ));
            }
        }
        if row.bucket_start >= horizon_at {
            let fallback = match request.side {
                SimulatedOrderSide::Buy => low.or(high),
                SimulatedOrderSide::Sell => high.or(low),
            }
            .ok_or(SimulatorError::NoExit)?;
            return Ok(exit_decision(
                row.bucket_start,
                apply_exit_mode_adjustment(
                    config.execution_mode,
                    fallback,
                    config.worst_case_slippage_gp,
                ),
                request.quantity,
                "horizon_expiry",
                config.execution_mode,
                max_drawdown,
            ));
        }
    }

    Err(SimulatorError::NoExit)
}

fn conservative_exit_fill(
    request: &SimulatedOrderRequest,
    future_rows: &[IntervalPrice],
) -> Result<FillDecision, SimulatorError> {
    let row = future_rows.first().ok_or(SimulatorError::NoExit)?;
    let fill_price = match request.side {
        SimulatedOrderSide::Buy => row.avg_low_price.map(|value| value.as_i64()),
        SimulatedOrderSide::Sell => row.avg_high_price.map(|value| value.as_i64()),
    }
    .ok_or(SimulatorError::NoExit)?;

    Ok(exit_decision(
        row.bucket_start,
        fill_price,
        request.quantity,
        "conservative_exit",
        ExecutionMode::ConservativeInstant,
        0.0,
    ))
}

fn conservative_instant_fill(
    request: &SimulatedOrderRequest,
    future_rows: &[IntervalPrice],
) -> Result<FillDecision, SimulatorError> {
    let row = future_rows.first().ok_or(SimulatorError::NoFill)?;
    let fill_price = match request.side {
        SimulatedOrderSide::Buy => row.avg_high_price.map(|value| value.as_i64()),
        SimulatedOrderSide::Sell => row.avg_low_price.map(|value| value.as_i64()),
    }
    .ok_or(SimulatorError::NoFill)?;

    Ok(FillDecision {
        filled_at: row.bucket_start,
        fill_price,
        filled_quantity: request.quantity,
        explanation: json!({
            "execution_mode": "conservative_instant",
            "requested_quantity": request.quantity,
            "filled_quantity": request.quantity,
            "conservative": true,
        }),
    })
}

fn passive_estimated_fill(
    request: &SimulatedOrderRequest,
    future_rows: &[IntervalPrice],
) -> Result<FillDecision, SimulatorError> {
    let limit_price = request.limit_price.ok_or_else(|| {
        SimulatorError::InvalidRequest("passive fill requires limit_price".to_string())
    })?;
    for row in future_rows {
        let touched = match request.side {
            SimulatedOrderSide::Buy => row
                .avg_low_price
                .map(|value| value.as_i64())
                .is_some_and(|value| value <= limit_price),
            SimulatedOrderSide::Sell => row
                .avg_high_price
                .map(|value| value.as_i64())
                .is_some_and(|value| value >= limit_price),
        };
        if touched {
            return Ok(FillDecision {
                filled_at: row.bucket_start,
                fill_price: limit_price,
                filled_quantity: request.quantity,
                explanation: json!({
                    "execution_mode": "passive_estimated",
                    "requested_quantity": request.quantity,
                    "filled_quantity": request.quantity,
                    "proxy_estimated": true,
                    "observed_volume": row.high_price_volume + row.low_price_volume,
                }),
            });
        }
    }

    Err(SimulatorError::NoFill)
}

fn haircut_passive_fill(
    config: &SimulatorConfig,
    request: &SimulatedOrderRequest,
    future_rows: &[IntervalPrice],
) -> Result<FillDecision, SimulatorError> {
    let limit_price = request.limit_price.ok_or_else(|| {
        SimulatorError::InvalidRequest("haircut passive fill requires limit_price".to_string())
    })?;
    for row in future_rows {
        let touched = match request.side {
            SimulatedOrderSide::Buy => row
                .avg_low_price
                .map(|value| value.as_i64())
                .is_some_and(|value| value <= limit_price),
            SimulatedOrderSide::Sell => row
                .avg_high_price
                .map(|value| value.as_i64())
                .is_some_and(|value| value >= limit_price),
        };
        if touched {
            let observed_volume = row.high_price_volume + row.low_price_volume;
            let quantity_cap =
                ((observed_volume as f64) * config.participation_rate * config.confidence_haircut)
                    .floor() as i64;
            let filled_quantity = request.quantity.min(quantity_cap).max(0);
            if filled_quantity == 0 {
                return Err(SimulatorError::NoFill);
            }
            return Ok(FillDecision {
                filled_at: row.bucket_start,
                fill_price: limit_price,
                filled_quantity,
                explanation: json!({
                    "execution_mode": "haircut_passive",
                    "observed_volume": observed_volume,
                    "participation_rate": config.participation_rate,
                    "confidence_haircut": config.confidence_haircut,
                    "requested_quantity": request.quantity,
                    "filled_quantity": filled_quantity,
                    "proxy_estimated": true,
                }),
            });
        }
    }

    Err(SimulatorError::NoFill)
}

fn worst_case_fill(
    config: &SimulatorConfig,
    request: &SimulatedOrderRequest,
    future_rows: &[IntervalPrice],
) -> Result<FillDecision, SimulatorError> {
    let row = future_rows.first().ok_or(SimulatorError::NoFill)?;
    let base = match request.side {
        SimulatedOrderSide::Buy => row.avg_high_price.map(|value| value.as_i64()),
        SimulatedOrderSide::Sell => row.avg_low_price.map(|value| value.as_i64()),
    }
    .ok_or(SimulatorError::NoFill)?;
    let fill_price = match request.side {
        SimulatedOrderSide::Buy => base.saturating_add(config.worst_case_slippage_gp),
        SimulatedOrderSide::Sell => base.saturating_sub(config.worst_case_slippage_gp),
    };

    Ok(FillDecision {
        filled_at: row.bucket_start,
        fill_price,
        filled_quantity: request.quantity,
        explanation: json!({
            "execution_mode": "worst_case",
            "requested_quantity": request.quantity,
            "filled_quantity": request.quantity,
            "worst_case_slippage_gp": config.worst_case_slippage_gp,
            "assumption": "worst_case delayed and adverse slippage",
        }),
    })
}

fn future_rows(created_at: DateTime<Utc>, history: &[IntervalPrice]) -> Vec<IntervalPrice> {
    let mut rows = history
        .iter()
        .filter(|row| row.bucket_start > created_at)
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.bucket_start);
    rows
}

fn exit_decision(
    filled_at: DateTime<Utc>,
    fill_price: i64,
    filled_quantity: i64,
    hit_reason: &str,
    execution_mode: ExecutionMode,
    max_drawdown: f64,
) -> FillDecision {
    FillDecision {
        filled_at,
        fill_price,
        filled_quantity,
        explanation: json!({
            "execution_mode": execution_mode,
            "hit_reason": hit_reason,
            "filled_quantity": filled_quantity,
            "max_drawdown": max_drawdown,
        }),
    }
}

fn apply_exit_mode_adjustment(execution_mode: ExecutionMode, price: i64, slippage: i64) -> i64 {
    match execution_mode {
        ExecutionMode::WorstCase => price.saturating_sub(slippage),
        _ => price,
    }
}
