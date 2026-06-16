use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Gp, ItemId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestPrice {
    pub item_id: ItemId,
    pub high: Option<Gp>,
    pub high_time: Option<DateTime<Utc>>,
    pub low: Option<Gp>,
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
    pub item_id: ItemId,
    pub bucket_start: DateTime<Utc>,
    pub interval: PriceInterval,
    pub avg_high_price: Option<Gp>,
    pub high_price_volume: i64,
    pub avg_low_price: Option<Gp>,
    pub low_price_volume: i64,
}
