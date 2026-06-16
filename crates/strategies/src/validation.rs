use grand_edge_domain::{
    Gp, HorizonSecs, ItemId, ModelVersion, Probability, StrategyId, StrategySignal,
};

use crate::StrategyError;

pub fn validate_signal(
    signal: &StrategySignal,
    expected_strategy_id: &str,
    expected_version: &str,
) -> Result<(), StrategyError> {
    StrategyId::new(expected_strategy_id)
        .map_err(|error| StrategyError::Validation(error.to_string()))?;
    ModelVersion::new(expected_version)
        .map_err(|error| StrategyError::Validation(error.to_string()))?;

    if signal.strategy_id.0 != expected_strategy_id {
        return Err(StrategyError::Validation(
            "signal strategy_id did not match registered strategy".to_string(),
        ));
    }
    if signal.model_version.0 != expected_version {
        return Err(StrategyError::Validation(
            "signal model_version did not match registered strategy version".to_string(),
        ));
    }
    ItemId::try_from(signal.item_id.0)
        .map_err(|error| StrategyError::Validation(error.to_string()))?;
    HorizonSecs::try_from(signal.horizon_secs.0)
        .map_err(|error| StrategyError::Validation(error.to_string()))?;
    Probability::new(signal.confidence.get())
        .map_err(|error| StrategyError::Validation(error.to_string()))?;
    if !signal.expected_return.get().is_finite() {
        return Err(StrategyError::Validation(
            "expected_return must be finite".to_string(),
        ));
    }
    if !signal.explanation.is_object() {
        return Err(StrategyError::Validation(
            "explanation must be a JSON object".to_string(),
        ));
    }
    validate_positive_gp(signal.target_entry, "target_entry")?;
    validate_positive_gp(signal.target_exit, "target_exit")?;
    validate_positive_gp(signal.stop_loss, "stop_loss")?;
    validate_positive_gp(signal.take_profit, "take_profit")?;

    Ok(())
}

fn validate_positive_gp(value: Option<Gp>, field: &str) -> Result<(), StrategyError> {
    if let Some(value) = value {
        if value.0 <= 0 {
            return Err(StrategyError::Validation(format!(
                "{field} must be positive when present"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        FeatureVector, Gp, HorizonSecs, ItemId, ModelVersion, Probability, Rate, SignalSide,
        StrategyId, StrategySignal,
    };

    use super::validate_signal;

    fn valid_signal() -> StrategySignal {
        StrategySignal {
            item_id: ItemId(4151),
            strategy_id: StrategyId("noop".to_string()),
            model_version: ModelVersion("v1".to_string()),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            side: SignalSide::Buy,
            horizon_secs: HorizonSecs(3600),
            confidence: Probability::new(0.7).unwrap(),
            expected_return: Rate::new(0.03).unwrap(),
            expected_net_gp_per_unit: Gp(100),
            target_entry: Some(Gp(100)),
            target_exit: Some(Gp(110)),
            stop_loss: Some(Gp(95)),
            take_profit: Some(Gp(120)),
            max_quantity: None,
            execution_estimate: None,
            explanation: serde_json::json!({ "reason": "test" }),
        }
    }

    #[test]
    fn validate_signal_rejects_invalid_confidence() {
        let mut signal = valid_signal();
        signal.confidence = Probability(1.5);
        assert!(validate_signal(&signal, "noop", "v1").is_err());
    }

    #[test]
    fn validate_signal_requires_explanation_object() {
        let mut signal = valid_signal();
        signal.explanation = serde_json::Value::Null;
        assert!(validate_signal(&signal, "noop", "v1").is_err());
        let _ = FeatureVector {
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values: serde_json::Map::new(),
        };
    }
}
