use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("window_end must be after window_start")]
    InvalidWindow,
    #[error("invalid retention policy: {0}")]
    InvalidRetentionPolicy(&'static str),
    #[error("simulation run `{0}` was not found")]
    MissingRun(uuid::Uuid),
    #[error("archive delete blocked: {0}")]
    ArchiveDeleteBlocked(String),
    #[error("failed to create directory `{0}`")]
    CreateDirectory(PathBuf),
    #[error("path has no file name: `{0}`")]
    MissingFileName(PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Polars(#[from] polars::error::PolarsError),
    #[error(transparent)]
    Storage(#[from] grand_edge_storage::StorageError),
}
