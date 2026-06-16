use grand_edge_domain::{ExecutionMode, MarketRules};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorConfig {
    pub execution_mode: ExecutionMode,
    pub market_rules: MarketRules,
    pub participation_rate: f64,
    pub confidence_haircut: f64,
    pub default_horizon_secs: i64,
    pub emergency_exit_slippage_gp: i64,
    pub worst_case_slippage_gp: i64,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            execution_mode: ExecutionMode::ConservativeInstant,
            market_rules: MarketRules::default(),
            participation_rate: 0.05,
            confidence_haircut: 0.5,
            default_horizon_secs: 21_600,
            emergency_exit_slippage_gp: 0,
            worst_case_slippage_gp: 0,
        }
    }
}
