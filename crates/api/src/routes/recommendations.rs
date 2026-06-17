use axum::{
    Json,
    extract::{Path, Query, State},
};
use axum_extra::extract::CookieJar;
use grand_edge_domain::{RecommendationAction, RecommendationId};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::recommendations::view::{RecommendationActionDto, RecommendationDto};
use crate::{auth::request_user_id, errors::ApiError, state::AppState};

#[derive(Debug, Deserialize, IntoParams)]
pub struct RecommendationQuery {
    pub action: Option<RecommendationActionDto>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[utoipa::path(
    get,
    path = "/api/recommendations",
    params(RecommendationQuery),
    responses((status = 200, body = [RecommendationDto]))
)]
pub async fn list_recommendations(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<RecommendationQuery>,
) -> Result<Json<Vec<RecommendationDto>>, ApiError> {
    let user_id = request_user_id(&state, &jar).await?;
    let recommendations = state
        .services
        .list_recommendations(
            user_id,
            query.action.map(RecommendationAction::from),
            query.limit,
            query.offset,
        )
        .await?;

    let mut payload = Vec::with_capacity(recommendations.len());
    for recommendation in recommendations {
        let item = state.services.get_item(recommendation.item_id).await?;
        payload.push(RecommendationDto::from_parts(recommendation, item));
    }

    Ok(Json(payload))
}

#[utoipa::path(
    get,
    path = "/api/recommendations/{id}/explanation",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = RecommendationDto), (status = 404))
)]
pub async fn get_recommendation_explanation(
    State(state): State<AppState>,
    Path(recommendation_id): Path<Uuid>,
) -> Result<Json<RecommendationDto>, ApiError> {
    let recommendation = state
        .services
        .get_recommendation_explanation(RecommendationId(recommendation_id))
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "recommendation {} was not found",
                recommendation_id
            ))
        })?;

    let item = state.services.get_item(recommendation.item_id).await?;
    Ok(Json(RecommendationDto::from_parts(recommendation, item)))
}
