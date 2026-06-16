//! Rust-only model artifact validation and inference runtime.

pub mod artifacts;
pub mod coefficients;
pub mod errors;
#[cfg(feature = "onnx")]
pub mod onnx;
pub mod runtime;
pub mod schema;

pub use artifacts::{
    ArtifactBundle, ModelArtifactKind, ModelArtifactMetadata, ValidatedArtifactBundle,
};
pub use errors::ModelRuntimeError;
pub use runtime::{InferenceRequest, ModelRuntime, ModelRuntimePrediction};
pub use schema::{
    ArtifactFeatureSchemaDocument, CalibrationDocument, CoefficientModelDocument,
    GraphArtifactMetadata, GraphFeatureGroup, ModelCardDocument, TrainingTargetLabel,
    compute_feature_schema_hash, validate_coefficient_model, validate_feature_schema,
    validate_model_card,
};
