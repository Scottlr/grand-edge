use std::convert::Infallible;

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{recommendations::view::RecommendationActionDto, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveEvent {
    PriceUpdated {
        item_id: i64,
        observed_at: DateTime<Utc>,
    },
    RecommendationUpdated {
        recommendation_id: Uuid,
        item_id: i64,
        action: RecommendationActionDto,
    },
    SimulationUpdated {
        run_id: Uuid,
        status: String,
    },
    StrategyConfigUpdated {
        strategy_id: String,
        enabled: bool,
    },
}

#[utoipa::path(
    get,
    path = "/api/live/stream",
    responses((status = 200, body = LiveEvent))
)]
pub async fn stream_live_events(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream =
        BroadcastStream::new(state.live_events.subscribe()).filter_map(|result| match result {
            Ok(event) => Some(Ok(Event::default()
                .event("live_event")
                .json_data(event)
                .expect("live event should serialize"))),
            Err(_) => None,
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
