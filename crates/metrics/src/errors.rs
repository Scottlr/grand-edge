#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("storage error: {0}")]
    Storage(#[from] grand_edge_storage::StorageError),
    #[error("domain validation error: {0}")]
    DomainValidation(#[from] grand_edge_domain::DomainValidationError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
