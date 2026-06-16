use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use grand_edge_domain::{
    FeatureVector, ItemId, ModelVersion, Prediction, PredictionDirection, PredictionId,
    Probability, Rate, StrategyId,
};
use grand_edge_strategies::PredictionSource;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    artifacts::{ArtifactBundle, ValidatedArtifactBundle},
    coefficients,
    errors::ModelRuntimeError,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub feature_snapshot_id: Uuid,
    pub item_id: ItemId,
    pub as_of: DateTime<Utc>,
    pub feature_vector: FeatureVector,
    pub artifact: ValidatedArtifactBundle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRuntimePrediction {
    pub prediction: Prediction,
    pub source: PredictionSource,
    pub artifact_hash: String,
    pub feature_schema_hash: String,
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

    pub fn infer(
        &self,
        request: InferenceRequest,
    ) -> Result<ModelRuntimePrediction, ModelRuntimeError> {
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
                        crate::artifacts::ModelArtifactKind::GraphRanker => "graph_ranker",
                        crate::artifacts::ModelArtifactKind::GraphNeuralNetworkDeferred => {
                            "graph_neural_network_deferred"
                        }
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

pub(crate) fn inference_to_prediction(
    request: &InferenceRequest,
    strategy_id: StrategyId,
    model_version: ModelVersion,
    expected_return: Rate,
    probability_positive: Probability,
    explanation: serde_json::Value,
    source: PredictionSource,
) -> Result<ModelRuntimePrediction, ModelRuntimeError> {
    let predicted_direction = if expected_return.get() > 0.0 {
        PredictionDirection::Up
    } else if expected_return.get() < 0.0 {
        PredictionDirection::Down
    } else {
        PredictionDirection::Flat
    };

    Ok(ModelRuntimePrediction {
        prediction: Prediction {
            prediction_id: PredictionId(Uuid::new_v4()),
            feature_snapshot_id: request.feature_snapshot_id,
            item_id: request.item_id,
            as_of: request.as_of,
            horizon_secs: grand_edge_domain::HorizonSecs(3_600),
            model_id: strategy_id,
            model_version,
            predicted_direction,
            predicted_return: Some(expected_return),
            confidence: probability_positive,
            prediction_interval: None,
            explanation,
            created_at: request.as_of,
        },
        source,
        artifact_hash: request.artifact.bundle.metadata.artifact_uri.clone(),
        feature_schema_hash: request.artifact.feature_schema_hash().to_string(),
    })
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
