#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("config load failed: {0}")]
    Config(#[from] config::ConfigError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("environment error: {0}")]
    Env(#[from] std::env::VarError),
    #[error("storage error: {0}")]
    Storage(#[from] grand_edge_storage::StorageError),
    #[error("ingest error: {0}")]
    Ingest(#[from] grand_edge_ingest::IngestError),
    #[error("invalid configuration: {0}")]
    Invalid(String),
    #[error("tracing subscriber init failed")]
    TracingInit,
}
