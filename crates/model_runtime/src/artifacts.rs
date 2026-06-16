use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    errors::ModelRuntimeError,
    schema::{
        ArtifactFeatureSchemaDocument, CalibrationDocument, CoefficientModelDocument,
        ModelCardDocument, TrainingTargetLabel, validate_coefficient_model,
        validate_feature_schema, validate_model_card,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelArtifactKind {
    GbdtRanker,
    ContextualBandit,
    OnlineEnsemble,
    MetaLabel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelArtifactMetadata {
    pub strategy_id: String,
    pub model_version: String,
    pub feature_set_version: String,
    pub feature_schema_hash: String,
    pub trained_at: DateTime<Utc>,
    pub training_window_start: DateTime<Utc>,
    pub training_window_end: DateTime<Utc>,
    pub evaluation_window_start: DateTime<Utc>,
    pub evaluation_window_end: DateTime<Utc>,
    pub artifact_uri: String,
    pub artifact_kind: ModelArtifactKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactBundle {
    pub root: PathBuf,
    pub metadata: ModelArtifactMetadata,
    pub model_card_path: PathBuf,
    pub feature_schema_path: PathBuf,
    pub calibration_path: PathBuf,
    pub model_path: Option<PathBuf>,
    pub coefficient_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatedArtifactBundle {
    pub bundle: ArtifactBundle,
    pub feature_schema: ArtifactFeatureSchemaDocument,
    pub model_card: ModelCardDocument,
    pub calibration: CalibrationDocument,
    pub coefficient_model: Option<CoefficientModelDocument>,
}

impl ArtifactBundle {
    pub fn from_root(
        root: &Path,
        as_of: DateTime<Utc>,
    ) -> Result<ValidatedArtifactBundle, ModelRuntimeError> {
        let model_card_path = root.join("model_card.json");
        let feature_schema_path = root.join("feature_schema.json");
        let calibration_path = root.join("calibration.json");

        for required in [&model_card_path, &feature_schema_path, &calibration_path] {
            if !required.is_file() {
                return Err(ModelRuntimeError::MissingFile(required.to_path_buf()));
            }
        }

        let model_card: ModelCardDocument =
            serde_json::from_str(&std::fs::read_to_string(&model_card_path)?)?;
        validate_model_card(&model_card, as_of)?;

        let feature_schema: ArtifactFeatureSchemaDocument =
            serde_json::from_str(&std::fs::read_to_string(&feature_schema_path)?)?;
        validate_feature_schema(&feature_schema, &model_card.feature_set_version)?;

        if feature_schema.target_label != model_card.target_label {
            return Err(ModelRuntimeError::Validation(
                "feature schema target_label did not match model card".to_string(),
            ));
        }
        if feature_schema.feature_schema_hash != model_card.feature_schema_hash {
            return Err(ModelRuntimeError::FeatureSchemaHashMismatch);
        }

        let calibration: CalibrationDocument =
            serde_json::from_str(&std::fs::read_to_string(&calibration_path)?)?;

        let model_path = root.join("model.onnx");
        let coefficient_path = root.join("coefficients.json");
        let model_path = model_path.is_file().then_some(model_path);
        let coefficient_path = coefficient_path.is_file().then_some(coefficient_path);

        if model_path.is_none() && coefficient_path.is_none() {
            return Err(ModelRuntimeError::Validation(
                "artifact bundle must contain model.onnx or coefficients.json".to_string(),
            ));
        }

        let coefficient_model = if let Some(path) = &coefficient_path {
            let document: CoefficientModelDocument =
                serde_json::from_str(&std::fs::read_to_string(path)?)?;
            validate_coefficient_model(&document)?;
            if document.version != model_card.model_version {
                return Err(ModelRuntimeError::Validation(
                    "coefficient version did not match model card".to_string(),
                ));
            }
            if document.features != feature_schema.feature_names {
                return Err(ModelRuntimeError::FeatureSchemaMismatch(
                    "coefficient features did not match feature schema".to_string(),
                ));
            }
            Some(document)
        } else {
            None
        };

        let artifact_path = coefficient_path
            .clone()
            .or_else(|| model_path.clone())
            .ok_or_else(|| {
                ModelRuntimeError::Validation(
                    "artifact bundle must contain model.onnx or coefficients.json".to_string(),
                )
            })?;
        let metadata = ModelArtifactMetadata {
            strategy_id: model_card.strategy_id.clone(),
            model_version: model_card.model_version.clone(),
            feature_set_version: model_card.feature_set_version.clone(),
            feature_schema_hash: model_card.feature_schema_hash.clone(),
            trained_at: calibration.fitted_at,
            training_window_start: model_card.training_window_start,
            training_window_end: model_card.training_window_end,
            evaluation_window_start: model_card.evaluation_window_start,
            evaluation_window_end: model_card.evaluation_window_end,
            artifact_uri: artifact_path.canonicalize()?.to_string_lossy().into_owned(),
            artifact_kind: infer_artifact_kind(
                &model_card.strategy_id,
                coefficient_model.is_some(),
            )?,
        };

        Ok(ValidatedArtifactBundle {
            bundle: Self {
                root: root.to_path_buf(),
                metadata,
                model_card_path,
                feature_schema_path,
                calibration_path,
                model_path,
                coefficient_path,
            },
            feature_schema,
            model_card,
            calibration,
            coefficient_model,
        })
    }
}

pub fn infer_artifact_kind(
    strategy_id: &str,
    coefficient_backed: bool,
) -> Result<ModelArtifactKind, ModelRuntimeError> {
    if coefficient_backed {
        return Ok(ModelArtifactKind::MetaLabel);
    }

    if strategy_id.starts_with("gbm_ranker") {
        return Ok(ModelArtifactKind::GbdtRanker);
    }
    if strategy_id.starts_with("contextual_bandit") {
        return Ok(ModelArtifactKind::ContextualBandit);
    }
    if strategy_id.starts_with("online_ensemble") {
        return Ok(ModelArtifactKind::OnlineEnsemble);
    }
    if strategy_id.starts_with("meta_label") {
        return Ok(ModelArtifactKind::MetaLabel);
    }

    Err(ModelRuntimeError::Validation(format!(
        "could not infer artifact kind from strategy_id `{strategy_id}`"
    )))
}

impl ValidatedArtifactBundle {
    pub fn feature_schema_hash(&self) -> &str {
        &self.bundle.metadata.feature_schema_hash
    }

    pub fn target_label(&self) -> TrainingTargetLabel {
        self.feature_schema.target_label
    }
}
