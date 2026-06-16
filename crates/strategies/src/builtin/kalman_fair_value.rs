use grand_edge_domain::{FeatureVector, Gp, Item, LatestPrice, SignalSide};

use crate::{
    LookbackSpec, Strategy, StrategyContext, StrategyError, kalman_update, math::KalmanConfig,
};

use super::{base_explanation, strategy_signal};

const STRATEGY_ID: &str = "kalman_fair_value_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone)]
pub struct KalmanFairValueStrategy {
    pub config: KalmanConfig,
}

impl Default for KalmanFairValueStrategy {
    fn default() -> Self {
        Self {
            config: KalmanConfig::default(),
        }
    }
}

impl Strategy for KalmanFairValueStrategy {
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
        _features: &FeatureVector,
    ) -> Result<grand_edge_domain::StrategySignal, StrategyError> {
        let mut explanation = base_explanation(self.id(), self.version(), "kalman_fair_value");
        explanation.insert(
            "method".to_string(),
            serde_json::json!("kalman_fair_value_heuristic_v1"),
        );
        explanation.insert(
            "assumptions".to_string(),
            serde_json::json!(["single-item online fair value", "no order-book depth"]),
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

        let observed_mid = (high + low) as f64 / 2.0;
        let prior = crate::KalmanState {
            fair_value: observed_mid * 0.995,
            variance: self.config.observation_variance.max(1.0),
        };
        let update = kalman_update(prior, observed_mid, self.config);
        let target_entry = low.max(1);
        let target_exit = update.posterior.fair_value.round() as i64;
        let tax = ctx
            .market_rules
            .tax_for_sale(item.item_id, Gp(target_exit.max(1)))
            .as_i64();
        let expected_net_gp = (target_exit - target_entry - tax).max(0);
        let side =
            if update.mispricing >= self.config.buy_mispricing_threshold && expected_net_gp > 0 {
                SignalSide::Buy
            } else if update.mispricing <= self.config.cashout_mispricing_threshold {
                SignalSide::Cashout
            } else {
                SignalSide::Watch
            };
        let directional_accuracy = ctx
            .recent_metrics
            .get(self.id())
            .and_then(|snapshot| snapshot.directional_accuracy.map(|value| value.get()));
        let confidence = directional_accuracy
            .unwrap_or(0.5)
            .max(0.45 + update.mispricing.abs().min(0.2));

        explanation.insert(
            "inputs".to_string(),
            serde_json::json!({
                "observed_mid": observed_mid,
                "posterior_fair_value": update.posterior.fair_value,
                "mispricing": update.mispricing,
                "kalman_gain": update.kalman_gain,
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
            update.mispricing,
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
    use grand_edge_domain::{FeatureVector, Gp, Item, ItemId, LatestPrice, SignalSide};

    use super::KalmanFairValueStrategy;
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

    fn latest(high: i64, low: i64) -> LatestPrice {
        LatestPrice {
            item_id: ItemId(4151),
            high: Some(Gp(high)),
            high_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 59, 0).unwrap()),
            low: Some(Gp(low)),
            low_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 58, 0).unwrap()),
            observed_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    fn features() -> FeatureVector {
        FeatureVector {
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values: serde_json::Map::new(),
        }
    }

    #[test]
    fn kalman_strategy_holds_small_mispricing() {
        let signal = KalmanFairValueStrategy::default()
            .generate(
                &test_context(),
                &item(),
                &latest(103_000, 102_000),
                &features(),
            )
            .unwrap();

        assert_eq!(signal.side, SignalSide::Watch);
    }
}
