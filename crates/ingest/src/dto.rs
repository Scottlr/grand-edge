use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MappingItemRaw {
    pub examine: Option<String>,
    pub id: i64,
    pub members: bool,
    pub lowalch: Option<i64>,
    pub limit: Option<i32>,
    pub value: Option<i64>,
    pub highalch: Option<i64>,
    pub icon: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LatestResponseRaw {
    pub data: HashMap<String, LatestPriceRaw>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestPriceRaw {
    pub high: Option<i64>,
    pub high_time: Option<i64>,
    pub low: Option<i64>,
    pub low_time: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntervalBulkResponseRaw {
    pub data: HashMap<String, IntervalPriceRaw>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntervalPriceRaw {
    pub avg_high_price: Option<i64>,
    pub high_price_volume: Option<i64>,
    pub avg_low_price: Option<i64>,
    pub low_price_volume: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeseriesResponseRaw {
    pub data: Vec<TimeseriesPriceRaw>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeseriesPriceRaw {
    pub timestamp: i64,
    pub avg_high_price: Option<i64>,
    pub avg_low_price: Option<i64>,
    pub high_price_volume: Option<i64>,
    pub low_price_volume: Option<i64>,
}
