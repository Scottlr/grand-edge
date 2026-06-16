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
