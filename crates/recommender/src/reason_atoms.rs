use grand_edge_domain::{
    InvalidationRule, MarketRules, Prediction, ReasonAtom, ReasonDirection, ReasonType,
    Recommendation, RecommendationAction,
};
use serde::{Deserialize, Serialize};

use crate::{RecommendationError, scoring::ScoreComponent};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskProfile {
    pub min_buy_score: f64,
    pub min_watch_score: f64,
    pub min_execution_confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataQualitySnapshot {
    pub freshness_confidence: f64,
    pub completeness_confidence: f64,
    pub stale: bool,
    pub missing_inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReasonAtomInputs<'a> {
    pub recommendation: &'a Recommendation,
    pub predictions: &'a [Prediction],
    pub score_components: &'a [ScoreComponent],
    pub market_rules: &'a MarketRules,
    pub risk_profile: &'a RiskProfile,
    pub data_quality: &'a DataQualitySnapshot,
}

pub fn build_reason_atoms(
    inputs: ReasonAtomInputs<'_>,
) -> Result<Vec<ReasonAtom>, RecommendationError> {
    let mut atoms = Vec::new();

    for prediction in inputs.predictions {
        atoms.push(ReasonAtom {
            reason_type: ReasonType::ModelSignal,
            reason_key: format!(
                "model_signal:{}:{}",
                prediction.model_id.0, prediction.horizon_secs.0
            ),
            label: format!(
                "{} predicted {:?}",
                prediction.model_id.0, prediction.predicted_direction
            ),
            direction: if prediction
                .predicted_return
                .map(|value| value.get())
                .unwrap_or_default()
                >= 0.0
            {
                ReasonDirection::Positive
            } else {
                ReasonDirection::Negative
            },
            weight: prediction.confidence.get(),
            evidence: serde_json::json!({
                "prediction_id": prediction.prediction_id.0,
                "predicted_return": prediction.predicted_return.map(|value| value.get()),
                "confidence": prediction.confidence.get(),
            }),
        });
    }

    push_score_component_atom(
        &mut atoms,
        inputs.score_components,
        "liquidity_penalty",
        ReasonType::LiquidityCheck,
        "liquidity:volume_capacity",
        "Liquidity capacity",
        true,
    )?;
    push_score_component_atom(
        &mut atoms,
        inputs.score_components,
        "risk_penalty",
        ReasonType::RiskCheck,
        "risk:profile_limit",
        "Risk profile",
        true,
    )?;
    push_score_component_atom(
        &mut atoms,
        inputs.score_components,
        "model_confidence_bonus",
        ReasonType::CalibrationCheck,
        "calibration:model_recent_error",
        "Calibration",
        false,
    )?;

    atoms.push(ReasonAtom {
        reason_type: ReasonType::CostCheck,
        reason_key: "cost:tax_and_spread".to_string(),
        label: "Tax and spread".to_string(),
        direction: if inputs
            .recommendation
            .expected_net_gp
            .map(|value| value.as_i64())
            .unwrap_or_default()
            > 0
        {
            ReasonDirection::Positive
        } else {
            ReasonDirection::Negative
        },
        weight: inputs
            .recommendation
            .expected_roi
            .map(|value| value.get().abs())
            .unwrap_or(0.0),
        evidence: serde_json::json!({
            "tax_rate": inputs.market_rules.tax_rate.get(),
            "expected_net_gp": inputs.recommendation.expected_net_gp.map(|value| value.as_i64()),
            "expected_roi": inputs.recommendation.expected_roi.map(|value| value.get()),
        }),
    });

    atoms.push(ReasonAtom {
        reason_type: ReasonType::UserExposureCheck,
        reason_key: "user_exposure:item_position".to_string(),
        label: "User exposure".to_string(),
        direction: match inputs.recommendation.action {
            RecommendationAction::Add | RecommendationAction::Hold => ReasonDirection::Positive,
            RecommendationAction::Avoid => ReasonDirection::Negative,
            _ => ReasonDirection::Neutral,
        },
        weight: 0.2,
        evidence: serde_json::json!({
            "action": serde_json::to_value(inputs.recommendation.action).ok(),
            "risk_label": inputs.recommendation.risk_label,
        }),
    });

    atoms.push(ReasonAtom {
        reason_type: ReasonType::RuleCheck,
        reason_key: "rule:buy_limit_tax_slot".to_string(),
        label: "Market rules".to_string(),
        direction: ReasonDirection::Neutral,
        weight: 0.1,
        evidence: serde_json::json!({
            "slot_limit": inputs.market_rules.slot_limit,
            "buy_limit_window_secs": inputs.market_rules.buy_limit_window_secs.0,
            "tax_cap_gp": inputs.market_rules.tax_cap_gp.as_i64(),
        }),
    });

    atoms.push(ReasonAtom {
        reason_type: ReasonType::DataQualityCheck,
        reason_key: "data_quality:freshness_completeness".to_string(),
        label: "Data quality".to_string(),
        direction: if inputs.data_quality.stale || !inputs.data_quality.missing_inputs.is_empty() {
            ReasonDirection::Negative
        } else {
            ReasonDirection::Positive
        },
        weight: ((inputs.data_quality.freshness_confidence
            + inputs.data_quality.completeness_confidence)
            / 2.0)
            .clamp(0.0, 1.0),
        evidence: serde_json::json!({
            "freshness_confidence": inputs.data_quality.freshness_confidence,
            "completeness_confidence": inputs.data_quality.completeness_confidence,
            "stale": inputs.data_quality.stale,
            "missing_inputs": inputs.data_quality.missing_inputs,
        }),
    });

    for atom in &atoms {
        atom.validate()?;
    }

    Ok(atoms)
}

pub fn reason_key(reason_type: ReasonType, stable_name: &str) -> String {
    let prefix = match reason_type {
        ReasonType::ModelSignal => "model_signal",
        ReasonType::CostCheck => "cost",
        ReasonType::LiquidityCheck => "liquidity",
        ReasonType::RiskCheck => "risk",
        ReasonType::CalibrationCheck => "calibration",
        ReasonType::DataQualityCheck => "data_quality",
        ReasonType::UserExposureCheck => "user_exposure",
        ReasonType::RuleCheck => "rule",
    };
    format!("{prefix}:{stable_name}")
}

pub fn build_invalidation_rules(
    recommendation: &Recommendation,
    score_components: &[ScoreComponent],
    market_rules: &MarketRules,
    risk_profile: &RiskProfile,
) -> Vec<InvalidationRule> {
    let score = score_components
        .iter()
        .find(|component| component.name == "final_score")
        .map(|component| component.value)
        .unwrap_or(recommendation.score.get());

    vec![
        InvalidationRule {
            rule_key: "score_threshold".to_string(),
            label: "Score threshold".to_string(),
            metric: "final_score".to_string(),
            operator: "<".to_string(),
            threshold: risk_profile.min_watch_score.to_string(),
            current_value: Some(score.to_string()),
        },
        InvalidationRule {
            rule_key: "tax_cap".to_string(),
            label: "Tax cap".to_string(),
            metric: "tax_cap_gp".to_string(),
            operator: ">".to_string(),
            threshold: market_rules.tax_cap_gp.as_i64().to_string(),
            current_value: recommendation
                .expected_net_gp
                .map(|value| value.as_i64().to_string()),
        },
    ]
}

fn push_score_component_atom(
    atoms: &mut Vec<ReasonAtom>,
    score_components: &[ScoreComponent],
    key: &str,
    reason_type: ReasonType,
    reason_key_value: &str,
    label: &str,
    negative_when_non_zero: bool,
) -> Result<(), RecommendationError> {
    if let Some(component) = score_components
        .iter()
        .find(|component| component.name == key)
    {
        let weight = component.value.abs();
        atoms.push(ReasonAtom {
            reason_type,
            reason_key: reason_key_value.to_string(),
            label: label.to_string(),
            direction: if negative_when_non_zero && component.value < 0.0 {
                ReasonDirection::Negative
            } else if component.value > 0.0 {
                ReasonDirection::Positive
            } else {
                ReasonDirection::Neutral
            },
            weight,
            evidence: serde_json::json!({
                "score_component": component.name,
                "value": component.value,
                "explanation": component.explanation,
            }),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        Gp, HorizonSecs, ItemId, ModelVersion, Prediction, PredictionDirection, PredictionId,
        Probability, Rate, Recommendation, RecommendationAction, RecommendationId, StrategyId,
        StructuredRecommendationExplanation,
    };
    use uuid::Uuid;

    use crate::scoring::ScoreComponent;

    use super::{
        DataQualitySnapshot, ReasonAtomInputs, RiskProfile, build_invalidation_rules,
        build_reason_atoms,
    };

    #[test]
    fn reason_atom_keys_are_stable_for_same_inputs() {
        let recommendation = recommendation(RecommendationAction::Buy);
        let predictions = vec![prediction()];
        let score_components = score_components();
        let market_rules = grand_edge_domain::MarketRules::default();
        let risk_profile = risk_profile();
        let data_quality = good_quality();

        let first = build_reason_atoms(ReasonAtomInputs {
            recommendation: &recommendation,
            predictions: &predictions,
            score_components: &score_components,
            market_rules: &market_rules,
            risk_profile: &risk_profile,
            data_quality: &data_quality,
        })
        .unwrap();
        let second = build_reason_atoms(ReasonAtomInputs {
            recommendation: &recommendation,
            predictions: &predictions,
            score_components: &score_components,
            market_rules: &market_rules,
            risk_profile: &risk_profile,
            data_quality: &data_quality,
        })
        .unwrap();

        let first_keys = first
            .into_iter()
            .map(|atom| atom.reason_key)
            .collect::<Vec<_>>();
        let second_keys = second
            .into_iter()
            .map(|atom| atom.reason_key)
            .collect::<Vec<_>>();
        assert_eq!(first_keys, second_keys);
    }

    #[test]
    fn avoid_recommendation_can_have_positive_model_signal_atom() {
        let recommendation = recommendation(RecommendationAction::Avoid);
        let atoms = build_reason_atoms(ReasonAtomInputs {
            recommendation: &recommendation,
            predictions: &[prediction()],
            score_components: &score_components(),
            market_rules: &grand_edge_domain::MarketRules::default(),
            risk_profile: &risk_profile(),
            data_quality: &good_quality(),
        })
        .unwrap();

        assert!(atoms.iter().any(|atom| {
            atom.reason_type == grand_edge_domain::ReasonType::ModelSignal
                && atom.direction == grand_edge_domain::ReasonDirection::Positive
        }));
    }

    #[test]
    fn invalidation_rule_contains_metric_operator_threshold_and_current_value() {
        let rules = build_invalidation_rules(
            &recommendation(RecommendationAction::Watch),
            &score_components(),
            &grand_edge_domain::MarketRules::default(),
            &risk_profile(),
        );
        assert!(rules.iter().any(|rule| {
            !rule.metric.is_empty()
                && !rule.operator.is_empty()
                && !rule.threshold.is_empty()
                && rule.current_value.is_some()
        }));
    }

    #[test]
    fn stale_data_creates_negative_data_quality_reason() {
        let recommendation = recommendation(RecommendationAction::Watch);
        let atoms = build_reason_atoms(ReasonAtomInputs {
            recommendation: &recommendation,
            predictions: &[prediction()],
            score_components: &score_components(),
            market_rules: &grand_edge_domain::MarketRules::default(),
            risk_profile: &risk_profile(),
            data_quality: &DataQualitySnapshot {
                freshness_confidence: 0.2,
                completeness_confidence: 0.5,
                stale: true,
                missing_inputs: vec!["price_staleness_secs".to_string()],
            },
        })
        .unwrap();
        assert!(atoms.iter().any(|atom| {
            atom.reason_type == grand_edge_domain::ReasonType::DataQualityCheck
                && atom.direction == grand_edge_domain::ReasonDirection::Negative
        }));
    }

    fn recommendation(action: RecommendationAction) -> Recommendation {
        Recommendation {
            recommendation_id: RecommendationId(Uuid::new_v4()),
            user_id: None,
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            action,
            score: Rate::new(0.4).unwrap(),
            prediction_confidence: Some(Probability::new(0.8).unwrap()),
            execution_confidence: Some(Probability::new(0.4).unwrap()),
            recommendation_confidence: Probability::new(0.55).unwrap(),
            expected_net_gp: Some(Gp(1200)),
            expected_roi: Some(Rate::new(0.03).unwrap()),
            risk_label: Some("medium".to_string()),
            reasons: vec!["fixture".to_string()],
            explanation: grand_edge_domain::RecommendationExplanation {
                feature_set_version: "features_v1".to_string(),
                market_rules_version: "rules_v1".to_string(),
                graph_version: None,
                graph_context: None,
                strategy_votes: Vec::new(),
                score_components: Vec::new(),
                accuracy_snapshot: None,
                structured_explanation: StructuredRecommendationExplanation::default(),
            },
        }
    }

    fn prediction() -> Prediction {
        Prediction {
            prediction_id: PredictionId(Uuid::new_v4()),
            feature_snapshot_id: Uuid::new_v4(),
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            horizon_secs: HorizonSecs(3600),
            model_id: StrategyId::new("spread_edge_v1").unwrap(),
            model_version: ModelVersion::new("v1").unwrap(),
            predicted_direction: PredictionDirection::Up,
            predicted_return: Some(Rate::new(0.03).unwrap()),
            confidence: Probability::new(0.8).unwrap(),
            prediction_interval: None,
            explanation: serde_json::json!({}),
            created_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    fn score_components() -> Vec<ScoreComponent> {
        vec![
            ScoreComponent {
                name: "final_score".to_string(),
                value: 0.4,
                explanation: "Final score".to_string(),
            },
            ScoreComponent {
                name: "liquidity_penalty".to_string(),
                value: -0.2,
                explanation: "Liquidity".to_string(),
            },
            ScoreComponent {
                name: "risk_penalty".to_string(),
                value: -0.1,
                explanation: "Risk".to_string(),
            },
            ScoreComponent {
                name: "model_confidence_bonus".to_string(),
                value: 0.05,
                explanation: "Calibration".to_string(),
            },
        ]
    }

    fn risk_profile() -> RiskProfile {
        RiskProfile {
            min_buy_score: 0.35,
            min_watch_score: 0.05,
            min_execution_confidence: 0.45,
        }
    }

    fn good_quality() -> DataQualitySnapshot {
        DataQualitySnapshot {
            freshness_confidence: 0.9,
            completeness_confidence: 1.0,
            stale: false,
            missing_inputs: Vec::new(),
        }
    }
}
