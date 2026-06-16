use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DomainValidationError, ItemId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GraphVersion {
    pub graph_version: String,
    pub source_hash: String,
    pub created_at: DateTime<Utc>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ItemGraphNode {
    pub item_id: ItemId,
    pub graph_version: String,
    pub category: Option<String>,
    pub metadata: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GraphEdgeType {
    IngredientOf,
    ComponentOfSet,
    RepairConversion,
    DoseConversion,
    AlchFloor,
    DeathCofferValue,
    ChargeConversion,
    DegradeConversion,
    ShopFloorCeiling,
    Substitute,
    Complement,
    SharedSource,
    SharedSink,
    SameCategory,
    EventLinked,
    PlayerBehaviourLink,
    CorrelatedWith,
    Leads,
    CoMovesAfterEvents,
    ShockTransmitsTo,
    GraphNeighborPredictive,
    RegimeDependentLink,
    CandidateDoseConversion,
    CandidateNamePattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GraphEdgeDirection {
    Upstream,
    Downstream,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GraphEdgeSourceType {
    Mechanical,
    Curated,
    PatternCandidate,
    Learned,
    EventCorpus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgeObservationMethod {
    Correlation,
    LeadLagRegression,
    GrangerStyle,
    VarImpulseResponse,
    EventStudy,
    OutcomeBacktest,
    ChangePoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GraphRecommendationAction {
    BuyLinked,
    PairTrade,
    Rotate,
    Hedge,
    ExploitConversion,
    AvoidBlastRadius,
    CashoutBeforeContagion,
    WatchSecondOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum GraphDomainError {
    #[error(transparent)]
    DomainValidation(#[from] DomainValidationError),
    #[error("graph_version must not be empty")]
    EmptyGraphVersion,
    #[error("source_hash must not be empty")]
    EmptySourceHash,
    #[error("description must not be empty")]
    EmptyDescription,
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("{field} must be within [0.0, 1.0]")]
    OutOfRange { field: &'static str },
    #[error("{field} must be finite")]
    NonFinite { field: &'static str },
    #[error("sign must be -1.0, 0.0, or 1.0")]
    InvalidSign,
    #[error("pattern candidate edges must require review")]
    PatternCandidateRequiresReview,
    #[error("learned edges above confidence 0.5 require at least one observation")]
    LearnedEdgeRequiresObservation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ItemGraphEdge {
    pub edge_id: Uuid,
    pub graph_version: String,
    pub from_item_id: ItemId,
    pub to_item_id: ItemId,
    pub edge_type: GraphEdgeType,
    pub direction: GraphEdgeDirection,
    pub sign: f64,
    pub weight: f64,
    pub lag_seconds: Option<i64>,
    pub confidence: f64,
    pub source_type: GraphEdgeSourceType,
    pub source_ref: Option<String>,
    #[serde(default)]
    pub observations: Vec<EdgeObservation>,
    pub formula: serde_json::Value,
    pub requires_review: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EdgeObservation {
    pub edge_id: Uuid,
    pub observed_at: DateTime<Utc>,
    pub method: EdgeObservationMethod,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub statistic: Option<f64>,
    pub p_value: Option<f64>,
    pub estimated_lag_seconds: Option<i64>,
    pub estimated_effect: Option<f64>,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GraphPathStep {
    pub from_item_id: ItemId,
    pub to_item_id: ItemId,
    pub edge_id: Uuid,
    pub edge_type: GraphEdgeType,
    pub confidence: f64,
    pub weight: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GraphPath {
    pub source_item_id: ItemId,
    pub target_item_id: ItemId,
    pub steps: Vec<GraphPathStep>,
    pub path_confidence: f64,
    pub expected_impact: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GraphRecommendationContext {
    pub graph_version: String,
    #[serde(default)]
    pub graph_action: Option<GraphRecommendationAction>,
    #[serde(default)]
    pub paths: Vec<GraphPath>,
    #[serde(default)]
    pub edge_confidence: Option<f64>,
    #[serde(default)]
    pub historical_path_performance: Option<serde_json::Value>,
}

impl GraphVersion {
    pub fn validate(&self) -> Result<(), GraphDomainError> {
        if self.graph_version.trim().is_empty() {
            return Err(GraphDomainError::EmptyGraphVersion);
        }
        if self.source_hash.trim().is_empty() {
            return Err(GraphDomainError::EmptySourceHash);
        }
        if self.description.trim().is_empty() {
            return Err(GraphDomainError::EmptyDescription);
        }

        Ok(())
    }
}

pub fn validate_edge_confidence(confidence: f64) -> Result<(), GraphDomainError> {
    validate_bounded_value(confidence, "confidence")
}

pub fn validate_edge_weight(weight: f64) -> Result<(), GraphDomainError> {
    validate_bounded_value(weight, "weight")
}

pub fn validate_graph_edge(edge: &ItemGraphEdge) -> Result<(), GraphDomainError> {
    if edge.graph_version.trim().is_empty() {
        return Err(GraphDomainError::EmptyGraphVersion);
    }

    if edge
        .source_ref
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(GraphDomainError::EmptyField {
            field: "source_ref",
        });
    }

    validate_edge_confidence(edge.confidence)?;
    validate_edge_weight(edge.weight)?;

    if !matches!(edge.sign, -1.0 | 0.0 | 1.0) {
        return Err(GraphDomainError::InvalidSign);
    }

    if matches!(edge.source_type, GraphEdgeSourceType::PatternCandidate) && !edge.requires_review {
        return Err(GraphDomainError::PatternCandidateRequiresReview);
    }

    if matches!(edge.source_type, GraphEdgeSourceType::Learned)
        && edge.confidence > 0.5
        && edge.observations.is_empty()
    {
        return Err(GraphDomainError::LearnedEdgeRequiresObservation);
    }

    for observation in &edge.observations {
        observation.validate()?;
    }

    Ok(())
}

impl EdgeObservation {
    pub fn validate(&self) -> Result<(), GraphDomainError> {
        validate_edge_confidence(self.confidence)
    }
}

impl GraphRecommendationContext {
    pub fn validate(&self) -> Result<(), GraphDomainError> {
        if self.graph_version.trim().is_empty() {
            return Err(GraphDomainError::EmptyGraphVersion);
        }

        if let Some(confidence) = self.edge_confidence {
            validate_edge_confidence(confidence)?;
        }

        for path in &self.paths {
            path.validate()?;
        }

        Ok(())
    }
}

impl GraphPath {
    pub fn validate(&self) -> Result<(), GraphDomainError> {
        validate_bounded_value(self.path_confidence, "path_confidence")?;

        for step in &self.steps {
            validate_edge_confidence(step.confidence)?;
            validate_edge_weight(step.weight)?;
        }

        Ok(())
    }
}

fn validate_bounded_value(value: f64, field: &'static str) -> Result<(), GraphDomainError> {
    if !value.is_finite() {
        return Err(GraphDomainError::NonFinite { field });
    }

    if !(0.0..=1.0).contains(&value) {
        return Err(GraphDomainError::OutOfRange { field });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{
        EdgeObservation, EdgeObservationMethod, GraphDomainError, GraphEdgeDirection,
        GraphEdgeSourceType, GraphEdgeType, GraphRecommendationContext, ItemGraphEdge,
        validate_graph_edge,
    };
    use crate::ItemId;

    fn sample_edge(source_type: GraphEdgeSourceType) -> ItemGraphEdge {
        ItemGraphEdge {
            edge_id: Uuid::new_v4(),
            graph_version: "graph_v1".to_string(),
            from_item_id: ItemId(4151),
            to_item_id: ItemId(11840),
            edge_type: GraphEdgeType::IngredientOf,
            direction: GraphEdgeDirection::Upstream,
            sign: 1.0,
            weight: 0.6,
            lag_seconds: Some(300),
            confidence: 0.8,
            source_type,
            source_ref: Some("source_registry.v1.json:mechanical".to_string()),
            observations: Vec::new(),
            formula: serde_json::json!({"formula_type": "recipe"}),
            requires_review: true,
            active: true,
            created_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    #[test]
    fn graph_edge_type_serde_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&GraphEdgeType::CandidateNamePattern).unwrap(),
            "\"candidate_name_pattern\""
        );
    }

    #[test]
    fn graph_edge_validation_rejects_confidence_above_one() {
        let mut edge = sample_edge(GraphEdgeSourceType::Mechanical);
        edge.confidence = 1.1;

        assert_eq!(
            validate_graph_edge(&edge),
            Err(GraphDomainError::OutOfRange {
                field: "confidence"
            })
        );
    }

    #[test]
    fn pattern_candidate_requires_review() {
        let mut edge = sample_edge(GraphEdgeSourceType::PatternCandidate);
        edge.requires_review = false;

        assert_eq!(
            validate_graph_edge(&edge),
            Err(GraphDomainError::PatternCandidateRequiresReview)
        );
    }

    #[test]
    fn learned_edge_requires_observation_for_high_confidence_use() {
        let edge = sample_edge(GraphEdgeSourceType::Learned);

        assert_eq!(
            validate_graph_edge(&edge),
            Err(GraphDomainError::LearnedEdgeRequiresObservation)
        );
    }

    #[test]
    fn learned_edge_with_observation_is_valid() {
        let mut edge = sample_edge(GraphEdgeSourceType::Learned);
        edge.observations.push(EdgeObservation {
            edge_id: edge.edge_id,
            observed_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            method: EdgeObservationMethod::OutcomeBacktest,
            window_start: Utc.with_ymd_and_hms(2026, 6, 15, 12, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            statistic: Some(0.3),
            p_value: Some(0.04),
            estimated_lag_seconds: Some(600),
            estimated_effect: Some(0.08),
            confidence: 0.7,
            metadata: serde_json::json!({}),
        });

        assert!(validate_graph_edge(&edge).is_ok());
    }

    #[test]
    fn graph_recommendation_context_rejects_empty_version() {
        let context = GraphRecommendationContext {
            graph_version: "   ".to_string(),
            graph_action: None,
            paths: Vec::new(),
            edge_confidence: None,
            historical_path_performance: None,
        };

        assert_eq!(context.validate(), Err(GraphDomainError::EmptyGraphVersion));
    }
}
