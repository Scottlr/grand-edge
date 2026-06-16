use serde::{Deserialize, Serialize};

use crate::{
    DomainValidationError, GraphRecommendationContext, Probability, RecommendationId, StrategyId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonDirection {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonType {
    ModelSignal,
    CostCheck,
    LiquidityCheck,
    RiskCheck,
    CalibrationCheck,
    DataQualityCheck,
    UserExposureCheck,
    RuleCheck,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasonAtom {
    pub reason_type: ReasonType,
    pub reason_key: String,
    pub label: String,
    pub direction: ReasonDirection,
    pub weight: f64,
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvalidationRule {
    pub rule_key: String,
    pub label: String,
    pub metric: String,
    pub operator: String,
    pub threshold: String,
    pub current_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceBreakdown {
    pub prediction_confidence: Probability,
    pub recommendation_confidence: Probability,
    pub data_quality_confidence: Probability,
    pub model_calibration_confidence: Probability,
    pub liquidity_confidence: Probability,
    pub explanation_confidence: Probability,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredRecommendationExplanation {
    pub summary: String,
    pub reason_atoms: Vec<ReasonAtom>,
    pub invalidation_rules: Vec<InvalidationRule>,
    pub confidence: ConfidenceBreakdown,
    #[serde(default)]
    pub graph_version: Option<String>,
    #[serde(default)]
    pub graph_reason_path_count: Option<usize>,
    #[serde(default)]
    pub graph_context: Option<GraphRecommendationContext>,
}

impl Default for ConfidenceBreakdown {
    fn default() -> Self {
        Self {
            prediction_confidence: Probability::new(0.0).expect("zero probability is valid"),
            recommendation_confidence: Probability::new(0.0).expect("zero probability is valid"),
            data_quality_confidence: Probability::new(0.0).expect("zero probability is valid"),
            model_calibration_confidence: Probability::new(0.0).expect("zero probability is valid"),
            liquidity_confidence: Probability::new(0.0).expect("zero probability is valid"),
            explanation_confidence: Probability::new(0.0).expect("zero probability is valid"),
        }
    }
}

impl Default for StructuredRecommendationExplanation {
    fn default() -> Self {
        Self {
            summary: String::new(),
            reason_atoms: Vec::new(),
            invalidation_rules: Vec::new(),
            confidence: ConfidenceBreakdown::default(),
            graph_version: None,
            graph_reason_path_count: None,
            graph_context: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationPredictionLink {
    pub recommendation_id: RecommendationId,
    pub prediction_id: crate::PredictionId,
    pub contribution_weight: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationPredictionContribution {
    pub prediction_id: crate::PredictionId,
    pub contribution_weight: f64,
    pub source_model_id: StrategyId,
    pub source_model_version: crate::ModelVersion,
}

impl ReasonAtom {
    pub fn validate(&self) -> Result<(), DomainValidationError> {
        if self.reason_key.trim().is_empty() {
            return Err(DomainValidationError::EmptyValue {
                field: "reason_key",
            });
        }
        if !self.weight.is_finite() {
            return Err(DomainValidationError::NonFiniteValue {
                field: "reason_weight",
            });
        }

        Ok(())
    }
}

impl RecommendationPredictionLink {
    pub fn new(
        recommendation_id: RecommendationId,
        prediction_id: crate::PredictionId,
        contribution_weight: f64,
    ) -> Result<Self, DomainValidationError> {
        if !contribution_weight.is_finite() {
            return Err(DomainValidationError::NonFiniteValue {
                field: "contribution_weight",
            });
        }

        Ok(Self {
            recommendation_id,
            prediction_id,
            contribution_weight,
        })
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        ConfidenceBreakdown, PredictionId, Probability, ReasonAtom, ReasonDirection, ReasonType,
        RecommendationId, RecommendationPredictionLink,
    };

    #[test]
    fn reason_atom_requires_reason_key() {
        let atom = ReasonAtom {
            reason_type: ReasonType::ModelSignal,
            reason_key: "   ".to_string(),
            label: "Model".to_string(),
            direction: ReasonDirection::Positive,
            weight: 0.4,
            evidence: serde_json::json!({}),
        };

        assert!(atom.validate().is_err());
    }

    #[test]
    fn confidence_breakdown_rejects_out_of_range_values() {
        assert!(Probability::new(1.2).is_err());
        let breakdown = ConfidenceBreakdown {
            prediction_confidence: Probability::new(0.7).unwrap(),
            recommendation_confidence: Probability::new(0.8).unwrap(),
            data_quality_confidence: Probability::new(0.9).unwrap(),
            model_calibration_confidence: Probability::new(0.85).unwrap(),
            liquidity_confidence: Probability::new(0.6).unwrap(),
            explanation_confidence: Probability::new(0.75).unwrap(),
        };
        assert_eq!(breakdown.recommendation_confidence.get(), 0.8);
    }

    #[test]
    fn recommendation_prediction_link_rejects_non_finite_weight() {
        let result = RecommendationPredictionLink::new(
            RecommendationId(Uuid::new_v4()),
            PredictionId(Uuid::new_v4()),
            f64::NAN,
        );
        assert!(result.is_err());
    }
}
