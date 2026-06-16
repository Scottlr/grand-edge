use grand_edge_domain::{FeatureVector, Gp, Item, LatestPrice, SignalSide};

use crate::{
    LookbackSpec, Strategy, StrategyContext, StrategyError, forecast_next_price,
    math::ArBaselineConfig,
};

use super::{base_explanation, feature_f64, strategy_signal};

const STRATEGY_ID: &str = "arima_baseline_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone)]
pub struct ArBaselineStrategy {
    pub config: ArBaselineConfig,
}

impl Default for ArBaselineStrategy {
    fn default() -> Self {
        Self {
            config: ArBaselineConfig::default(),
        }
    }
}

impl Strategy for ArBaselineStrategy {
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
        latest: &LatestPrice,
        features: &FeatureVector,
    ) -> Result<grand_edge_domain::StrategySignal, StrategyError> {
        let mut explanation = base_explanation(self.id(), self.version(), "ar_baseline");
        explanation.insert(
            "method".to_string(),
            serde_json::json!("arima_baseline_simplified_ar1_v1"),
        );
        explanation.insert(
            "assumptions".to_string(),
            serde_json::json!([
                "simplified AR(1) on price differences, not full ARIMA",
                "latest mid price is a proxy for near-term fair value"
            ]),
        );

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

        let current_price = (high + low) as f64 / 2.0;
        let last_delta = feature_f64(features, "return_1h").unwrap_or(0.0) * current_price;
        let forecast = forecast_next_price(current_price, last_delta, self.config);
        let target_entry = low.max(1);
        let target_exit = forecast.forecast_price.round() as i64;
        let tax = ctx
            .market_rules
            .tax_for_sale(item.item_id, Gp(target_exit.max(1)))
            .as_i64();
        let expected_net_gp = (target_exit - target_entry - tax).max(0);
        let side =
            if forecast.expected_return >= self.config.min_expected_return && expected_net_gp > 0 {
                SignalSide::Buy
            } else if forecast.expected_return <= -self.config.min_expected_return {
                SignalSide::Cashout
            } else {
                SignalSide::Watch
            };
        let directional_accuracy = ctx
            .recent_metrics
            .get(self.id())
            .and_then(|snapshot| snapshot.directional_accuracy.map(|value| value.get()));
        let confidence = directional_accuracy
            .unwrap_or(self.config.confidence_floor)
            .max(self.config.confidence_floor + forecast.expected_return.abs().min(0.2));

        explanation.insert(
            "inputs".to_string(),
            serde_json::json!({
                "current_price": current_price,
                "last_delta": last_delta,
                "forecast_delta": forecast.forecast_delta,
                "forecast_price": forecast.forecast_price,
                "expected_return": forecast.expected_return,
                "expected_net_gp_per_unit": expected_net_gp,
            }),
        );
        if let Some(directional_accuracy) = directional_accuracy {
            explanation.insert(
                "recent_directional_accuracy".to_string(),
                serde_json::json!(directional_accuracy),
            );
        }

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            side,
            confidence.min(0.95),
            forecast.expected_return,
            expected_net_gp,
            Some(target_entry),
            Some(target_exit.max(1)),
            Some((target_entry - 1).max(1)),
            Some((target_exit + 1).max(1)),
            item.buy_limit.map(i64::from),
            None,
            explanation,
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{FeatureVector, Gp, Item, ItemId, LatestPrice};

    use super::ArBaselineStrategy;
    use crate::{Strategy, builtin::test_context};

    fn item() -> Item {
        Item {
            item_id: ItemId(4151),
            name: "Abyssal whip".to_string(),
            examine: None,
            members: true,
            buy_limit: Some(70),
            low_alch: None,
            high_alch: None,
            value: None,
            icon: None,
            updated_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    fn latest() -> LatestPrice {
        LatestPrice {
            item_id: ItemId(4151),
            high: Some(Gp(103_000)),
            high_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 59, 0).unwrap()),
            low: Some(Gp(101_000)),
            low_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 58, 0).unwrap()),
            observed_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    fn features() -> FeatureVector {
        let mut values = serde_json::Map::new();
        values.insert("return_1h".to_string(), serde_json::json!(0.02));

        FeatureVector {
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values,
        }
    }

    #[test]
    fn ar_strategy_discloses_simplified_ar1() {
        let signal = ArBaselineStrategy::default()
            .generate(&test_context(), &item(), &latest(), &features())
            .unwrap();

        assert!(
            signal
                .explanation
                .to_string()
                .contains("simplified AR(1) on price differences, not full ARIMA")
        );
    }
}
