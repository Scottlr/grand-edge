use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Gp, GraphRecommendationContext, ItemId, ModelAccuracySnapshot, Probability, Rate,
    RecommendationId, StrategySignal, StructuredRecommendationExplanation, UserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationAction {
    Buy,
    Add,
    Hold,
    Cashout,
    Avoid,
    Watch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScoreComponent {
    pub key: String,
    pub label: String,
    pub value: Rate,
    pub weight: Option<Rate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RecommendationExplanation {
    pub feature_set_version: String,
    pub market_rules_version: String,
    #[serde(default)]
    pub graph_version: Option<String>,
    #[serde(default)]
    pub graph_context: Option<GraphRecommendationContext>,
    pub strategy_votes: Vec<StrategySignal>,
    pub score_components: Vec<ScoreComponent>,
    pub accuracy_snapshot: Option<ModelAccuracySnapshot>,
    #[serde(default)]
    pub structured_explanation: StructuredRecommendationExplanation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
    use super::{RecommendationAction, RecommendationExplanation};

    #[test]
    fn recommendation_action_serde_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&RecommendationAction::Cashout).unwrap(),
            "\"cashout\""
        );
    }

    #[test]
    fn recommendation_graph_context_defaults_to_none() {
        let payload = serde_json::json!({
            "feature_set_version": "features_v1",
            "market_rules_version": "rules_v1",
            "strategy_votes": [],
            "score_components": [],
            "accuracy_snapshot": null,
            "structured_explanation": {
                "summary": "",
                "reason_atoms": [],
                "invalidation_rules": [],
                "confidence": {
                    "prediction_confidence": 0.0,
                    "recommendation_confidence": 0.0,
                    "data_quality_confidence": 0.0,
                    "model_calibration_confidence": 0.0,
                    "liquidity_confidence": 0.0,
                    "explanation_confidence": 0.0
                }
            }
        });

        let explanation: RecommendationExplanation = serde_json::from_value(payload).unwrap();
        assert_eq!(explanation.graph_version, None);
        assert_eq!(explanation.graph_context, None);
        assert_eq!(explanation.structured_explanation.graph_context, None);
    }
}
