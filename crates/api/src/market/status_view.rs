use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataStateDto {
    Loading,
    Live,
    Stale,
    Degraded,
    Empty,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MarketStatusDto {
    pub data_state: DataStateDto,
    pub stale_reason: Option<String>,
    pub degraded_reason: Option<String>,
}
