use std::collections::BTreeMap;

use grand_edge_domain::FeatureVector;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketRegime {
    CalmLiquid,
    TrendingUp,
    TrendingDown,
    Volatile,
    Illiquid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RegimeEstimate {
    pub regime: MarketRegime,
    pub probability: f64,
    pub method: RegimeMethod,
    pub strategy_overrides: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegimeMethod {
    HeuristicV1,
    TrainedHmm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RegimeHeuristicConfig {
    pub high_volatility_z: f64,
    pub high_spread_pct: f64,
    pub low_observed_volume_z: f64,
    pub trend_return_threshold: f64,
}

pub fn classify_regime(features: &FeatureVector) -> RegimeEstimate {
    classify_regime_with_config(features, &RegimeHeuristicConfig::default())
}

pub fn classify_regime_with_config(
    features: &FeatureVector,
    config: &RegimeHeuristicConfig,
) -> RegimeEstimate {
    let volatility_z = feature_f64(features, "volatility_z_24h").unwrap_or(0.0);
    let spread_pct = feature_f64(features, "spread_pct").unwrap_or(0.0);
    let observed_volume_z = feature_f64(features, "observed_volume_z_24h").unwrap_or(0.0);
    let return_1h = feature_f64(features, "return_1h").unwrap_or(0.0);
    let return_6h = feature_f64(features, "return_6h").unwrap_or(0.0);
    let mut strategy_overrides = BTreeMap::new();

    let (regime, probability) = if spread_pct >= config.high_spread_pct
        && observed_volume_z <= config.low_observed_volume_z
    {
        strategy_overrides.insert("spread_edge_v1".to_string(), "deweight".to_string());
        strategy_overrides.insert(
            "execution_confidence_v1".to_string(),
            "prioritize".to_string(),
        );
        (MarketRegime::Illiquid, 0.88)
    } else if volatility_z >= config.high_volatility_z || spread_pct >= config.high_spread_pct {
        strategy_overrides.insert("mean_reversion_v1".to_string(), "deweight".to_string());
        strategy_overrides.insert("volatility_filter_v1".to_string(), "prioritize".to_string());
        (MarketRegime::Volatile, 0.84)
    } else if return_6h >= config.trend_return_threshold
        || return_1h >= config.trend_return_threshold
    {
        strategy_overrides.insert("momentum_v1".to_string(), "prioritize".to_string());
        (MarketRegime::TrendingUp, 0.73)
    } else if return_6h <= -config.trend_return_threshold
        || return_1h <= -config.trend_return_threshold
    {
        strategy_overrides.insert("momentum_v1".to_string(), "deweight".to_string());
        (MarketRegime::TrendingDown, 0.73)
    } else {
        strategy_overrides.insert("spread_edge_v1".to_string(), "normal".to_string());
        (MarketRegime::CalmLiquid, 0.65)
    };

    RegimeEstimate {
        regime,
        probability,
        method: RegimeMethod::HeuristicV1,
        strategy_overrides,
    }
}

fn feature_f64(features: &FeatureVector, key: &str) -> Option<f64> {
    features.values.get(key).and_then(|value| value.as_f64())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{FeatureVector, ItemId};

    use super::{MarketRegime, RegimeMethod, classify_regime};

    fn features(values: &[(&str, serde_json::Value)]) -> FeatureVector {
        let mut map = serde_json::Map::new();
        for (key, value) in values {
            map.insert((*key).to_string(), value.clone());
        }
        FeatureVector {
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values: map,
        }
    }

    #[test]
    fn classify_regime_volatile_fixture() {
        let estimate = classify_regime(&features(&[
            ("volatility_z_24h", serde_json::json!(2.5)),
            ("spread_pct", serde_json::json!(0.04)),
            ("observed_volume_z_24h", serde_json::json!(-0.2)),
        ]));

        assert!(matches!(
            estimate.regime,
            MarketRegime::Volatile | MarketRegime::Illiquid
        ));
    }

    #[test]
    fn classify_regime_trending_up_fixture() {
        let estimate = classify_regime(&features(&[
            ("return_1h", serde_json::json!(0.03)),
            ("return_6h", serde_json::json!(0.025)),
        ]));

        assert_eq!(estimate.regime, MarketRegime::TrendingUp);
    }

    #[test]
    fn heuristic_regime_does_not_return_trained_hmm_method() {
        let estimate = classify_regime(&features(&[]));
        assert_eq!(estimate.method, RegimeMethod::HeuristicV1);
    }
}
