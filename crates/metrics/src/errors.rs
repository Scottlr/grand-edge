#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("storage error: {0}")]
    Storage(#[from] grand_edge_storage::StorageError),
    #[error("domain validation error: {0}")]
    DomainValidation(#[from] grand_edge_domain::DomainValidationError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("recommendation {0} is missing an evaluable strategy signal")]
    MissingOutcomeSignal(grand_edge_domain::RecommendationId),
    #[error("recommendation {0} is missing required future price data")]
    MissingPriceData(grand_edge_domain::RecommendationId),
}
