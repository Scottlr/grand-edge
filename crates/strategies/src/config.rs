use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub enabled_strategies: Vec<String>,
    pub risk: RiskConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_gp_per_item: i64,
    pub max_portfolio_drawdown: f64,
    pub min_expected_roi: f64,
    pub min_confidence: f64,
    pub participation_rate: f64,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            enabled_strategies: Vec::new(),
            risk: RiskConfig::default(),
        }
    }
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_gp_per_item: 5_000_000,
            max_portfolio_drawdown: 0.15,
            min_expected_roi: 0.01,
            min_confidence: 0.55,
            participation_rate: 0.10,
        }
    }
}
