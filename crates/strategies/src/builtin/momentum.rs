use grand_edge_domain::{FeatureVector, Item, LatestPrice, SignalSide};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError};

use super::{base_explanation, feature_f64, strategy_signal};

const STRATEGY_ID: &str = "momentum_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MomentumConfig {
    pub threshold: f64,
    pub exit_threshold: f64,
    pub w_return_1h: f64,
    pub w_return_6h: f64,
    pub w_observed_volume_z: f64,
    pub w_spread_pct: f64,
    pub w_volatility: f64,
}

impl Default for MomentumConfig {
    fn default() -> Self {
        Self {
            threshold: 0.02,
            exit_threshold: -0.01,
            w_return_1h: 1.2,
            w_return_6h: 0.8,
            w_observed_volume_z: 0.4,
            w_spread_pct: 1.0,
            w_volatility: 0.7,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MomentumStrategy {
    config: MomentumConfig,
}

impl Strategy for MomentumStrategy {
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
        let return_1h = feature_f64(features, "return_1h").unwrap_or(0.0);
        let return_6h = feature_f64(features, "return_6h").unwrap_or(0.0);
        let observed_volume_z = feature_f64(features, "observed_volume_z_24h").unwrap_or(0.0);
        let spread_pct = feature_f64(features, "spread_pct").unwrap_or(0.0);
        let volatility = feature_f64(features, "ewma_volatility_24h").unwrap_or(0.0);
        let score = self.config.w_return_1h * return_1h
            + self.config.w_return_6h * return_6h
            + self.config.w_observed_volume_z * observed_volume_z
            - self.config.w_spread_pct * spread_pct
            - self.config.w_volatility * volatility;

        let side = if score >= self.config.threshold {
            SignalSide::Buy
        } else if score <= self.config.exit_threshold {
            SignalSide::Cashout
        } else {
            SignalSide::Watch
        };

        let mut explanation = base_explanation(self.id(), self.version(), "momentum_score");
        explanation.insert("score".to_string(), serde_json::json!(score));
        explanation.insert(
            "observed_volume_note".to_string(),
            serde_json::json!("observed volume confirms interest but is not exact liquidity"),
        );

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            side,
            0.55 + score.clamp(-0.2, 0.2).abs(),
            score,
            (score.max(0.0) * 1_000.0) as i64,
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
