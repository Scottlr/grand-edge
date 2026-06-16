use grand_edge_domain::{FeatureVector, Item, LatestPrice, SignalSide};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError};

use super::{base_explanation, feature_f64, strategy_signal};

const STRATEGY_ID: &str = "volatility_filter_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone, Default)]
pub struct VolatilityFilterStrategy;

impl Strategy for VolatilityFilterStrategy {
    fn id(&self) -> &'static str {
        STRATEGY_ID
    }

    fn version(&self) -> &'static str {
        STRATEGY_VERSION
    }

    fn required_lookback(&self) -> LookbackSpec {
        LookbackSpec {
            min_5m_buckets: 0,
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
        let volatility = feature_f64(features, "ewma_volatility_24h").unwrap_or(0.0);
        let side = if volatility > 0.08 {
            SignalSide::Avoid
        } else {
            SignalSide::Watch
        };
        let mut explanation = base_explanation(self.id(), self.version(), "volatility_filter");
        explanation.insert(
            "ewma_volatility_24h".to_string(),
            serde_json::json!(volatility),
        );

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            side,
            0.65,
            -volatility,
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
