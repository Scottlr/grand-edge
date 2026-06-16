use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Gp, ItemId, PositionId, Quantity, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPosition {
    pub position_id: PositionId,
    pub user_id: UserId,
    pub item_id: ItemId,
    pub quantity: Quantity,
    pub avg_buy_price: Gp,
    pub bought_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}
