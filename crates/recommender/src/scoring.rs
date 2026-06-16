use grand_edge_domain::{FeatureVector, ModelAccuracySnapshot, StrategySignal};
use serde::{Deserialize, Serialize};

use crate::RecommendationConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreComponent {
    pub name: String,
    pub value: f64,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationScore {
    pub raw_edge: f64,
    pub liquidity_adjusted_edge: f64,
    pub prediction_confidence: Option<f64>,
    pub execution_confidence: Option<f64>,
    pub recommendation_confidence: f64,
    pub risk_penalty: f64,
    pub liquidity_penalty: f64,
    pub model_confidence_bonus: f64,
    pub user_fit_bonus: f64,
    pub final_score: f64,
    pub components: Vec<ScoreComponent>,
}

pub fn score_candidate(
    signal: &StrategySignal,
    features: &FeatureVector,
    accuracy: Option<&ModelAccuracySnapshot>,
    config: &RecommendationConfig,
) -> RecommendationScore {
    let raw_edge = signal.expected_return.get();
    let prediction_confidence = Some(signal.confidence.get());
    let execution_confidence = signal
        .execution_estimate
        .as_ref()
        .and_then(|estimate| estimate.estimated_fill_probability)
        .map(|value| value.get())
        .or_else(|| {
            signal
                .execution_estimate
                .as_ref()
                .and_then(|estimate| estimate.liquidity_confidence)
                .map(|value| value.get())
        });

    let volatility = feature_f64(features, "ewma_volatility_24h").unwrap_or(0.0);
    let spread_pct = feature_f64(features, "spread_pct").unwrap_or(0.0);
    let staleness_penalty = feature_f64(features, "price_staleness_secs").unwrap_or(0.0) / 3600.0
        * config.lambda_staleness;
    let volatility_penalty = volatility * config.lambda_volatility;
    let spread_penalty = spread_pct * config.lambda_spread;
    let overlay = overlay_penalties(signal);
    let risk_penalty = volatility_penalty
        + spread_penalty
        + staleness_penalty
        + overlay.volatility_penalty
        + overlay.spread_penalty
        + overlay.staleness_penalty
        + overlay.regime_penalty;

    let liquidity_penalty = signal
        .execution_estimate
        .as_ref()
        .and_then(|estimate| estimate.liquidity_confidence)
        .map(|value| (1.0 - value.get()) * config.lambda_liquidity)
        .unwrap_or(config.lambda_liquidity * 0.5);

    let model_confidence_bonus = accuracy
        .and_then(|snapshot| snapshot.directional_accuracy)
        .map(|value| value.get() * config.confidence_bonus_weight)
        .unwrap_or(0.0);
    let execution_bonus = execution_confidence
        .map(|value| value * config.execution_confidence_weight)
        .unwrap_or(0.0);
    let user_fit_bonus = 0.0;
    let liquidity_adjusted_edge = raw_edge - liquidity_penalty;
    let recommendation_confidence = clamp01(
        prediction_confidence.unwrap_or(0.0) * 0.5
            + execution_confidence.unwrap_or(prediction_confidence.unwrap_or(0.0)) * 0.3
            + accuracy
                .and_then(|snapshot| snapshot.directional_accuracy)
                .map(|value| value.get())
                .unwrap_or(0.5)
                * 0.2,
    );
    let final_score =
        raw_edge - risk_penalty - liquidity_penalty + model_confidence_bonus + execution_bonus;

    let components = vec![
        component("raw_edge", raw_edge, "Expected return before penalties."),
        component(
            "liquidity_adjusted_edge",
            liquidity_adjusted_edge,
            "Raw edge after liquidity penalty.",
        ),
        component(
            "prediction_confidence",
            prediction_confidence.unwrap_or(0.0),
            "Strategy-level confidence from the prediction.",
        ),
        component(
            "execution_confidence",
            execution_confidence.unwrap_or(0.0),
            "Estimated execution confidence from proxy liquidity.",
        ),
        component(
            "recommendation_confidence",
            recommendation_confidence,
            "Blended confidence for the final recommendation.",
        ),
        component(
            "risk_penalty",
            -risk_penalty,
            "Penalty from volatility, spread, and staleness.",
        ),
        component(
            "regime_penalty",
            -overlay.regime_penalty,
            "Additional penalty from advisory market regime overlays.",
        ),
        component(
            "overlay_volatility_penalty",
            -overlay.volatility_penalty,
            "Overlay penalty for extreme volatility.",
        ),
        component(
            "overlay_spread_penalty",
            -overlay.spread_penalty,
            "Overlay penalty for wide spreads.",
        ),
        component(
            "overlay_staleness_penalty",
            -overlay.staleness_penalty,
            "Overlay penalty for stale prices.",
        ),
        component(
            "liquidity_penalty",
            -liquidity_penalty,
            "Penalty for uncertain execution quality.",
        ),
        component(
            "model_confidence_bonus",
            model_confidence_bonus,
            "Bonus from recent measured model accuracy.",
        ),
        component(
            "user_fit_bonus",
            user_fit_bonus,
            "User-fit bonus for position-aware logic.",
        ),
        component("final_score", final_score, "Final decision score."),
    ];

    RecommendationScore {
        raw_edge,
        liquidity_adjusted_edge,
        prediction_confidence,
        execution_confidence,
        recommendation_confidence,
        risk_penalty,
        liquidity_penalty,
        model_confidence_bonus,
        user_fit_bonus,
        final_score,
        components,
    }
}

fn feature_f64(features: &FeatureVector, key: &str) -> Option<f64> {
    features.values.get(key).and_then(|value| value.as_f64())
}

#[derive(Default)]
struct OverlayPenaltyBreakdown {
    volatility_penalty: f64,
    spread_penalty: f64,
    staleness_penalty: f64,
    regime_penalty: f64,
}

fn overlay_penalties(signal: &StrategySignal) -> OverlayPenaltyBreakdown {
    let Some(overlay) = signal.explanation.get("risk_overlay") else {
        return OverlayPenaltyBreakdown::default();
    };

    OverlayPenaltyBreakdown {
        volatility_penalty: overlay
            .get("volatility_penalty")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        spread_penalty: overlay
            .get("spread_penalty")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        staleness_penalty: overlay
            .get("staleness_penalty")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        regime_penalty: overlay
            .get("regime_penalty")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
    }
}

fn component(name: &str, value: f64, explanation: &str) -> ScoreComponent {
    ScoreComponent {
        name: name.to_string(),
        value,
        explanation: explanation.to_string(),
    }
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        ExecutionEstimate, FeatureVector, Gp, HorizonSecs, ItemId, ModelAccuracySnapshot,
        ModelVersion, ObservedLiquidityProxy, Probability, Quantity, Rate, SignalSide, StrategyId,
        StrategySignal,
    };

    use crate::RecommendationConfig;

    use super::score_candidate;

    #[test]
    fn confidence_breakdown_keeps_prediction_execution_and_recommendation_separate() {
        let signal = signal();
        let features = features();
        let accuracy = ModelAccuracySnapshot {
            strategy_id: StrategyId("momentum_v1".to_string()),
            model_version: ModelVersion("v1".to_string()),
            lookback_window: "seven_days".to_string(),
            sample_size: 10,
            directional_accuracy: Some(Rate::new(0.7).unwrap()),
            brier_score: Some(Rate::new(0.2).unwrap()),
            avg_realized_roi: Some(Rate::new(0.03).unwrap()),
            max_drawdown: Some(Rate::new(0.1).unwrap()),
            calibration: serde_json::json!({}),
        };

        let score = score_candidate(
            &signal,
            &features,
            Some(&accuracy),
            &RecommendationConfig::default(),
        );
        assert_eq!(score.prediction_confidence, Some(0.8));
        assert_eq!(score.execution_confidence, Some(0.6));
        assert_ne!(score.recommendation_confidence, 0.8);
        assert_ne!(score.recommendation_confidence, 0.6);
    }

    #[test]
    fn score_components_sum_to_final_score() {
        let score = score_candidate(
            &signal(),
            &features(),
            None,
            &RecommendationConfig::default(),
        );
        let computed = score.raw_edge - score.risk_penalty - score.liquidity_penalty
            + score.model_confidence_bonus
            + score.execution_confidence.unwrap_or(0.0)
                * RecommendationConfig::default().execution_confidence_weight
            + score.user_fit_bonus;
        assert!((computed - score.final_score).abs() < 1e-9);
    }

    fn signal() -> StrategySignal {
        StrategySignal {
            item_id: ItemId(4151),
            strategy_id: StrategyId("momentum_v1".to_string()),
            model_version: ModelVersion("v1".to_string()),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            side: SignalSide::Buy,
            horizon_secs: HorizonSecs(3_600),
            confidence: Probability::new(0.8).unwrap(),
            expected_return: Rate::new(0.05).unwrap(),
            expected_net_gp_per_unit: Gp(1_500),
            target_entry: Some(Gp(100_000)),
            target_exit: Some(Gp(105_000)),
            stop_loss: None,
            take_profit: None,
            max_quantity: Some(Quantity(10)),
            execution_estimate: Some(ExecutionEstimate {
                observed_liquidity: ObservedLiquidityProxy {
                    observed_volume: Quantity(400),
                    observed_high_side_volume: Quantity(220),
                    observed_low_side_volume: Quantity(180),
                    observed_volume_z: None,
                    observed_volume_reliability: None,
                    high_low_volume_ratio: None,
                    note: "proxy".to_string(),
                },
                estimated_fill_probability: Some(Probability::new(0.6).unwrap()),
                liquidity_confidence: Some(Probability::new(0.7).unwrap()),
                estimated_capacity: Some(Quantity(10)),
                participation_rate: None,
                confidence_haircut: None,
                spread_pct: Some(Rate::new(0.02).unwrap()),
                price_staleness_seconds: Some(HorizonSecs(120)),
                volatility: Some(Rate::new(0.03).unwrap()),
            }),
            explanation: serde_json::json!({}),
        }
    }

    fn features() -> FeatureVector {
        let mut values = serde_json::Map::new();
        values.insert("ewma_volatility_24h".to_string(), serde_json::json!(0.03));
        values.insert("spread_pct".to_string(), serde_json::json!(0.02));
        values.insert("price_staleness_secs".to_string(), serde_json::json!(120));
        FeatureVector {
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values,
        }
    }
}
