use serde::{Deserialize, Serialize};

use crate::math::{ArBaselineConfig, KalmanConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub enabled_strategies: Vec<String>,
    pub risk: RiskConfig,
    pub kalman_fair_value: KalmanConfig,
    pub ar_baseline: ArBaselineConfig,
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
            enabled_strategies: default_enabled_strategy_ids(),
            risk: RiskConfig::default(),
            kalman_fair_value: KalmanConfig::default(),
            ar_baseline: ArBaselineConfig::default(),
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

impl Default for KalmanConfig {
    fn default() -> Self {
        Self {
            process_variance: 0.0001,
            observation_variance: 0.0025,
            buy_mispricing_threshold: 0.015,
            cashout_mispricing_threshold: -0.01,
        }
    }
}

impl Default for ArBaselineConfig {
    fn default() -> Self {
        Self {
            intercept: 0.0,
            phi: 0.35,
            min_expected_return: 0.01,
            confidence_floor: 0.35,
        }
    }
}

pub fn default_enabled_strategy_ids() -> Vec<String> {
    [
        "spread_edge_v1",
        "momentum_v1",
        "mean_reversion_v1",
        "volatility_filter_v1",
        "execution_confidence_v1",
        "portfolio_optimizer_v1",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
