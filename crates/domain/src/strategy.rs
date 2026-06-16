use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Gp, HorizonSecs, ItemId, ModelVersion, Probability, Quantity, Rate, StrategyId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SignalSide {
    Buy,
    Sell,
    Hold,
    Avoid,
    Cashout,
    Watch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ObservedLiquidityProxy {
    pub observed_volume: Quantity,
    pub observed_high_side_volume: Quantity,
    pub observed_low_side_volume: Quantity,
    pub observed_volume_z: Option<Rate>,
    pub observed_volume_reliability: Option<Probability>,
    pub high_low_volume_ratio: Option<Rate>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionEstimate {
    pub observed_liquidity: ObservedLiquidityProxy,
    pub estimated_fill_probability: Option<Probability>,
    pub liquidity_confidence: Option<Probability>,
    pub estimated_capacity: Option<Quantity>,
    pub participation_rate: Option<Probability>,
    pub confidence_haircut: Option<Probability>,
    pub spread_pct: Option<Rate>,
    pub price_staleness_seconds: Option<HorizonSecs>,
    pub volatility: Option<Rate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StrategySignal {
    pub item_id: ItemId,
    pub strategy_id: StrategyId,
    pub model_version: ModelVersion,
    pub as_of: DateTime<Utc>,
    pub side: SignalSide,
    pub horizon_secs: HorizonSecs,
    pub confidence: Probability,
    pub expected_return: Rate,
    pub expected_net_gp_per_unit: Gp,
    pub target_entry: Option<Gp>,
    pub target_exit: Option<Gp>,
    pub stop_loss: Option<Gp>,
    pub take_profit: Option<Gp>,
    pub max_quantity: Option<Quantity>,
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
                observed_volume: crate::Quantity(10),
                observed_high_side_volume: crate::Quantity(4),
                observed_low_side_volume: crate::Quantity(6),
                observed_volume_z: Some(crate::Rate::new(1.2).unwrap()),
                observed_volume_reliability: Some(crate::Probability::new(0.8).unwrap()),
                high_low_volume_ratio: Some(crate::Rate::new(0.66).unwrap()),
                note: "Observed volume is elevated, but not true GE depth.".to_string(),
            },
            estimated_fill_probability: Some(crate::Probability::new(0.4).unwrap()),
            liquidity_confidence: Some(crate::Probability::new(0.7).unwrap()),
            estimated_capacity: Some(crate::Quantity(50)),
            participation_rate: Some(crate::Probability::new(0.1).unwrap()),
            confidence_haircut: Some(crate::Probability::new(0.3).unwrap()),
            spread_pct: Some(crate::Rate::new(0.02).unwrap()),
            price_staleness_seconds: Some(crate::HorizonSecs(300)),
            volatility: Some(crate::Rate::new(0.15).unwrap()),
        };

        let serialized = serde_json::to_string(&estimate).unwrap();
        assert!(serialized.contains("observed_liquidity"));
        assert!(serialized.contains("estimated_fill_probability"));
        assert!(!serialized.contains("trueLiquidity"));
        assert!(!serialized.contains("marketDepth"));
        assert!(!serialized.contains("\"fill_probability\""));
    }
}
