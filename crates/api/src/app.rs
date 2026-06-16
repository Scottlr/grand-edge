use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::{get, patch, post},
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::{OpenApi, ToSchema};

use crate::{openapi::ApiDoc, routes, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

pub fn build_router(state: AppState, cors_origin: Option<String>) -> Router {
    let cors = if let Some(origin) = cors_origin {
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PATCH])
            .allow_origin(
                origin
                    .parse::<HeaderValue>()
                    .expect("valid cors origin header value"),
            )
    } else {
        CorsLayer::permissive()
    };

    Router::new()
        .route("/health", get(crate::health))
        .route("/api/items", get(routes::items::list_items))
        .route("/api/items/{id}", get(routes::items::get_item))
        .route(
            "/api/items/{id}/history",
            get(routes::items::get_item_history),
        )
        .route(
            "/api/recommendations",
            get(routes::recommendations::list_recommendations),
        )
        .route(
            "/api/recommendations/{id}/explanation",
            get(routes::recommendations::get_recommendation_explanation),
        )
        .route("/api/strategies", get(routes::strategies::list_strategies))
        .route(
            "/api/strategies/{id}",
            patch(routes::strategies::patch_strategy),
        )
        .route(
            "/api/simulations",
            get(routes::simulations::list_simulations),
        )
        .route(
            "/api/simulations",
            post(routes::simulations::create_simulation),
        )
        .route(
            "/api/users/me/positions",
            get(routes::positions::list_positions).post(routes::positions::create_position),
        )
        .route(
            "/api/users/me/positions/{id}",
            patch(routes::positions::update_position),
        )
        .route("/api/live/stream", get(routes::live::stream_live_events))
        .route("/api/openapi.json", get(crate::openapi_json))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

pub fn openapi_document() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
