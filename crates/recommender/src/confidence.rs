use grand_edge_domain::{ConfidenceBreakdown, Prediction, Probability, ReasonAtom};
use serde::{Deserialize, Serialize};

use crate::{RecommendationError, scoring::ScoreComponent};

use super::reason_atoms::DataQualitySnapshot;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationSnapshot {
    pub recent_directional_accuracy: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiquiditySnapshot {
    pub liquidity_confidence: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceInputs<'a> {
    pub predictions: &'a [Prediction],
    pub score_components: &'a [ScoreComponent],
    pub calibration: Option<&'a CalibrationSnapshot>,
    pub liquidity: Option<&'a LiquiditySnapshot>,
    pub data_quality: &'a DataQualitySnapshot,
    pub explanation_atoms: &'a [ReasonAtom],
}

pub fn build_confidence_breakdown(
    inputs: ConfidenceInputs<'_>,
) -> Result<ConfidenceBreakdown, RecommendationError> {
    let prediction_confidence = average(
        &inputs
            .predictions
            .iter()
            .map(|prediction| prediction.confidence.get())
            .collect::<Vec<_>>(),
    );
    let recommendation_confidence = inputs
        .score_components
        .iter()
        .find(|component| component.name == "recommendation_confidence")
        .map(|component| component.value)
        .unwrap_or(prediction_confidence);
    let data_quality_confidence = ((inputs.data_quality.freshness_confidence
        + inputs.data_quality.completeness_confidence)
        / 2.0)
        .clamp(0.0, 1.0);
    let model_calibration_confidence = inputs
        .calibration
        .and_then(|snapshot| snapshot.recent_directional_accuracy)
        .unwrap_or(prediction_confidence)
        .clamp(0.0, 1.0);
    let liquidity_confidence = inputs
        .liquidity
        .and_then(|snapshot| snapshot.liquidity_confidence)
        .unwrap_or(prediction_confidence)
        .clamp(0.0, 1.0);
    let explanation_confidence = if inputs.explanation_atoms.is_empty() {
        0.0
    } else {
        average(
            &inputs
                .explanation_atoms
                .iter()
                .map(|atom| atom.weight.abs().clamp(0.0, 1.0))
                .collect::<Vec<_>>(),
        )
    };

    Ok(ConfidenceBreakdown {
        prediction_confidence: Probability::new(prediction_confidence)?,
        recommendation_confidence: Probability::new(recommendation_confidence.clamp(0.0, 1.0))?,
        data_quality_confidence: Probability::new(data_quality_confidence)?,
        model_calibration_confidence: Probability::new(model_calibration_confidence)?,
        liquidity_confidence: Probability::new(liquidity_confidence)?,
        explanation_confidence: Probability::new(explanation_confidence)?,
    })
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        Prediction, PredictionDirection, PredictionId, Probability, Rate, ReasonAtom,
        ReasonDirection, ReasonType,
    };
    use uuid::Uuid;

    use crate::scoring::ScoreComponent;

    use super::{
        CalibrationSnapshot, ConfidenceInputs, LiquiditySnapshot, build_confidence_breakdown,
    };

    #[test]
    fn confidence_breakdown_contains_six_dimensions() {
        let breakdown = build_confidence_breakdown(ConfidenceInputs {
            predictions: &[prediction()],
            score_components: &[ScoreComponent {
                name: "recommendation_confidence".to_string(),
                value: 0.65,
                explanation: "blended".to_string(),
            }],
            calibration: Some(&CalibrationSnapshot {
                recent_directional_accuracy: Some(0.7),
            }),
            liquidity: Some(&LiquiditySnapshot {
                liquidity_confidence: Some(0.6),
            }),
            data_quality: &super::DataQualitySnapshot {
                freshness_confidence: 0.9,
                completeness_confidence: 0.8,
                stale: false,
                missing_inputs: Vec::new(),
            },
            explanation_atoms: &[ReasonAtom {
                reason_type: ReasonType::ModelSignal,
                reason_key: "model_signal:spread_edge:3600".to_string(),
                label: "Model signal".to_string(),
                direction: ReasonDirection::Positive,
                weight: 0.7,
                evidence: serde_json::json!({}),
            }],
        })
        .unwrap();

        assert_probability_eq(breakdown.prediction_confidence.get(), 0.8);
        assert_probability_eq(breakdown.recommendation_confidence.get(), 0.65);
        assert_probability_eq(breakdown.data_quality_confidence.get(), 0.85);
        assert_probability_eq(breakdown.model_calibration_confidence.get(), 0.7);
        assert_probability_eq(breakdown.liquidity_confidence.get(), 0.6);
        assert_probability_eq(breakdown.explanation_confidence.get(), 0.7);
    }

    fn prediction() -> Prediction {
        Prediction {
            prediction_id: PredictionId(Uuid::new_v4()),
            feature_snapshot_id: Uuid::new_v4(),
            item_id: grand_edge_domain::ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            horizon_secs: grand_edge_domain::HorizonSecs(3600),
            model_id: grand_edge_domain::StrategyId::new("spread_edge").unwrap(),
            model_version: grand_edge_domain::ModelVersion::new("v1").unwrap(),
            predicted_direction: PredictionDirection::Up,
            predicted_return: Some(Rate::new(0.03).unwrap()),
            confidence: Probability::new(0.8).unwrap(),
            prediction_interval: None,
            explanation: serde_json::json!({}),
            created_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    fn assert_probability_eq(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9);
    }
}
