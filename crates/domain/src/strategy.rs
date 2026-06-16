use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalSide {
    Buy,
    Sell,
    Hold,
    Avoid,
    Cashout,
    Watch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedLiquidityProxy {
    pub observed_volume: i64,
    pub observed_high_side_volume: i64,
    pub observed_low_side_volume: i64,
    pub observed_volume_z: Option<f64>,
    pub observed_volume_reliability: Option<f64>,
    pub high_low_volume_ratio: Option<f64>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionEstimate {
    pub observed_liquidity: ObservedLiquidityProxy,
    pub estimated_fill_probability: Option<f64>,
    pub liquidity_confidence: Option<f64>,
    pub estimated_capacity: Option<i64>,
    pub participation_rate: Option<f64>,
    pub confidence_haircut: Option<f64>,
    pub spread_pct: Option<f64>,
    pub price_staleness_seconds: Option<i64>,
    pub volatility: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategySignal {
    pub item_id: i64,
    pub strategy_id: String,
    pub model_version: String,
    pub as_of: DateTime<Utc>,
    pub side: SignalSide,
    pub horizon_secs: i64,
    pub confidence: f64,
    pub expected_return: f64,
    pub expected_net_gp_per_unit: i64,
    pub target_entry: Option<i64>,
    pub target_exit: Option<i64>,
    pub stop_loss: Option<i64>,
    pub take_profit: Option<i64>,
    pub max_quantity: Option<i64>,
    #[serde(default)]
    pub execution_estimate: Option<ExecutionEstimate>,
    pub explanation: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::{ExecutionEstimate, ObservedLiquidityProxy, SignalSide};

    #[test]
    fn signal_side_serde_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&SignalSide::Cashout).unwrap(),
            "\"cashout\""
        );
    }

    #[test]
    fn execution_estimate_serde_uses_estimated_field_names() {
        let estimate = ExecutionEstimate {
            observed_liquidity: ObservedLiquidityProxy {
                observed_volume: 10,
                observed_high_side_volume: 4,
                observed_low_side_volume: 6,
                observed_volume_z: Some(1.2),
                observed_volume_reliability: Some(0.8),
                high_low_volume_ratio: Some(0.66),
                note: "Observed volume is elevated, but not true GE depth.".to_string(),
            },
            estimated_fill_probability: Some(0.4),
            liquidity_confidence: Some(0.7),
            estimated_capacity: Some(50),
            participation_rate: Some(0.1),
            confidence_haircut: Some(0.3),
            spread_pct: Some(0.02),
            price_staleness_seconds: Some(300),
            volatility: Some(0.15),
        };

        let serialized = serde_json::to_string(&estimate).unwrap();
        assert!(serialized.contains("observed_liquidity"));
        assert!(serialized.contains("estimated_fill_probability"));
        assert!(!serialized.contains("trueLiquidity"));
        assert!(!serialized.contains("marketDepth"));
        assert!(!serialized.contains("\"fill_probability\""));
    }
}
