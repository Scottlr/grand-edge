#[derive(Debug, thiserror::Error)]
pub enum SimulatorError {
    #[error("storage operation failed")]
    Storage(#[from] grand_edge_storage::StorageError),
    #[error("invalid simulation request: {0}")]
    InvalidRequest(String),
    #[error("no fill could be produced for request")]
    NoFill,
    #[error("no exit could be produced after fill")]
    NoExit,
}
