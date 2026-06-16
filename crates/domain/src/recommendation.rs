use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    Gp, ItemId, ModelAccuracySnapshot, Probability, Rate, RecommendationId, StrategySignal, UserId,
};

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
    pub value: Rate,
    pub weight: Option<Rate>,
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
    pub recommendation_id: RecommendationId,
    pub user_id: Option<UserId>,
    pub item_id: ItemId,
    pub as_of: DateTime<Utc>,
    pub action: RecommendationAction,
    pub score: Rate,
    pub prediction_confidence: Option<Probability>,
    pub execution_confidence: Option<Probability>,
    pub recommendation_confidence: Probability,
    pub expected_net_gp: Option<Gp>,
    pub expected_roi: Option<Rate>,
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
