use std::sync::Arc;

use chrono::{TimeZone, Utc};
use grand_edge_domain::{
    ExecutionEstimate, FeatureVector, Gp, HorizonSecs, Item, LatestPrice, ObservedLiquidityProxy,
    Probability, Quantity, Rate, SignalSide, StrategyId, StrategySignal,
};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError, StrategyRegistry};

pub mod advanced_risk_overlay;
pub mod ar_baseline;
pub mod conformal_interval;
pub mod execution_confidence;
pub mod kalman_fair_value;
pub mod mean_reversion;
pub mod momentum;
pub mod portfolio_optimizer;
pub mod regime_hmm;
pub mod spread_edge;
pub mod volatility_filter;

pub use advanced_risk_overlay::AdvancedRiskOverlayStrategy;
pub use ar_baseline::ArBaselineStrategy;
pub use conformal_interval::ConformalIntervalStrategy;
pub use execution_confidence::{
    ExecutionConfidenceEstimate, ExecutionConfidenceStrategy, estimate_execution_confidence,
};
pub use kalman_fair_value::KalmanFairValueStrategy;
pub use mean_reversion::{MeanReversionConfig, MeanReversionStrategy};
pub use momentum::{MomentumConfig, MomentumStrategy};
pub use portfolio_optimizer::{
    PortfolioCandidate, PortfolioOptimizerStrategy, PortfolioOrderSuggestion, optimize_portfolio,
};
pub use regime_hmm::RegimeHmmStrategy;
pub use spread_edge::{SpreadEdgeConfig, SpreadEdgeStrategy};
pub use volatility_filter::VolatilityFilterStrategy;

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

pub fn register_baseline_strategies(registry: &mut StrategyRegistry) -> Result<(), StrategyError> {
    registry.register(Arc::new(SpreadEdgeStrategy::default()))?;
    registry.register(Arc::new(MomentumStrategy::default()))?;
    registry.register(Arc::new(MeanReversionStrategy::default()))?;
    registry.register(Arc::new(VolatilityFilterStrategy::default()))?;
    registry.register(Arc::new(ExecutionConfidenceStrategy::default()))?;
    registry.register(Arc::new(PortfolioOptimizerStrategy::default()))?;
    registry.register(Arc::new(KalmanFairValueStrategy::default()))?;
    registry.register(Arc::new(ArBaselineStrategy::default()))?;
    registry.register(Arc::new(RegimeHmmStrategy::default()))?;
    registry.register(Arc::new(ConformalIntervalStrategy::default()))?;
    registry.register(Arc::new(AdvancedRiskOverlayStrategy::default()))?;
    Ok(())
}

pub(crate) fn feature_f64(features: &FeatureVector, key: &str) -> Option<f64> {
    features.values.get(key).and_then(|value| value.as_f64())
}

pub(crate) fn feature_i64(features: &FeatureVector, key: &str) -> Option<i64> {
    features.values.get(key).and_then(|value| value.as_i64())
}

pub(crate) fn base_explanation(
    strategy_id: &str,
    strategy_version: &str,
    reason: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let mut explanation = serde_json::Map::new();
    explanation.insert("strategy_id".to_string(), serde_json::json!(strategy_id));
    explanation.insert(
        "model_version".to_string(),
        serde_json::json!(strategy_version),
    );
    explanation.insert("reason".to_string(), serde_json::json!(reason));
    explanation
}

pub(crate) fn observed_liquidity_proxy(
    observed_volume: i64,
    observed_high_side_volume: i64,
    observed_low_side_volume: i64,
    observed_volume_z: Option<f64>,
    observed_volume_reliability: Option<f64>,
    high_low_volume_ratio: Option<f64>,
) -> ObservedLiquidityProxy {
    ObservedLiquidityProxy {
        observed_volume: Quantity::positive(observed_volume.max(1)).unwrap(),
        observed_high_side_volume: Quantity::positive(observed_high_side_volume.max(1)).unwrap(),
        observed_low_side_volume: Quantity::positive(observed_low_side_volume.max(1)).unwrap(),
        observed_volume_z: observed_volume_z.and_then(|value| Rate::new(value).ok()),
        observed_volume_reliability: observed_volume_reliability
            .and_then(|value| Probability::new(value).ok()),
        high_low_volume_ratio: high_low_volume_ratio.and_then(|value| Rate::new(value).ok()),
        note: "Observed volume is a proxy from candle aggregates, not true GE depth.".to_string(),
    }
}

pub(crate) fn execution_estimate_from_proxy(
    observed_liquidity: ObservedLiquidityProxy,
    estimated_fill_probability: f64,
    liquidity_confidence: f64,
    estimated_capacity: i64,
    participation_rate: f64,
    confidence_haircut: f64,
    spread_pct: f64,
    price_staleness_seconds: i64,
    volatility: f64,
) -> ExecutionEstimate {
    ExecutionEstimate {
        observed_liquidity,
        estimated_fill_probability: Probability::new(estimated_fill_probability).ok(),
        liquidity_confidence: Probability::new(liquidity_confidence).ok(),
        estimated_capacity: Quantity::positive(estimated_capacity.max(1)).ok(),
        participation_rate: Probability::new(participation_rate).ok(),
        confidence_haircut: Probability::new(confidence_haircut).ok(),
        spread_pct: Rate::new(spread_pct).ok(),
        price_staleness_seconds: HorizonSecs::positive(price_staleness_seconds.max(1)).ok(),
        volatility: Rate::new(volatility).ok(),
    }
}

