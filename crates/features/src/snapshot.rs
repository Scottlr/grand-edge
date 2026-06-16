use chrono::{DateTime, Utc};
use grand_edge_domain::{IntervalPrice, Item, ItemGraphEdge, LatestPrice};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborPriceHistory {
    pub edge: ItemGraphEdge,
    pub interval_1h: Vec<IntervalPrice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphFeatureContext {
    pub graph_version: String,
    pub incoming_neighbors: Vec<NeighborPriceHistory>,
    pub outgoing_neighbors: Vec<NeighborPriceHistory>,
    pub sector_neighbors: Vec<NeighborPriceHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemFeatureInput {
    pub item: Item,
    pub latest: LatestPrice,
    pub interval_5m: Vec<IntervalPrice>,
    pub interval_1h: Vec<IntervalPrice>,
    pub as_of: DateTime<Utc>,
    pub graph_context: Option<GraphFeatureContext>,
}
