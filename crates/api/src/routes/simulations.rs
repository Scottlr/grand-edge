use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    errors::ApiError,
    routes::live::LiveEvent,
    state::{AppState, SimulationRunDraft},
};

#[derive(Debug, Deserialize, IntoParams)]
pub struct SimulationListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSimulationRequest {
    pub name: String,
    #[serde(default)]
    pub strategy_config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SimulationRunDto {
    pub run_id: Uuid,
    pub name: String,
    pub strategy_config: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
}

fn default_limit() -> i64 {
    50
}

#[utoipa::path(
    get,
    path = "/api/simulations",
    params(SimulationListQuery),
    responses((status = 200, body = [SimulationRunDto]))
)]
pub async fn list_simulations(
    State(state): State<AppState>,
    Query(query): Query<SimulationListQuery>,
) -> Result<Json<Vec<SimulationRunDto>>, ApiError> {
    let runs = state
        .services
        .list_simulation_runs(query.limit, query.offset)
        .await?;
    Ok(Json(
        runs.into_iter()
            .map(|run| SimulationRunDto::from(&run))
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/simulations",
    request_body = CreateSimulationRequest,
    responses((status = 200, body = SimulationRunDto))
)]
pub async fn create_simulation(
    State(state): State<AppState>,
    Json(request): Json<CreateSimulationRequest>,
) -> Result<Json<SimulationRunDto>, ApiError> {
    let run = state
        .services
        .create_simulation_run(SimulationRunDraft {
            name: request.name,
            strategy_config: request.strategy_config,
        })
        .await?;

    state.live_events.publish(LiveEvent::SimulationUpdated {
        run_id: run.run_id.0,
        status: run.status.clone(),
    });

    Ok(Json(SimulationRunDto::from(&run)))
}

impl From<&grand_edge_storage::StoredSimulationRun> for SimulationRunDto {
    fn from(value: &grand_edge_storage::StoredSimulationRun) -> Self {
        Self {
            run_id: value.run_id.0,
            name: value.name.clone(),
            strategy_config: value.strategy_config.clone(),
            started_at: value.started_at,
            finished_at: value.finished_at,
            status: value.status.clone(),
        }
    }
}
