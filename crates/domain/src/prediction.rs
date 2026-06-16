use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    DomainValidationError, HorizonSecs, ItemId, ModelVersion, PredictionId, Probability, Rate,
    StrategyId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictionDirection {
    Up,
    Down,
    Flat,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictionInterval {
    pub low: Option<Rate>,
    pub high: Option<Rate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    pub prediction_id: PredictionId,
    pub feature_snapshot_id: Uuid,
    pub item_id: ItemId,
    pub as_of: DateTime<Utc>,
    pub horizon_secs: HorizonSecs,
    pub model_id: StrategyId,
    pub model_version: ModelVersion,
    pub predicted_direction: PredictionDirection,
    pub predicted_return: Option<Rate>,
    pub confidence: Probability,
    pub prediction_interval: Option<PredictionInterval>,
    pub explanation: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl Prediction {
    pub fn validate_feature_snapshot_id(&self) -> Result<(), DomainValidationError> {
        if self.feature_snapshot_id == Uuid::nil() {
            return Err(DomainValidationError::EmptyValue {
                field: "feature_snapshot_id",
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use crate::{
        HorizonSecs, ItemId, ModelVersion, Prediction, PredictionDirection, PredictionId,
        Probability, Rate, StrategyId,
    };

    #[test]
    fn prediction_direction_serde_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&PredictionDirection::Unknown).unwrap(),
            "\"unknown\""
        );
    }

    #[test]
    fn prediction_keeps_recommendation_action_out_of_contract() {
        let prediction = Prediction {
            prediction_id: PredictionId(Uuid::new_v4()),
            feature_snapshot_id: Uuid::new_v4(),
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            horizon_secs: HorizonSecs(21_600),
            model_id: StrategyId::new("gbm_ranker_v1").unwrap(),
            model_version: ModelVersion::new("2026-06-16.1").unwrap(),
            predicted_direction: PredictionDirection::Up,
            predicted_return: Some(Rate::new(0.03).unwrap()),
            confidence: Probability::new(0.62).unwrap(),
            prediction_interval: None,
            explanation: serde_json::json!({"reason": "fixture"}),
            created_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        };

        let serialized = serde_json::to_string(&prediction).unwrap();
        assert!(serialized.contains("feature_snapshot_id"));
        assert!(!serialized.contains("action"));
    }
}
