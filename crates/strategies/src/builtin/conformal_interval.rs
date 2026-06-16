use grand_edge_domain::{FeatureVector, Item, LatestPrice, SignalSide};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError, conformal_interval};

use super::{base_explanation, feature_f64, strategy_signal};

const STRATEGY_ID: &str = "conformal_interval_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone)]
pub struct ConformalIntervalStrategy {
    pub coverage: f64,
    pub residual_floor: f64,
}

impl Default for ConformalIntervalStrategy {
    fn default() -> Self {
        Self {
            coverage: 0.90,
            residual_floor: 0.018,
        }
    }
}

impl Strategy for ConformalIntervalStrategy {
    fn id(&self) -> &'static str {
        STRATEGY_ID
    }

    fn version(&self) -> &'static str {
        STRATEGY_VERSION
    }

    fn required_lookback(&self) -> LookbackSpec {
        LookbackSpec {
            min_5m_buckets: 1,
            min_1h_buckets: 6,
        }
    }

    fn generate(
        &self,
        ctx: &StrategyContext,
        item: &Item,
        _latest: &LatestPrice,
        features: &FeatureVector,
    ) -> Result<grand_edge_domain::StrategySignal, StrategyError> {
        let predicted_return = feature_f64(features, "return_1h").unwrap_or(0.0);
        let residual_quantile = feature_f64(features, "ewma_volatility_24h")
            .unwrap_or(self.residual_floor)
            .max(self.residual_floor);
        let interval = conformal_interval(predicted_return, residual_quantile, self.coverage);
        let mut explanation = base_explanation(self.id(), self.version(), "conformal_interval");
        explanation.insert(
            "conformal_interval".to_string(),
            serde_json::to_value(&interval).unwrap(),
        );

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            SignalSide::Watch,
            0.5,
            predicted_return,
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
