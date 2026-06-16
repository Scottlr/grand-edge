use chrono::{TimeZone, Utc};
use grand_edge_domain::{
    FeatureVector, Gp, HorizonSecs, Item, LatestPrice, Probability, Rate, SignalSide, StrategyId,
    StrategySignal,
};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError};

pub struct NoopTestStrategy;
pub struct FailingTestStrategy;

impl Strategy for NoopTestStrategy {
    fn id(&self) -> &'static str {
        "noop"
    }

    fn version(&self) -> &'static str {
        "v1"
    }

    fn required_lookback(&self) -> LookbackSpec {
        LookbackSpec {
            min_5m_buckets: 1,
            min_1h_buckets: 1,
        }
    }

    fn generate(
        &self,
        ctx: &StrategyContext,
        item: &Item,
        _latest: &LatestPrice,
        _features: &FeatureVector,
    ) -> Result<StrategySignal, StrategyError> {
        Ok(StrategySignal {
            item_id: item.item_id,
            strategy_id: StrategyId(self.id().to_string()),
            model_version: grand_edge_domain::ModelVersion(self.version().to_string()),
            as_of: ctx.as_of,
            side: SignalSide::Hold,
            horizon_secs: HorizonSecs(3600),
            confidence: Probability::new(0.6).unwrap(),
            expected_return: Rate::new(0.02).unwrap(),
            expected_net_gp_per_unit: Gp(100),
            target_entry: Some(Gp(100)),
            target_exit: Some(Gp(105)),
            stop_loss: Some(Gp(95)),
            take_profit: Some(Gp(110)),
            max_quantity: None,
            execution_estimate: None,
            explanation: serde_json::json!({
                "strategy": self.id(),
                "as_of": ctx.as_of.to_rfc3339(),
            }),
        })
    }
}

impl Strategy for FailingTestStrategy {
    fn id(&self) -> &'static str {
        "fail"
    }

    fn version(&self) -> &'static str {
        "v1"
    }

    fn required_lookback(&self) -> LookbackSpec {
        LookbackSpec {
            min_5m_buckets: 0,
            min_1h_buckets: 0,
        }
    }

    fn generate(
        &self,
        _ctx: &StrategyContext,
        _item: &Item,
        _latest: &LatestPrice,
        _features: &FeatureVector,
    ) -> Result<StrategySignal, StrategyError> {
        Err(StrategyError::Validation(
            "simulated strategy failure".to_string(),
        ))
    }
}

pub fn test_context() -> StrategyContext {
    StrategyContext {
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        market_rules: grand_edge_domain::MarketRules::default(),
        risk: crate::RiskConfig::default(),
        recent_metrics: std::collections::HashMap::new(),
    }
}
