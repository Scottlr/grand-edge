use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ItemId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureVector {
    pub item_id: ItemId,
    pub as_of: DateTime<Utc>,
    pub feature_set_version: String,
    pub values: serde_json::Map<String, serde_json::Value>,
}
