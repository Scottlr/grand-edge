use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureVector {
    pub item_id: i64,
    pub as_of: DateTime<Utc>,
    pub feature_set_version: String,
    pub values: serde_json::Map<String, serde_json::Value>,
}
