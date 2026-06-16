use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestPrice {
    pub item_id: i64,
    pub high: Option<i64>,
    pub high_time: Option<DateTime<Utc>>,
    pub low: Option<i64>,
    pub low_time: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceInterval {
    FiveMinute,
    OneHour,
    SixHour,
    TwentyFourHour,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntervalPrice {
    pub item_id: i64,
    pub bucket_start: DateTime<Utc>,
    pub interval: PriceInterval,
    pub avg_high_price: Option<i64>,
    pub high_price_volume: i64,
    pub avg_low_price: Option<i64>,
    pub low_price_volume: i64,
}
