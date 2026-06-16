use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ModelRuntimeError {
    #[error("required artifact file missing: {0}")]
    MissingFile(PathBuf),
    #[error("artifact validation failed: {0}")]
    Validation(String),
    #[error("artifact feature schema hash mismatch")]
    FeatureSchemaHashMismatch,
    #[error("feature vector does not match artifact schema: {0}")]
    FeatureSchemaMismatch(String),
    #[error("graph artifact metadata missing")]
    MissingGraphMetadata,
    #[error("graph artifact relation_corpus_hash missing")]
    MissingRelationCorpusHash,
    #[error("graph artifact feature groups missing")]
    MissingGraphFeatureGroups,
    #[error("graph neural network artifacts are deferred until runtime support is added")]
    GraphNeuralNetworkDeferred,
    #[error("learned-edge artifacts must not claim causality in model card text")]
    CausalLearnedEdgeClaim,
    #[error("unsupported artifact kind for current runtime: {0}")]
    UnsupportedArtifactKind(&'static str),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("json error")]
    Json(#[from] serde_json::Error),
    #[error("domain validation error")]
    Domain(#[from] grand_edge_domain::DomainValidationError),
}
