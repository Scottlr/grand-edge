#[derive(Debug, thiserror::Error)]
pub enum RecommendationError {
    #[error("storage error: {0}")]
    Storage(#[from] grand_edge_storage::StorageError),
    #[error("metrics error: {0}")]
    Metrics(#[from] grand_edge_metrics::MetricsError),
    #[error("simulator error: {0}")]
    Simulator(#[from] grand_edge_simulator::SimulatorError),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
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
    #[error("duplicate prediction link for prediction {0}")]
    DuplicatePredictionLink(uuid::Uuid),
    #[error("prediction contribution weight is not finite: {0}")]
    InvalidContributionWeight(f64),
    #[error("prediction contribution weight must not be negative: {0}")]
    NegativeContributionWeight(f64),
    #[error("recommendation action `{0:?}` requires at least one prediction")]
    MissingPredictionsForAction(grand_edge_domain::RecommendationAction),
    #[error("strategy adapter error: {0}")]
    Strategy(#[from] grand_edge_strategies::StrategyError),
}
