use grand_edge_domain::{FeatureVector, Item, LatestPrice, SignalSide};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError};

use super::{base_explanation, feature_f64, strategy_signal};

const STRATEGY_ID: &str = "mean_reversion_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeanReversionConfig {
    pub z_entry: f64,
    pub z_exit: f64,
    pub min_observed_volume_z: f64,
}

impl Default for MeanReversionConfig {
    fn default() -> Self {
        Self {
            z_entry: -1.5,
            z_exit: 1.0,
            min_observed_volume_z: -2.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MeanReversionStrategy {
    config: MeanReversionConfig,
}

impl Strategy for MeanReversionStrategy {
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
        let z_score = feature_f64(features, "z_score_24h").unwrap_or(0.0);
        let observed_volume_z = feature_f64(features, "observed_volume_z_24h").unwrap_or(0.0);
        let side = if z_score <= self.config.z_entry
            && observed_volume_z >= self.config.min_observed_volume_z
        {
            SignalSide::Buy
        } else if z_score >= self.config.z_exit {
            SignalSide::Cashout
        } else {
            SignalSide::Watch
        };

        let mut explanation = base_explanation(self.id(), self.version(), "mean_reversion_z_score");
        explanation.insert("z_score_24h".to_string(), serde_json::json!(z_score));
        explanation.insert(
            "observed_volume_z".to_string(),
            serde_json::json!(observed_volume_z),
        );

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            side,
            0.6,
            (-z_score).max(0.0) * 0.01,
            ((-z_score).max(0.0) * 500.0) as i64,
            None,
            None,
            None,
            None,
            item.buy_limit.map(i64::from),
            None,
            explanation,
        )
    }
}
