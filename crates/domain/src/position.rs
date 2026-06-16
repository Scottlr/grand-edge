use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPosition {
    pub position_id: Uuid,
    pub user_id: Uuid,
    pub item_id: i64,
    pub quantity: i64,
    pub avg_buy_price: i64,
    pub bought_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}
