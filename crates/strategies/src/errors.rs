#[derive(Debug, thiserror::Error)]
pub enum StrategyError {
    #[error("duplicate strategy id `{0}`")]
    DuplicateStrategyId(String),
    #[error("signal validation failed: {0}")]
    Validation(String),
    #[error("missing model artifact for strategy `{0}`")]
    MissingArtifact(String),
    #[error("storage operation failed")]
    Storage(#[from] grand_edge_storage::StorageError),
}
