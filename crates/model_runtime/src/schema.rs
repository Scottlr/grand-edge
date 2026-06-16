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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactFeatureSchemaDocument {
    pub feature_set_version: String,
    pub feature_names: Vec<String>,
    pub target_label: TrainingTargetLabel,
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
    target_label: TrainingTargetLabel,
) -> String {
    let quoted_feature_names = feature_names
        .iter()
        .map(|name| format!("'{}'", python_repr_escape(name)))
        .collect::<Vec<_>>()
        .join(", ");
    let repr = format!(
        "{{'feature_set_version': '{}', 'feature_names': [{}], 'target_label': '{}'}}",
        python_repr_escape(feature_set_version),
        quoted_feature_names,
        target_label.as_python_str()
    );

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
