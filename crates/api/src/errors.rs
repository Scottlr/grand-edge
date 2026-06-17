use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Config(String),
    #[error("storage error")]
    Storage(#[from] grand_edge_storage::StorageError),
    #[error("strategy error")]
    Strategy(#[from] grand_edge_strategies::StrategyError),
    #[error("simulator error")]
    Simulator(#[from] grand_edge_simulator::SimulatorError),
    #[error("metrics error")]
    Metrics(#[from] grand_edge_metrics::MetricsError),
    #[error("recommendation error")]
    Recommendation(#[from] grand_edge_recommender::RecommendationError),
    #[error("domain validation error")]
    Domain(#[from] grand_edge_domain::DomainValidationError),
    #[error("serialization error")]
    Serialization(#[from] serde_json::Error),
    #[error("{0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::BadRequest(_) | Self::Domain(_) | Self::Config(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Storage(_)
            | Self::Strategy(_)
            | Self::Simulator(_)
            | Self::Metrics(_)
            | Self::Recommendation(_)
            | Self::Serialization(_)
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = ErrorBody {
            error: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}
