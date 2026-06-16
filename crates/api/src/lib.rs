pub mod app;
pub mod config;
pub mod errors;
pub mod market;
pub mod model_accuracy;
pub mod openapi;
pub mod recommendations;
pub mod routes;
pub mod state;

use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = HealthResponse)))]
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/api/openapi.json",
    responses((status = 200, body = serde_json::Value))
)]
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(app::openapi_document())
}
