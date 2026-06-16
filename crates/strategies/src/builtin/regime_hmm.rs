use grand_edge_domain::{FeatureVector, Item, LatestPrice, SignalSide};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError, classify_regime};

use super::{base_explanation, strategy_signal};

const STRATEGY_ID: &str = "regime_hmm_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone, Default)]
pub struct RegimeHmmStrategy;

impl Strategy for RegimeHmmStrategy {
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
        let estimate = classify_regime(features);
        let mut explanation = base_explanation(self.id(), self.version(), "market_regime");
        explanation.insert("method".to_string(), serde_json::json!("heuristic_v1"));
        explanation.insert(
            "regime".to_string(),
            serde_json::to_value(&estimate).unwrap(),
        );

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            SignalSide::Watch,
            estimate.probability,
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
