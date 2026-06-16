#[derive(Debug, thiserror::Error)]
pub enum RecommendationError {
    #[error("storage error: {0}")]
    Storage(#[from] grand_edge_storage::StorageError),
    #[error("metrics error: {0}")]
    Metrics(#[from] grand_edge_metrics::MetricsError),
    #[error("simulator error: {0}")]
    Simulator(#[from] grand_edge_simulator::SimulatorError),
    #[error("domain validation error: {0}")]
    DomainValidation(#[from] grand_edge_domain::DomainValidationError),
    #[error("missing feature vector for item {0}")]
    MissingFeatures(i64),
    #[error("missing latest price snapshot for item {0}")]
    MissingLatestPrice(i64),
    #[error("recommendation confidence out of range: {0}")]
    InvalidConfidence(f64),
    #[error("recommendation score is not finite: {0}")]
    InvalidRate(f64),
}
