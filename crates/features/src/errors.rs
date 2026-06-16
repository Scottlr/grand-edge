#[derive(Debug, thiserror::Error)]
pub enum FeatureError {
    #[error("item {0} was missing from storage during feature generation")]
    MissingItem(i64),
    #[error("storage operation failed")]
    Storage(#[from] grand_edge_storage::StorageError),
}
