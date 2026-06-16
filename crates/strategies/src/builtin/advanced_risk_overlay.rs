use grand_edge_domain::{FeatureVector, Item, LatestPrice, SignalSide};

use crate::{
    LookbackSpec, Strategy, StrategyContext, StrategyError, advanced_risk_overlay, classify_regime,
};

use super::{base_explanation, strategy_signal};

const STRATEGY_ID: &str = "advanced_risk_overlay_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone, Default)]
pub struct AdvancedRiskOverlayStrategy;

impl Strategy for AdvancedRiskOverlayStrategy {
    fn id(&self) -> &'static str {
        STRATEGY_ID
    }

    fn version(&self) -> &'static str {
        STRATEGY_VERSION
    }

    fn required_lookback(&self) -> LookbackSpec {
        LookbackSpec {
            min_5m_buckets: 1,
            min_1h_buckets: 24,
        }
    }

    fn generate(
        &self,
        ctx: &StrategyContext,
        item: &Item,
        _latest: &LatestPrice,
        features: &FeatureVector,
    ) -> Result<grand_edge_domain::StrategySignal, StrategyError> {
        let regime = classify_regime(features);
        let overlay = advanced_risk_overlay(features, &regime, &ctx.risk);
        let mut explanation = base_explanation(self.id(), self.version(), "advanced_risk_overlay");
        explanation.insert("regime".to_string(), serde_json::to_value(&regime).unwrap());
        explanation.insert(
            "risk_overlay".to_string(),
            serde_json::to_value(&overlay).unwrap(),
        );

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            if overlay.final_multiplier < 0.40 {
                SignalSide::Avoid
            } else {
                SignalSide::Watch
            },
            overlay.final_multiplier.max(0.05),
            0.0,
            0,
            None,
            None,
            None,
            None,
            None,
            None,
            explanation,
        )
    }
}
