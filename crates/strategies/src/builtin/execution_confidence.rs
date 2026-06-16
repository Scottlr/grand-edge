use grand_edge_domain::{FeatureVector, Item, LatestPrice, SignalSide};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError};

use super::{
    base_explanation, execution_estimate_from_proxy, feature_f64, feature_i64,
    observed_liquidity_proxy, strategy_signal,
};

const STRATEGY_ID: &str = "execution_confidence_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionConfidenceEstimate {
    pub observed_volume: i64,
    pub observed_volume_z: Option<f64>,
    pub estimated_capacity: i64,
    pub estimated_fill_probability: f64,
    pub liquidity_confidence: f64,
    pub participation_rate: f64,
    pub confidence_haircut: f64,
    pub quantity_hint: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ExecutionConfidenceStrategy;

impl Strategy for ExecutionConfidenceStrategy {
    fn id(&self) -> &'static str {
        STRATEGY_ID
    }

    fn version(&self) -> &'static str {
        STRATEGY_VERSION
    }

    fn required_lookback(&self) -> LookbackSpec {
        LookbackSpec {
            min_5m_buckets: 0,
            min_1h_buckets: 1,
        }
    }

    fn generate(
        &self,
        ctx: &StrategyContext,
        item: &Item,
        _latest: &LatestPrice,
        features: &FeatureVector,
    ) -> Result<grand_edge_domain::StrategySignal, StrategyError> {
        let observed_volume = feature_i64(features, "observed_volume_1h").unwrap_or(0);
        let observed_volume_z = feature_f64(features, "observed_volume_z_24h").unwrap_or(0.0);
        let spread_pct = feature_f64(features, "spread_pct").unwrap_or(0.0);
        let volatility = feature_f64(features, "ewma_volatility_24h").unwrap_or(0.0);
        let price_age_seconds = feature_i64(features, "price_staleness_secs").unwrap_or(0);
        let estimate = estimate_execution_confidence(
            item.buy_limit,
            observed_volume,
            ctx.risk.participation_rate,
            0.5,
            observed_volume_z,
            spread_pct,
            price_age_seconds,
            volatility,
        );

        let mut explanation =
            base_explanation(self.id(), self.version(), "execution_confidence_proxy");
        explanation.insert(
            "estimated_capacity".to_string(),
            serde_json::json!(estimate.estimated_capacity),
        );
        explanation.insert(
            "estimated_fill_probability".to_string(),
            serde_json::json!(estimate.estimated_fill_probability),
        );
        explanation.insert(
            "liquidity_note".to_string(),
            serde_json::json!(
                "estimated fill probability is a bounded proxy, not exact fill probability"
            ),
        );

        let execution_estimate = execution_estimate_from_proxy(
            observed_liquidity_proxy(
                estimate.observed_volume,
                feature_i64(features, "observed_high_side_volume_1h").unwrap_or(1),
                feature_i64(features, "observed_low_side_volume_1h").unwrap_or(1),
                estimate.observed_volume_z,
                feature_f64(features, "observed_volume_reliability_24h"),
                feature_f64(features, "high_low_volume_ratio_1h"),
            ),
            estimate.estimated_fill_probability,
            estimate.liquidity_confidence,
            estimate.estimated_capacity,
            estimate.participation_rate,
            estimate.confidence_haircut,
            spread_pct,
            price_age_seconds.max(1),
            volatility,
        );

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            if estimate.estimated_fill_probability >= ctx.risk.min_confidence {
                SignalSide::Watch
            } else {
                SignalSide::Avoid
            },
            estimate.liquidity_confidence,
            0.0,
            0,
            None,
            None,
            None,
            None,
            Some(estimate.estimated_capacity),
            Some(execution_estimate),
            explanation,
        )
    }
}

pub fn estimate_execution_confidence(
    buy_limit: Option<i32>,
    observed_volume: i64,
    participation_rate: f64,
    confidence_haircut: f64,
    observed_volume_z: f64,
    spread_pct: f64,
    price_age_seconds: i64,
    volatility: f64,
) -> ExecutionConfidenceEstimate {
    let capacity_from_volume = ((observed_volume.max(0) as f64)
        * participation_rate.max(0.0)
        * confidence_haircut.max(0.0))
    .floor() as i64;
    let estimated_capacity = buy_limit
        .map(i64::from)
        .map(|buy_limit| capacity_from_volume.min(buy_limit))
        .unwrap_or(capacity_from_volume)
        .max(0);
    let price_age_minutes = price_age_seconds.max(0) as f64 / 60.0;
    let raw_score = -0.2 + 0.7 * observed_volume_z
        - 15.0 * spread_pct
        - 0.03 * price_age_minutes
        - 6.0 * volatility;
    let estimated_fill_probability = sigmoid(raw_score);
    let liquidity_confidence = sigmoid(raw_score + 0.25);

    ExecutionConfidenceEstimate {
        observed_volume,
        observed_volume_z: Some(observed_volume_z),
        estimated_capacity,
        estimated_fill_probability,
        liquidity_confidence,
        participation_rate,
        confidence_haircut,
        quantity_hint: estimated_capacity,
    }
}

fn sigmoid(value: f64) -> f64 {
    1.0 / (1.0 + (-value).exp())
}
