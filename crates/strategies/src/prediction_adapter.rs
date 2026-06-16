use chrono::{DateTime, Utc};
use grand_edge_domain::{
    Prediction, PredictionDirection, PredictionId, PredictionInterval, Rate, StrategySignal,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::StrategyError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionBatch {
    pub feature_snapshot_id: Uuid,
    pub item_id: grand_edge_domain::ItemId,
    pub as_of: DateTime<Utc>,
    pub predictions: Vec<Prediction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictionSource {
    Strategy,
    ModelRuntime,
    Baseline,
}

pub fn strategy_output_to_prediction(
    output: &StrategySignal,
    feature_snapshot_id: Uuid,
    created_at: DateTime<Utc>,
) -> Result<Prediction, StrategyError> {
    if feature_snapshot_id == Uuid::nil() {
        return Err(StrategyError::Validation(
            "feature_snapshot_id must not be nil".to_string(),
        ));
    }

    Ok(Prediction {
        prediction_id: PredictionId(Uuid::new_v4()),
        feature_snapshot_id,
        item_id: output.item_id,
        as_of: output.as_of,
        horizon_secs: output.horizon_secs,
        model_id: output.strategy_id.clone(),
        model_version: output.model_version.clone(),
        predicted_direction: prediction_direction(output),
        predicted_return: Some(output.expected_return),
        confidence: output.confidence,
        prediction_interval: prediction_interval(output),
        explanation: output.explanation.clone(),
        created_at,
    })
}

fn prediction_direction(signal: &StrategySignal) -> PredictionDirection {
    let expected_return = signal.expected_return.get();
    if expected_return > 0.0 {
        PredictionDirection::Up
    } else if expected_return < 0.0 {
        PredictionDirection::Down
    } else {
        PredictionDirection::Flat
    }
}

fn prediction_interval(signal: &StrategySignal) -> Option<PredictionInterval> {
    let spread = signal
        .execution_estimate
        .as_ref()
        .and_then(|estimate| estimate.spread_pct)
        .unwrap_or_else(|| Rate::new(0.0).expect("zero spread is valid"));
    let volatility = signal
        .execution_estimate
        .as_ref()
        .and_then(|estimate| estimate.volatility)
        .unwrap_or_else(|| Rate::new(0.0).expect("zero volatility is valid"));
    let width = spread.get().abs().max(volatility.get().abs());
    if width == 0.0 {
        return None;
    }

    Some(PredictionInterval {
        low: Rate::new(signal.expected_return.get() - width).ok(),
        high: Rate::new(signal.expected_return.get() + width).ok(),
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        ExecutionEstimate, Gp, HorizonSecs, ItemId, ModelVersion, ObservedLiquidityProxy,
        Probability, Quantity, Rate, SignalSide, StrategyId, StrategySignal,
    };
    use uuid::Uuid;

    use super::{PredictionSource, strategy_output_to_prediction};

    #[test]
    fn prediction_source_serde_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&PredictionSource::ModelRuntime).unwrap(),
            "\"model_runtime\""
        );
    }

    #[test]
    fn strategy_output_to_prediction_preserves_feature_snapshot_id() {
        let feature_snapshot_id = Uuid::new_v4();
        let prediction =
            strategy_output_to_prediction(&signal(), feature_snapshot_id, signal().as_of).unwrap();
        assert_eq!(prediction.feature_snapshot_id, feature_snapshot_id);
        assert_eq!(prediction.model_id.0, "spread_edge_v1");
        assert!(prediction.prediction_interval.is_some());
    }

    fn signal() -> StrategySignal {
        StrategySignal {
            item_id: ItemId(4151),
            strategy_id: StrategyId::new("spread_edge_v1").unwrap(),
            model_version: ModelVersion::new("2026-06-16.1").unwrap(),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            side: SignalSide::Buy,
            horizon_secs: HorizonSecs(3_600),
            confidence: Probability::new(0.8).unwrap(),
            expected_return: Rate::new(0.04).unwrap(),
            expected_net_gp_per_unit: Gp(1_200),
            target_entry: Some(Gp(100_000)),
            target_exit: Some(Gp(104_000)),
            stop_loss: Some(Gp(99_000)),
            take_profit: Some(Gp(104_500)),
            max_quantity: Some(Quantity(8)),
            execution_estimate: Some(ExecutionEstimate {
                observed_liquidity: ObservedLiquidityProxy {
                    observed_volume: Quantity(500),
                    observed_high_side_volume: Quantity(260),
                    observed_low_side_volume: Quantity(240),
                    observed_volume_z: None,
                    observed_volume_reliability: None,
                    high_low_volume_ratio: None,
                    note: "proxy".to_string(),
                },
                estimated_fill_probability: Some(Probability::new(0.7).unwrap()),
                liquidity_confidence: Some(Probability::new(0.75).unwrap()),
                estimated_capacity: Some(Quantity(8)),
                participation_rate: None,
                confidence_haircut: None,
                spread_pct: Some(Rate::new(0.02).unwrap()),
                price_staleness_seconds: Some(HorizonSecs(60)),
                volatility: Some(Rate::new(0.03).unwrap()),
            }),
            explanation: serde_json::json!({"strategy": "spread_edge"}),
        }
    }
}
