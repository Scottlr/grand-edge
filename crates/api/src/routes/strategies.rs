use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{errors::ApiError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyStatusDto {
    pub strategy_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PatchStrategyRequest {
    pub enabled: bool,
}

#[utoipa::path(
    get,
    path = "/api/strategies",
    responses((status = 200, body = [StrategyStatusDto]))
)]
pub async fn list_strategies(
    State(state): State<AppState>,
) -> Result<Json<Vec<StrategyStatusDto>>, ApiError> {
    let strategies = state.services.list_strategies().await?;
    Ok(Json(
        strategies
            .into_iter()
            .map(|record| StrategyStatusDto {
                strategy_id: record.strategy_id,
                enabled: record.enabled,
            })
            .collect(),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/strategies/{id}",
    request_body = PatchStrategyRequest,
    params(("id" = String, Path)),
    responses((status = 200, body = StrategyStatusDto), (status = 404))
)]
pub async fn patch_strategy(
    State(state): State<AppState>,
    Path(strategy_id): Path<String>,
    Json(request): Json<PatchStrategyRequest>,
) -> Result<Json<StrategyStatusDto>, ApiError> {
    let strategy = state
        .services
        .patch_strategy(&strategy_id, request.enabled)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("unknown strategy id `{strategy_id}`")))?;

    state
        .live_events
        .publish(crate::routes::live::LiveEvent::StrategyConfigUpdated {
            strategy_id: strategy.strategy_id.clone(),
            enabled: strategy.enabled,
        });

    Ok(Json(StrategyStatusDto {
        strategy_id: strategy.strategy_id,
        enabled: strategy.enabled,
    }))
}
