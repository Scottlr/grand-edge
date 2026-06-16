use grand_edge_domain::MarketRules;
use serde::{Deserialize, Serialize};

use crate::graph_actions::GraphActionConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationConfig {
    pub min_buy_score: f64,
    pub min_watch_score: f64,
    pub min_confidence: f64,
    pub min_execution_confidence: f64,
    pub min_expected_roi: f64,
    pub lambda_volatility: f64,
    pub lambda_spread: f64,
    pub lambda_staleness: f64,
    pub lambda_liquidity: f64,
    pub confidence_bonus_weight: f64,
    pub user_fit_bonus_weight: f64,
    pub execution_confidence_weight: f64,
    pub feature_set_version: String,
    pub market_rules: MarketRules,
    pub graph_actions: GraphActionConfig,
}

impl Default for RecommendationConfig {
    fn default() -> Self {
        Self {
            min_buy_score: 0.35,
            min_watch_score: 0.05,
            min_confidence: 0.55,
            min_execution_confidence: 0.45,
            min_expected_roi: 0.01,
            lambda_volatility: 0.35,
            lambda_spread: 0.25,
            lambda_staleness: 0.15,
            lambda_liquidity: 0.25,
            confidence_bonus_weight: 0.20,
            user_fit_bonus_weight: 0.10,
            execution_confidence_weight: 0.20,
            feature_set_version: "features_v1".to_string(),
            market_rules: MarketRules::default(),
            graph_actions: GraphActionConfig::default(),
        }
    }
}
