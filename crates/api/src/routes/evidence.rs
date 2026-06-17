use axum::{
    Json,
    extract::{Path, State},
};
use grand_edge_domain::RecommendationId;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{errors::ApiError, evidence::view::RecommendationEvidenceDto, state::AppState};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct EvidenceErrorDto {
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/api/recommendations/{id}/evidence",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = RecommendationEvidenceDto), (status = 404, body = EvidenceErrorDto))
)]
pub async fn get_recommendation_evidence(
    State(state): State<AppState>,
    Path(recommendation_id): Path<Uuid>,
) -> Result<Json<RecommendationEvidenceDto>, ApiError> {
    let bundle = state
        .services
        .get_recommendation_evidence(RecommendationId(recommendation_id))
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "recommendation {} was not found",
                recommendation_id
            ))
        })?;

    Ok(Json(RecommendationEvidenceDto::from_record(
        bundle.record,
        bundle.item,
        bundle.reason_performance,
    )))
}
