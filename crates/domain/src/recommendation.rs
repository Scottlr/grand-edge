use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ModelAccuracySnapshot, StrategySignal};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationAction {
    Buy,
    Add,
    Hold,
    Cashout,
    Avoid,
    Watch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreComponent {
    pub key: String,
    pub label: String,
    pub value: f64,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationExplanation {
    pub feature_set_version: String,
    pub market_rules_version: String,
    pub strategy_votes: Vec<StrategySignal>,
    pub score_components: Vec<ScoreComponent>,
    pub accuracy_snapshot: Option<ModelAccuracySnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommendation_id: Uuid,
    pub user_id: Option<Uuid>,
    pub item_id: i64,
    pub as_of: DateTime<Utc>,
    pub action: RecommendationAction,
    pub score: f64,
    pub prediction_confidence: Option<f64>,
    pub execution_confidence: Option<f64>,
    pub recommendation_confidence: f64,
    pub expected_net_gp: Option<i64>,
    pub expected_roi: Option<f64>,
    pub risk_label: Option<String>,
    pub reasons: Vec<String>,
    pub explanation: RecommendationExplanation,
}

#[cfg(test)]
mod tests {
    use super::RecommendationAction;

    #[test]
    fn recommendation_action_serde_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&RecommendationAction::Cashout).unwrap(),
            "\"cashout\""
        );
    }
}
