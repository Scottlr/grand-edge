use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use grand_edge_domain::{FeatureVector, ItemId, ModelVersion, Probability, Rate, StrategyId};
use serde::{Deserialize, Serialize};

use crate::{
    artifacts::{ArtifactBundle, ValidatedArtifactBundle},
    coefficients,
    errors::ModelRuntimeError,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub item_id: ItemId,
    pub as_of: DateTime<Utc>,
    pub feature_vector: FeatureVector,
    pub artifact: ValidatedArtifactBundle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InferenceOutput {
    pub strategy_id: StrategyId,
    pub model_version: ModelVersion,
    pub item_id: ItemId,
    pub as_of: DateTime<Utc>,
    pub expected_return: Rate,
    pub probability_positive: Probability,
    pub explanation: serde_json::Value,
}

pub struct ModelRuntime {
    artifact_root: PathBuf,
}

impl ModelRuntime {
    pub fn new(artifact_root: PathBuf) -> Self {
        Self { artifact_root }
    }

    pub fn load_bundle(
        &self,
        strategy_id: &StrategyId,
        model_version: &ModelVersion,
        as_of: DateTime<Utc>,
    ) -> Result<ValidatedArtifactBundle, ModelRuntimeError> {
        self.validate_bundle_path(
            &self
                .artifact_root
                .join(&strategy_id.0)
                .join(&model_version.0),
            as_of,
        )
    }

    pub fn validate_bundle_path(
        &self,
        artifact_path: &Path,
        as_of: DateTime<Utc>,
    ) -> Result<ValidatedArtifactBundle, ModelRuntimeError> {
        ArtifactBundle::from_root(artifact_path, as_of)
    }

    pub fn infer(&self, request: InferenceRequest) -> Result<InferenceOutput, ModelRuntimeError> {
        let artifact = request.artifact.clone();
        if request.feature_vector.feature_set_version
            != artifact.bundle.metadata.feature_set_version
        {
            return Err(ModelRuntimeError::FeatureSchemaMismatch(
                "feature_set_version did not match artifact".to_string(),
            ));
        }
        validate_feature_vector(&request.feature_vector, &artifact)?;

        if artifact.coefficient_model.is_some() {
            return coefficients::infer(request, &artifact);
        }

        if artifact.bundle.model_path.is_some() {
            #[cfg(feature = "onnx")]
            {
                return crate::onnx::infer(request, &artifact);
            }
            #[cfg(not(feature = "onnx"))]
            {
                return Err(ModelRuntimeError::UnsupportedArtifactKind(
                    match artifact.bundle.metadata.artifact_kind {
                        crate::artifacts::ModelArtifactKind::GbdtRanker => "gbdt_ranker",
                        crate::artifacts::ModelArtifactKind::ContextualBandit => {
                            "contextual_bandit"
                        }
                        crate::artifacts::ModelArtifactKind::OnlineEnsemble => "online_ensemble",
                        crate::artifacts::ModelArtifactKind::MetaLabel => "meta_label",
                    },
                ));
            }
        }

        Err(ModelRuntimeError::Validation(
            "artifact bundle had no inference backend".to_string(),
        ))
    }
}

fn validate_feature_vector(
    feature_vector: &FeatureVector,
    artifact: &ValidatedArtifactBundle,
) -> Result<(), ModelRuntimeError> {
    let schema_names = &artifact.feature_schema.feature_names;
    for required in schema_names {
        if !feature_vector.values.contains_key(required) {
            return Err(ModelRuntimeError::FeatureSchemaMismatch(format!(
                "feature `{required}` missing from feature vector"
            )));
        }
    }

    let extra_features = feature_vector
        .values
        .keys()
        .filter(|name| !schema_names.contains(name))
        .cloned()
        .collect::<Vec<_>>();
    if !extra_features.is_empty() {
        return Err(ModelRuntimeError::FeatureSchemaMismatch(format!(
            "feature vector contained unexpected features: {}",
            extra_features.join(", ")
        )));
    }

    if artifact.feature_schema_hash() != artifact.model_card.feature_schema_hash {
        return Err(ModelRuntimeError::FeatureSchemaHashMismatch);
    }

    Ok(())
}
