use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::errors::ModelRuntimeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TrainingTargetLabel {
    #[serde(rename = "future_return_6h")]
    FutureReturn6h,
    #[serde(rename = "future_tax_adjusted_return_6h")]
    FutureTaxAdjustedReturn6h,
    #[serde(rename = "future_actionable_return_6h")]
    FutureActionableReturn6h,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GraphFeatureGroup {
    OwnItemFeatures,
    ObservedExecutionProxyFeatures,
    NeighborReturnFeatures,
    SectorFeatures,
    ConversionFeatures,
    ShockFeatures,
    EdgeStabilityFeatures,
    EventFeatures,
    MissingDataFlags,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GraphArtifactMetadata {
    pub graph_feature_set_version: String,
    pub graph_version: String,
    pub relation_corpus_hash: String,
    pub edge_observation_window_start: DateTime<Utc>,
    pub edge_observation_window_end: DateTime<Utc>,
    pub graph_feature_groups: Vec<GraphFeatureGroup>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactFeatureSchemaDocument {
    pub feature_set_version: String,
    pub feature_names: Vec<String>,
    pub target_label: TrainingTargetLabel,
    #[serde(default)]
    pub graph_feature_groups: Vec<GraphFeatureGroup>,
    pub feature_schema_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ModelCardDocument {
    pub strategy_id: String,
    pub model_version: String,
    pub feature_set_version: String,
    pub feature_schema_hash: String,
    pub training_window_start: DateTime<Utc>,
    pub training_window_end: DateTime<Utc>,
    pub evaluation_window_start: DateTime<Utc>,
    pub evaluation_window_end: DateTime<Utc>,
    pub metrics: serde_json::Map<String, serde_json::Value>,
    pub known_limitations: Vec<String>,
    pub target_label: TrainingTargetLabel,
    pub notes: String,
    #[serde(default)]
    pub graph: Option<GraphArtifactMetadata>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CalibrationDocument {
    pub method: String,
    pub fitted_at: DateTime<Utc>,
    pub bins: Vec<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CoefficientModelDocument {
    pub model_id: String,
    pub version: String,
    pub features: Vec<String>,
    pub intercept: f64,
    pub weights: Vec<f64>,
}

impl TrainingTargetLabel {
    pub fn as_python_str(self) -> &'static str {
        match self {
            Self::FutureReturn6h => "future_return_6h",
            Self::FutureTaxAdjustedReturn6h => "future_tax_adjusted_return_6h",
            Self::FutureActionableReturn6h => "future_actionable_return_6h",
        }
    }
}

pub fn compute_feature_schema_hash(
    feature_set_version: &str,
    feature_names: &[String],
    graph_feature_groups: &[GraphFeatureGroup],
    target_label: TrainingTargetLabel,
) -> String {
    let quoted_feature_names = feature_names
        .iter()
        .map(|name| format!("'{}'", python_repr_escape(name)))
        .collect::<Vec<_>>()
        .join(", ");
    let quoted_graph_feature_groups = graph_feature_groups
        .iter()
        .map(|group| format!("'{}'", graph_feature_group_as_str(*group)))
        .collect::<Vec<_>>()
        .join(", ");
    let repr = if graph_feature_groups.is_empty() {
        format!(
            "{{'feature_set_version': '{}', 'feature_names': [{}], 'target_label': '{}'}}",
            python_repr_escape(feature_set_version),
            quoted_feature_names,
            target_label.as_python_str()
        )
    } else {
        format!(
            "{{'feature_set_version': '{}', 'feature_names': [{}], 'target_label': '{}', 'graph_feature_groups': [{}]}}",
            python_repr_escape(feature_set_version),
            quoted_feature_names,
            target_label.as_python_str(),
            quoted_graph_feature_groups
        )
    };

    let digest = Sha256::digest(repr.as_bytes());
    format!("sha256:{digest:x}")
}

pub fn validate_feature_schema(
    schema: &ArtifactFeatureSchemaDocument,
    expected_feature_set_version: &str,
) -> Result<(), ModelRuntimeError> {
    if schema.feature_set_version != expected_feature_set_version {
        return Err(ModelRuntimeError::Validation(
            "feature schema version did not match expected version".to_string(),
        ));
    }

    let expected_hash = compute_feature_schema_hash(
        &schema.feature_set_version,
        &schema.feature_names,
        &schema.graph_feature_groups,
        schema.target_label,
    );
    if expected_hash != schema.feature_schema_hash {
        return Err(ModelRuntimeError::FeatureSchemaHashMismatch);
    }

    Ok(())
}

pub fn validate_model_card(
    card: &ModelCardDocument,
    as_of: DateTime<Utc>,
) -> Result<(), ModelRuntimeError> {
    if card.strategy_id.trim().is_empty() {
        return Err(ModelRuntimeError::Validation(
            "model card strategy_id must not be empty".to_string(),
        ));
    }
    if card.model_version.trim().is_empty() {
        return Err(ModelRuntimeError::Validation(
            "model card model_version must not be empty".to_string(),
        ));
    }
    if card.feature_schema_hash.trim().is_empty() {
        return Err(ModelRuntimeError::Validation(
            "model card requires feature_schema_hash".to_string(),
        ));
    }
    if card.training_window_end > as_of {
        return Err(ModelRuntimeError::Validation(
            "artifact training window must not extend past as_of".to_string(),
        ));
    }
    if card.training_window_end < card.training_window_start {
        return Err(ModelRuntimeError::Validation(
            "artifact training window end must be after start".to_string(),
        ));
    }
    if card.evaluation_window_start < card.training_window_end {
        return Err(ModelRuntimeError::Validation(
            "artifact evaluation window must start after training window end".to_string(),
        ));
    }
    if card.evaluation_window_end < card.evaluation_window_start {
        return Err(ModelRuntimeError::Validation(
            "artifact evaluation window end must be after evaluation start".to_string(),
        ));
    }
    if card.evaluation_window_end > as_of {
        return Err(ModelRuntimeError::Validation(
            "artifact evaluation window must not extend past as_of".to_string(),
        ));
    }
    if let Some(graph) = &card.graph {
        validate_graph_metadata(graph)?;
    }
    let lower_notes = card.notes.to_ascii_lowercase();
    if lower_notes.contains("causal") && !lower_notes.contains("non-causal") {
        return Err(ModelRuntimeError::CausalLearnedEdgeClaim);
    }
    for limitation in &card.known_limitations {
        let lower_limitation = limitation.to_ascii_lowercase();
        if lower_limitation.contains("causal") && !lower_limitation.contains("non-causal") {
            return Err(ModelRuntimeError::CausalLearnedEdgeClaim);
        }
    }

    Ok(())
}

pub fn validate_graph_metadata(graph: &GraphArtifactMetadata) -> Result<(), ModelRuntimeError> {
    if graph.graph_feature_set_version.trim().is_empty() {
        return Err(ModelRuntimeError::Validation(
            "graph artifacts require graph_feature_set_version".to_string(),
        ));
    }
    if graph.graph_version.trim().is_empty() {
        return Err(ModelRuntimeError::Validation(
            "graph artifacts require graph_version".to_string(),
        ));
    }
    if graph.relation_corpus_hash.trim().is_empty() {
        return Err(ModelRuntimeError::MissingRelationCorpusHash);
    }
    if graph.graph_feature_groups.is_empty() {
        return Err(ModelRuntimeError::MissingGraphFeatureGroups);
    }
    if graph.edge_observation_window_end < graph.edge_observation_window_start {
        return Err(ModelRuntimeError::Validation(
            "graph edge observation window end must be after start".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_coefficient_model(
    model: &CoefficientModelDocument,
) -> Result<(), ModelRuntimeError> {
    if model.model_id.trim().is_empty() {
        return Err(ModelRuntimeError::Validation(
            "coefficient model_id must not be empty".to_string(),
        ));
    }
    if model.features.len() != model.weights.len() {
        return Err(ModelRuntimeError::Validation(
            "coefficient features and weights must have the same length".to_string(),
        ));
    }
    if !model.intercept.is_finite() || model.weights.iter().any(|weight| !weight.is_finite()) {
        return Err(ModelRuntimeError::Validation(
            "coefficient intercept and weights must be finite".to_string(),
        ));
    }

    Ok(())
}

fn python_repr_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn graph_feature_group_as_str(group: GraphFeatureGroup) -> &'static str {
    match group {
        GraphFeatureGroup::OwnItemFeatures => "own_item_features",
        GraphFeatureGroup::ObservedExecutionProxyFeatures => "observed_execution_proxy_features",
        GraphFeatureGroup::NeighborReturnFeatures => "neighbor_return_features",
        GraphFeatureGroup::SectorFeatures => "sector_features",
        GraphFeatureGroup::ConversionFeatures => "conversion_features",
        GraphFeatureGroup::ShockFeatures => "shock_features",
        GraphFeatureGroup::EdgeStabilityFeatures => "edge_stability_features",
        GraphFeatureGroup::EventFeatures => "event_features",
        GraphFeatureGroup::MissingDataFlags => "missing_data_flags",
    }
}
