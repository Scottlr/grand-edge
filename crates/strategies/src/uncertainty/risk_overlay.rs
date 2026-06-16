use grand_edge_domain::FeatureVector;
use serde::{Deserialize, Serialize};

use crate::{MarketRegime, RegimeEstimate, RiskConfig};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskOverlay {
    pub volatility_penalty: f64,
    pub spread_penalty: f64,
    pub staleness_penalty: f64,
    pub regime_penalty: f64,
    pub final_multiplier: f64,
    pub reasons: Vec<RiskOverlayReason>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskOverlayReason {
    pub key: String,
    pub label: String,
    pub penalty: f64,
}

pub fn advanced_risk_overlay(
    features: &FeatureVector,
    regime: &RegimeEstimate,
    config: &RiskConfig,
) -> RiskOverlay {
    let volatility_penalty = feature_f64(features, "ewma_volatility_24h")
        .unwrap_or(0.0)
        .max(0.0)
        * config.overlay_volatility_penalty_weight;
    let spread_penalty = feature_f64(features, "spread_pct").unwrap_or(0.0).max(0.0)
        * config.overlay_spread_penalty_weight;
    let staleness_penalty = (feature_f64(features, "price_staleness_secs").unwrap_or(0.0) / 3600.0)
        .max(0.0)
        * config.overlay_staleness_penalty_weight;
    let regime_penalty = match regime.regime {
        MarketRegime::Illiquid => config.overlay_regime_penalty_weight,
        MarketRegime::Volatile => config.overlay_regime_penalty_weight * 0.75,
        MarketRegime::TrendingDown => config.overlay_regime_penalty_weight * 0.5,
        _ => 0.0,
    };

    let mut reasons = Vec::new();
    if volatility_penalty > 0.0 {
        reasons.push(reason(
            "high_volatility",
            "High volatility reduces conviction.",
            volatility_penalty,
        ));
    }
    if spread_penalty > 0.0 {
        reasons.push(reason(
            "wide_spread",
            "Wide spread hurts execution.",
            spread_penalty,
        ));
    }
    if staleness_penalty > 0.0 {
        reasons.push(reason(
            "stale_price",
            "Price snapshot is stale.",
            staleness_penalty,
        ));
    }
    if regime_penalty > 0.0 {
        reasons.push(reason(
            match regime.regime {
                MarketRegime::Illiquid => "illiquid_regime",
                MarketRegime::Volatile => "volatile_regime",
                MarketRegime::TrendingDown => "trending_down_regime",
                _ => "regime_penalty",
            },
            "Current market regime increases risk.",
            regime_penalty,
        ));
    }

    let final_multiplier =
        (1.0 - volatility_penalty - spread_penalty - staleness_penalty - regime_penalty)
            .clamp(0.0, 1.0);

    RiskOverlay {
        volatility_penalty,
        spread_penalty,
        staleness_penalty,
        regime_penalty,
        final_multiplier,
        reasons,
    }
}

fn feature_f64(features: &FeatureVector, key: &str) -> Option<f64> {
    features.values.get(key).and_then(|value| value.as_f64())
}

fn reason(key: &str, label: &str, penalty: f64) -> RiskOverlayReason {
    RiskOverlayReason {
        key: key.to_string(),
        label: label.to_string(),
        penalty,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{FeatureVector, ItemId};

    use super::advanced_risk_overlay;
    use crate::{MarketRegime, RegimeEstimate, RegimeMethod, RiskConfig};

    fn features() -> FeatureVector {
        let mut values = serde_json::Map::new();
        values.insert("ewma_volatility_24h".to_string(), serde_json::json!(0.20));
        values.insert("spread_pct".to_string(), serde_json::json!(0.04));
        values.insert(
            "price_staleness_secs".to_string(),
            serde_json::json!(7200.0),
        );
        FeatureVector {
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values,
        }
    }

    #[test]
    fn risk_overlay_explains_each_penalty() {
        let overlay = advanced_risk_overlay(
            &features(),
            &RegimeEstimate {
                regime: MarketRegime::Illiquid,
                probability: 0.9,
                method: RegimeMethod::HeuristicV1,
                strategy_overrides: std::collections::BTreeMap::new(),
            },
            &RiskConfig::default(),
        );

        assert!(
            overlay
                .reasons
                .iter()
                .any(|reason| reason.key == "high_volatility")
        );
        assert!(
            overlay
                .reasons
                .iter()
                .any(|reason| reason.key == "wide_spread")
        );
        assert!(
            overlay
                .reasons
                .iter()
                .any(|reason| reason.key == "stale_price")
        );
        assert!(
            overlay
                .reasons
                .iter()
                .any(|reason| reason.key == "illiquid_regime")
        );
    }

    #[test]
    fn risk_overlay_multiplier_is_clamped() {
        let mut config = RiskConfig::default();
        config.overlay_volatility_penalty_weight = 5.0;
        config.overlay_spread_penalty_weight = 5.0;
        config.overlay_staleness_penalty_weight = 5.0;
        config.overlay_regime_penalty_weight = 5.0;
        let overlay = advanced_risk_overlay(
            &features(),
            &RegimeEstimate {
                regime: MarketRegime::Illiquid,
                probability: 0.9,
                method: RegimeMethod::HeuristicV1,
                strategy_overrides: std::collections::BTreeMap::new(),
            },
            &config,
        );
        assert!((0.0..=1.0).contains(&overlay.final_multiplier));
    }
}