pub(crate) fn strategy_signal(
    strategy_id: &'static str,
    strategy_version: &'static str,
    ctx: &StrategyContext,
    item: &Item,
    side: SignalSide,
    confidence: f64,
    expected_return: f64,
    expected_net_gp_per_unit: i64,
    target_entry: Option<i64>,
    target_exit: Option<i64>,
    stop_loss: Option<i64>,
    take_profit: Option<i64>,
    max_quantity: Option<i64>,
    execution_estimate: Option<ExecutionEstimate>,
    explanation: serde_json::Map<String, serde_json::Value>,
) -> Result<StrategySignal, StrategyError> {
    Ok(StrategySignal {
        item_id: item.item_id,
        strategy_id: StrategyId(strategy_id.to_string()),
        model_version: grand_edge_domain::ModelVersion(strategy_version.to_string()),
        as_of: ctx.as_of,
        side,
        horizon_secs: HorizonSecs::positive(3600)
            .map_err(|error| StrategyError::Validation(error.to_string()))?,
        confidence: Probability::new(confidence)
            .map_err(|error| StrategyError::Validation(error.to_string()))?,
        expected_return: Rate::new(expected_return)
            .map_err(|error| StrategyError::Validation(error.to_string()))?,
        expected_net_gp_per_unit: Gp::try_from(expected_net_gp_per_unit)
            .map_err(|error| StrategyError::Validation(error.to_string()))?,
        target_entry: optional_gp(target_entry)?,
        target_exit: optional_gp(target_exit)?,
        stop_loss: optional_gp(stop_loss)?,
        take_profit: optional_gp(take_profit)?,
        max_quantity: max_quantity
            .map(|value| {
                Quantity::positive(value)
                    .map_err(|error| StrategyError::Validation(error.to_string()))
            })
            .transpose()?,
        execution_estimate,
        explanation: serde_json::Value::Object(explanation),
    })
}

