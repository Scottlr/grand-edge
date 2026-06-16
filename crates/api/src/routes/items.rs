use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use grand_edge_domain::{Gp, Item, ItemIcon, ItemId, PriceInterval, WikiImageSource};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{errors::ApiError, state::AppState};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
pub enum IntervalDto {
    #[serde(rename = "5m")]
    FiveMinute,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "6h")]
    SixHour,
    #[serde(rename = "24h")]
    TwentyFourHour,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListItemsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct HistoryQuery {
    pub interval: IntervalDto,
    pub limit: Option<i64>,
    pub before: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemIconDto {
    pub source_file_name: String,
    pub canonical_file_name: String,
    pub cdn_url: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemDto {
    pub item_id: i64,
    pub name: String,
    pub examine: Option<String>,
    pub members: bool,
    pub buy_limit: Option<i32>,
    pub low_alch: Option<i64>,
    pub high_alch: Option<i64>,
    pub value: Option<i64>,
    pub icon: Option<ItemIconDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IntervalPriceDto {
    pub item_id: i64,
    pub bucket_start: DateTime<Utc>,
    pub interval: IntervalDto,
    pub avg_high_price: Option<i64>,
    pub high_price_volume: i64,
    pub avg_low_price: Option<i64>,
    pub low_price_volume: i64,
}

fn default_limit() -> i64 {
    50
}

#[utoipa::path(
    get,
    path = "/api/items",
    params(ListItemsQuery),
    responses((status = 200, body = [ItemDto]))
)]
pub async fn list_items(
    State(state): State<AppState>,
    Query(query): Query<ListItemsQuery>,
) -> Result<Json<Vec<ItemDto>>, ApiError> {
    let items = state.services.list_items(query.limit, query.offset).await?;
    Ok(Json(items.into_iter().map(ItemDto::from).collect()))
}

#[utoipa::path(
    get,
    path = "/api/items/{id}",
    params(("id" = i64, Path)),
    responses((status = 200, body = ItemDto), (status = 404))
)]
pub async fn get_item(
    State(state): State<AppState>,
    Path(item_id): Path<i64>,
) -> Result<Json<ItemDto>, ApiError> {
    let item_id = ItemId::try_from(item_id)?;
    let item = state
        .services
        .get_item(item_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("item {} was not found", item_id.0)))?;
    Ok(Json(ItemDto::from(item)))
}

#[utoipa::path(
    get,
    path = "/api/items/{id}/history",
    params(("id" = i64, Path), HistoryQuery),
    responses((status = 200, body = [IntervalPriceDto]), (status = 400))
)]
pub async fn get_item_history(
    State(state): State<AppState>,
    Path(item_id): Path<i64>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<IntervalPriceDto>>, ApiError> {
    let item_id = ItemId::try_from(item_id)?;
    let limit = query.limit.ok_or_else(|| {
        ApiError::BadRequest("history endpoint requires the limit query parameter".to_string())
    })?;
    if limit <= 0 {
        return Err(ApiError::BadRequest(
            "history limit must be positive".to_string(),
        ));
    }

    let rows = state
        .services
        .item_history(item_id, query.interval.into(), limit, query.before)
        .await?;
    Ok(Json(rows.into_iter().map(IntervalPriceDto::from).collect()))
}

impl From<Item> for ItemDto {
    fn from(value: Item) -> Self {
        Self {
            item_id: value.item_id.0,
            name: value.name,
            examine: value.examine,
            members: value.members,
            buy_limit: value.buy_limit,
            low_alch: value.low_alch.map(|gp| gp.0),
            high_alch: value.high_alch.map(|gp| gp.0),
            value: value.value.map(|gp| gp.0),
            icon: value.icon.map(ItemIconDto::from),
        }
    }
}

impl From<ItemIcon> for ItemIconDto {
    fn from(value: ItemIcon) -> Self {
        Self {
            source_file_name: value.source_file_name,
            canonical_file_name: value.canonical_file_name,
            cdn_url: value.cdn_url,
            source: match value.source {
                WikiImageSource::MappingIcon => "mapping_icon",
                WikiImageSource::HtmlSourceMatch => "html_source_match",
                WikiImageSource::Missing => "missing",
            }
            .to_string(),
        }
    }
}

impl From<grand_edge_domain::IntervalPrice> for IntervalPriceDto {
    fn from(value: grand_edge_domain::IntervalPrice) -> Self {
        Self {
            item_id: value.item_id.0,
            bucket_start: value.bucket_start,
            interval: value.interval.into(),
            avg_high_price: value.avg_high_price.map(Gp::as_i64),
            high_price_volume: value.high_price_volume,
            avg_low_price: value.avg_low_price.map(Gp::as_i64),
            low_price_volume: value.low_price_volume,
        }
    }
}

impl From<IntervalDto> for PriceInterval {
    fn from(value: IntervalDto) -> Self {
        match value {
            IntervalDto::FiveMinute => Self::FiveMinute,
            IntervalDto::OneHour => Self::OneHour,
            IntervalDto::SixHour => Self::SixHour,
            IntervalDto::TwentyFourHour => Self::TwentyFourHour,
        }
    }
}

impl From<PriceInterval> for IntervalDto {
    fn from(value: PriceInterval) -> Self {
        match value {
            PriceInterval::FiveMinute => Self::FiveMinute,
            PriceInterval::OneHour => Self::OneHour,
            PriceInterval::SixHour => Self::SixHour,
            PriceInterval::TwentyFourHour => Self::TwentyFourHour,
        }
    }
}
