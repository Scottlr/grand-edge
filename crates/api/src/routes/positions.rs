use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
use grand_edge_domain::{PositionId, UserPosition};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    auth::require_user_id,
    errors::ApiError,
    state::{AppState, PositionUpsert},
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPositionRequest {
    pub item_id: i64,
    pub quantity: i64,
    pub avg_buy_price: i64,
    pub bought_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PositionDto {
    pub position_id: Uuid,
    pub user_id: Uuid,
    pub item_id: i64,
    pub quantity: i64,
    pub avg_buy_price: i64,
    pub bought_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/users/me/positions",
    responses((status = 200, body = [PositionDto]))
)]
pub async fn list_positions(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<PositionDto>>, ApiError> {
    let user_id = require_user_id(&state, &jar).await?;
    let positions = state.services.list_positions(user_id).await?;
    Ok(Json(positions.into_iter().map(PositionDto::from).collect()))
}

#[utoipa::path(
    post,
    path = "/api/users/me/positions",
    request_body = UpsertPositionRequest,
    responses((status = 200, body = PositionDto))
)]
pub async fn create_position(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<UpsertPositionRequest>,
) -> Result<Json<PositionDto>, ApiError> {
    let user_id = require_user_id(&state, &jar).await?;
    let position = state
        .services
        .create_position(user_id, request.into())
        .await?;
    Ok(Json(PositionDto::from(position)))
}

#[utoipa::path(
    patch,
    path = "/api/users/me/positions/{id}",
    request_body = UpsertPositionRequest,
    params(("id" = Uuid, Path)),
    responses((status = 200, body = PositionDto), (status = 404))
)]
pub async fn update_position(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(position_id): Path<Uuid>,
    Json(request): Json<UpsertPositionRequest>,
) -> Result<Json<PositionDto>, ApiError> {
    let user_id = require_user_id(&state, &jar).await?;
    let position = state
        .services
        .update_position(user_id, PositionId(position_id), request.into())
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("position {} was not found", position_id)))?;
    Ok(Json(PositionDto::from(position)))
}

impl From<UpsertPositionRequest> for PositionUpsert {
    fn from(value: UpsertPositionRequest) -> Self {
        Self {
            item_id: value.item_id,
            quantity: value.quantity,
            avg_buy_price: value.avg_buy_price,
            bought_at: value.bought_at,
            notes: value.notes,
        }
    }
}

impl From<UserPosition> for PositionDto {
    fn from(value: UserPosition) -> Self {
        Self {
            position_id: value.position_id.0,
            user_id: value.user_id.0,
            item_id: value.item_id.0,
            quantity: value.quantity.0,
            avg_buy_price: value.avg_buy_price.0,
            bought_at: value.bought_at,
            notes: value.notes,
        }
    }
}