fn optional_gp(value: Option<i64>) -> Result<Option<Gp>, StrategyError> {
    value
        .map(|value| {
            Gp::try_from(value).map_err(|error| StrategyError::Validation(error.to_string()))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{Duration, TimeZone, Utc};
    use grand_edge_domain::{FeatureVector, Gp, Item, ItemId, LatestPrice, SignalSide};

    use super::{
        ExecutionConfidenceStrategy, FailingTestStrategy, MeanReversionStrategy, MomentumStrategy,
        NoopTestStrategy, PortfolioCandidate, SpreadEdgeStrategy, Strategy,
        VolatilityFilterStrategy, estimate_execution_confidence, optimize_portfolio,
        register_baseline_strategies, test_context,
    };
    use crate::StrategyRegistry;

    fn base_item() -> Item {
        Item {
            item_id: ItemId(4151),
            name: "Abyssal whip".to_string(),
            examine: None,
            members: true,
            buy_limit: Some(70),
            low_alch: Some(Gp(48_000)),
            high_alch: Some(Gp(72_000)),
            value: Some(Gp(120_001)),
            icon: None,
            updated_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    fn latest(high: Option<i64>, low: Option<i64>) -> LatestPrice {
        LatestPrice {
            item_id: ItemId(4151),
            high: high.map(Gp),
            high_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 59, 0).unwrap()),
            low: low.map(Gp),
            low_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 58, 0).unwrap()),
            observed_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    fn features(values: &[(&str, serde_json::Value)]) -> FeatureVector {
        let mut map = serde_json::Map::new();
        for (key, value) in values {
            map.insert((*key).to_string(), value.clone());
        }
        FeatureVector {
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values: map,
        }
    }

    #[test]
    fn register_baseline_strategies_registers_all_eleven() {
        let mut registry = StrategyRegistry::new();
        register_baseline_strategies(&mut registry).unwrap();
        for strategy_id in [
            "spread_edge_v1",
            "momentum_v1",
            "mean_reversion_v1",
            "volatility_filter_v1",
            "execution_confidence_v1",
            "portfolio_optimizer_v1",
            "kalman_fair_value_v1",
            "arima_baseline_v1",
            "regime_hmm_v1",
            "conformal_interval_v1",
            "advanced_risk_overlay_v1",
        ] {
            assert!(
                registry.get(strategy_id).is_some(),
                "{strategy_id} was missing"
            );
        }
    }

    #[test]
    fn spread_edge_rejects_when_tax_kills_edge() {
        let strategy = SpreadEdgeStrategy::default();
        let signal = strategy
            .generate(
                &test_context(),
                &base_item(),
                &latest(Some(103_000), Some(100_000)),
                &features(&[
                    ("observed_volume_1h", serde_json::json!(400)),
                    ("spread_pct", serde_json::json!(0.03)),
                    ("ewma_volatility_24h", serde_json::json!(0.01)),
                    ("price_staleness_secs", serde_json::json!(60)),
                    ("observed_high_side_volume_1h", serde_json::json!(250)),
                    ("observed_low_side_volume_1h", serde_json::json!(170)),
                ]),
            )
            .unwrap();

        assert_ne!(signal.side, SignalSide::Buy);
        assert_eq!(signal.expected_net_gp_per_unit, Gp(940));
    }

    #[test]
    fn spread_edge_buys_when_net_roi_exceeds_threshold() {
        let strategy = SpreadEdgeStrategy::default();
        let signal = strategy
            .generate(
                &test_context(),
                &base_item(),
                &latest(Some(105_000), Some(100_000)),
                &features(&[
                    ("observed_volume_1h", serde_json::json!(600)),
                    ("spread_pct", serde_json::json!(0.048)),
                    ("ewma_volatility_24h", serde_json::json!(0.01)),
                    ("price_staleness_secs", serde_json::json!(60)),
                    ("observed_high_side_volume_1h", serde_json::json!(350)),
                    ("observed_low_side_volume_1h", serde_json::json!(250)),
                ]),
            )
            .unwrap();

        assert_eq!(signal.side, SignalSide::Buy);
    }

    #[test]
    fn momentum_scores_positive_return_and_observed_volume_proxy() {
        let strategy = MomentumStrategy::default();
        let signal = strategy
            .generate(
                &test_context(),
                &base_item(),
                &latest(Some(100), Some(95)),
                &features(&[
                    ("return_1h", serde_json::json!(0.04)),
                    ("return_6h", serde_json::json!(0.06)),
                    ("observed_volume_z_24h", serde_json::json!(1.5)),
                    ("spread_pct", serde_json::json!(0.01)),
                    ("ewma_volatility_24h", serde_json::json!(0.01)),
                ]),
            )
            .unwrap();

        assert_eq!(signal.side, SignalSide::Buy);
    }

    #[test]
    fn mean_reversion_buys_negative_z_score() {
        let strategy = MeanReversionStrategy::default();
        let signal = strategy
            .generate(
                &test_context(),
                &base_item(),
                &latest(Some(100), Some(80)),
                &features(&[
                    ("z_score_24h", serde_json::json!(-2.0)),
                    ("observed_volume_z_24h", serde_json::json!(0.2)),
                ]),
            )
            .unwrap();

        assert_eq!(signal.side, SignalSide::Buy);
    }

    #[test]
    fn execution_confidence_capacity_uses_observed_volume_haircut() {
        let estimate = estimate_execution_confidence(Some(70), 400, 0.05, 0.5, 0.0, 0.01, 60, 0.02);
        assert_eq!(estimate.estimated_capacity, 10);
    }

    #[test]
    fn strategy_explanation_does_not_claim_true_liquidity() {
        let signal = ExecutionConfidenceStrategy::default()
            .generate(
                &test_context(),
                &base_item(),
                &latest(Some(100), Some(80)),
                &features(&[
                    ("observed_volume_1h", serde_json::json!(400)),
                    ("observed_volume_z_24h", serde_json::json!(1.0)),
                    ("spread_pct", serde_json::json!(0.02)),
                    ("price_staleness_secs", serde_json::json!(60)),
                    ("ewma_volatility_24h", serde_json::json!(0.01)),
                    ("observed_high_side_volume_1h", serde_json::json!(250)),
                    ("observed_low_side_volume_1h", serde_json::json!(170)),
                    ("observed_volume_reliability_24h", serde_json::json!(0.8)),
                    ("high_low_volume_ratio_1h", serde_json::json!(1.47)),
                ]),
            )
            .unwrap();
        let explanation = serde_json::to_string(&signal.explanation).unwrap();
        assert!(!explanation.contains("true liquidity"));
        assert!(explanation.contains("estimated fill probability"));
    }

    #[test]
    fn portfolio_optimizer_respects_capital_and_slots() {
        let suggestions = optimize_portfolio(
            500,
            1,
            &[
                PortfolioCandidate {
                    item_id: 1,
                    entry_price: 100,
                    expected_net_gp_per_unit: 30,
                    max_quantity: 2,
                    risk_score: 0.2,
                },
                PortfolioCandidate {
                    item_id: 2,
                    entry_price: 200,
                    expected_net_gp_per_unit: 20,
                    max_quantity: 2,
                    risk_score: 0.1,
                },
            ],
        );
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].item_id, 1);
    }

    #[test]
    fn baseline_registry_still_allows_existing_test_strategies() {
        let mut registry = StrategyRegistry::new();
        registry.register(Arc::new(NoopTestStrategy)).unwrap();
        registry.register(Arc::new(FailingTestStrategy)).unwrap();
        assert!(registry.get("noop").is_some());
        assert!(registry.get("fail").is_some());
        let _ = VolatilityFilterStrategy::default();
        let _ = Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap() - Duration::minutes(1);
    }
}
