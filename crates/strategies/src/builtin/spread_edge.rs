use grand_edge_domain::{FeatureVector, Gp, Item, LatestPrice, SignalSide};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError};

use super::{
    base_explanation, execution_estimate_from_proxy, feature_f64, feature_i64,
    observed_liquidity_proxy, strategy_signal,
};

const STRATEGY_ID: &str = "spread_edge_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpreadEdgeConfig {
    pub min_roi: f64,
    pub min_spread_pct: f64,
    pub min_observed_volume: i64,
    pub required_margin_gp: i64,
    pub buy_buffer_gp: i64,
    pub sell_buffer_gp: i64,
    pub slippage_buffer_gp: i64,
    pub liquidity_uncertainty_buffer_gp: i64,
}

impl Default for SpreadEdgeConfig {
    fn default() -> Self {
        Self {
            min_roi: 0.015,
            min_spread_pct: 0.01,
            min_observed_volume: 100,
            required_margin_gp: 1,
            buy_buffer_gp: 0,
            sell_buffer_gp: 0,
            slippage_buffer_gp: 0,
            liquidity_uncertainty_buffer_gp: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SpreadEdgeStrategy {
    config: SpreadEdgeConfig,
}

impl Strategy for SpreadEdgeStrategy {
    fn id(&self) -> &'static str {
        STRATEGY_ID
    }

    fn version(&self) -> &'static str {
        STRATEGY_VERSION
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
        latest: &LatestPrice,
        features: &FeatureVector,
    ) -> Result<grand_edge_domain::StrategySignal, StrategyError> {
        let observed_volume = feature_i64(features, "observed_volume_1h").unwrap_or(0);
        let spread_pct = feature_f64(features, "spread_pct").unwrap_or(0.0);
        let volatility = feature_f64(features, "ewma_volatility_24h").unwrap_or(0.0);
        let price_age_seconds = feature_i64(features, "price_staleness_secs").unwrap_or(0);
        let mut explanation = base_explanation(self.id(), self.version(), "spread_edge");

        let (Some(high), Some(low)) = (latest.high.map(Gp::as_i64), latest.low.map(Gp::as_i64))
        else {
            explanation.insert(
                "rejection_reason".to_string(),
                serde_json::json!("missing_high_or_low"),
            );
            return strategy_signal(
                self.id(),
                self.version(),
                ctx,
                item,
                SignalSide::Avoid,
                0.2,
                0.0,
                0,
                None,
                None,
                None,
                None,
                None,
                None,
                explanation,
            );
        };

        let target_buy = low + self.config.buy_buffer_gp;
        let target_sell = high - self.config.sell_buffer_gp;
        let tax = ctx
            .market_rules
            .tax_for_sale(item.item_id, Gp(target_sell))
            .as_i64();
        let expected_edge = target_sell
            - tax
            - target_buy
            - self.config.slippage_buffer_gp
            - self.config.liquidity_uncertainty_buffer_gp;
        let roi = if target_buy > 0 {
            expected_edge as f64 / target_buy as f64
        } else {
            0.0
        };

        explanation.insert("target_buy".to_string(), serde_json::json!(target_buy));
        explanation.insert("target_sell".to_string(), serde_json::json!(target_sell));
        explanation.insert("tax".to_string(), serde_json::json!(tax));
        explanation.insert(
            "expected_edge".to_string(),
            serde_json::json!(expected_edge),
        );
        explanation.insert("roi".to_string(), serde_json::json!(roi));
        explanation.insert(
            "liquidity_note".to_string(),
            serde_json::json!("observed volume is a proxy, not true liquidity"),
        );

        let side = if observed_volume < self.config.min_observed_volume
            || spread_pct < self.config.min_spread_pct
            || expected_edge <= self.config.required_margin_gp
        {
            SignalSide::Watch
        } else if roi >= self.config.min_roi {
            SignalSide::Buy
        } else {
            SignalSide::Avoid
        };

        let execution_estimate = execution_estimate_from_proxy(
            observed_liquidity_proxy(
                observed_volume.max(1),
                feature_i64(features, "observed_high_side_volume_1h").unwrap_or(1),
                feature_i64(features, "observed_low_side_volume_1h").unwrap_or(1),
                feature_f64(features, "observed_volume_z_24h"),
                feature_f64(features, "observed_volume_reliability_24h"),
                feature_f64(features, "high_low_volume_ratio_1h"),
            ),
            0.55,
            0.6,
            item.buy_limit.map(i64::from).unwrap_or(0).max(1),
            ctx.risk.participation_rate,
            0.5,
            spread_pct,
            price_age_seconds.max(1),
            volatility,
        );

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            side,
            if side == SignalSide::Buy { 0.7 } else { 0.4 },
            roi,
            expected_edge.max(0),
            Some(target_buy.max(1)),
            Some(target_sell.max(1)),
            Some((target_buy - 1).max(1)),
            Some((target_sell + 1).max(1)),
            item.buy_limit.map(i64::from),
            Some(execution_estimate),
            explanation,
        )
    }
}
