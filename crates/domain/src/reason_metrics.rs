use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{ExecutionMode, Gp, ModelVersion, Probability, Rate, ReasonType, RecommendationAction};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasonOutcomeSummary {
    pub reason_type: ReasonType,
    pub reason_key: String,
    pub model_version: ModelVersion,
    pub recommendation_action: RecommendationAction,
    #[serde(default)]
    pub execution_mode: Option<ExecutionMode>,
    #[serde(default)]
    pub confidence_bucket: Option<String>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub sample_size: i64,
    pub publishable: bool,
    pub win_rate: Option<Probability>,
    pub avg_actual_return: Option<Rate>,
    pub avg_net_gp: Option<Gp>,
    pub calibration_error: Option<f64>,
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::{
        ExecutionMode, Gp, ModelVersion, Probability, Rate, ReasonOutcomeSummary, ReasonType,
        RecommendationAction,
    };

    #[test]
    fn reason_outcome_summary_serde_preserves_optional_dimensions() {
        let summary = ReasonOutcomeSummary {
            reason_type: ReasonType::LiquidityCheck,
            reason_key: "liquidity:volume_capacity".to_string(),
            model_version: ModelVersion::new("2026-06-16.1").unwrap(),
            recommendation_action: RecommendationAction::Buy,
            execution_mode: Some(ExecutionMode::PassiveEstimated),
            confidence_bucket: Some("0.6-0.7".to_string()),
            window_start: Utc.with_ymd_and_hms(2026, 6, 9, 0, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            sample_size: 4,
            publishable: true,
            win_rate: Some(Probability::new(0.75).unwrap()),
            avg_actual_return: Some(Rate::new(0.02).unwrap()),
            avg_net_gp: Some(Gp(12_000)),
            calibration_error: Some(0.08),
        };

        let json = serde_json::to_value(&summary).unwrap();
        assert_eq!(json.get("recommendation_action").unwrap(), "buy");
        assert_eq!(json.get("execution_mode").unwrap(), "passive_estimated");
        assert_eq!(json.get("confidence_bucket").unwrap(), "0.6-0.7");
        assert_eq!(json.get("publishable").unwrap(), true);
    }
}
