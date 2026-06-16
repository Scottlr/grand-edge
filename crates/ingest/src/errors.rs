use grand_edge_domain::{DomainValidationError, ItemImageError};
use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error("invalid OSRS Wiki config: {0}")]
    InvalidConfig(String),
    #[error("unsupported latest interval: {0:?}")]
    UnsupportedInterval(grand_edge_domain::PriceInterval),
    #[error("unsupported timeseries interval: {0:?}")]
    UnsupportedTimeseriesInterval(grand_edge_domain::PriceInterval),
    #[error("invalid item id `{value}`: {source}")]
    InvalidItemId {
        value: i64,
        #[source]
        source: DomainValidationError,
    },
    #[error("invalid gp value for {field}: {value} ({source})")]
    InvalidGp {
        field: &'static str,
        value: i64,
        #[source]
        source: DomainValidationError,
    },
    #[error("invalid unix timestamp `{0}`")]
    InvalidTimestamp(i64),
    #[error("invalid latest item id key `{0}`")]
    InvalidLatestKey(String),
    #[error("http request failed")]
    Http(#[from] reqwest::Error),
    #[error("unexpected OSRS Wiki response status {status}")]
    UnexpectedStatus { status: StatusCode, body: String },
    #[error("storage operation failed")]
    Storage(#[from] grand_edge_storage::StorageError),
    #[error("graph domain validation failed")]
    GraphDomain(#[from] grand_edge_domain::GraphDomainError),
    #[error("wiki image normalization failed")]
    WikiImage(#[from] ItemImageError),
    #[error("filesystem operation failed")]
    Io(#[from] std::io::Error),
    #[error("json serialization or parsing failed")]
    Json(#[from] serde_json::Error),
    #[error("invalid relation corpus: {0}")]
    InvalidRelationCorpus(String),
    #[error("invalid market intelligence corpus: {0}")]
    InvalidMarketIntelligenceCorpus(String),
}
