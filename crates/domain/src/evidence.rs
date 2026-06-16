use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ItemId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureSnapshot {
    pub feature_snapshot_id: Uuid,
    pub item_id: ItemId,
    pub as_of: DateTime<Utc>,
    pub feature_set_version: String,
    #[serde(default)]
    pub graph_version: Option<String>,
    pub source_window_start: DateTime<Utc>,
    pub source_window_end: DateTime<Utc>,
    pub features: serde_json::Map<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use crate::{FeatureSnapshot, ItemId};

    #[test]
    fn feature_snapshot_defaults_graph_version_for_non_graph_records() {
        let payload = serde_json::json!({
            "feature_snapshot_id": Uuid::nil(),
            "item_id": 4151,
            "as_of": "2026-06-16T12:00:00Z",
            "feature_set_version": "features_v1",
            "source_window_start": "2026-06-15T12:00:00Z",
            "source_window_end": "2026-06-16T12:00:00Z",
            "features": {},
            "created_at": "2026-06-16T12:00:00Z"
        });

        let snapshot: FeatureSnapshot = serde_json::from_value(payload).unwrap();
        assert_eq!(snapshot.item_id, ItemId(4151));
        assert_eq!(snapshot.graph_version, None);
        assert_eq!(
            snapshot.as_of,
            Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap()
        );
    }
}
