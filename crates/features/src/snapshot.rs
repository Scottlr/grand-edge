use chrono::{DateTime, Utc};
use grand_edge_domain::{IntervalPrice, Item, LatestPrice};

#[derive(Debug, Clone)]
pub struct ItemFeatureInput {
    pub item: Item,
    pub latest: LatestPrice,
    pub interval_5m: Vec<IntervalPrice>,
    pub interval_1h: Vec<IntervalPrice>,
    pub as_of: DateTime<Utc>,
}
