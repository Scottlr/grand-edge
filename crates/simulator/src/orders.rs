use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulatedOrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulatedOrderStatus {
    Created,
    Open,
    PartiallyFilled,
    Filled,
    Holding,
    ExitPending,
    Closed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedOrderRequest {
    pub run_id: Uuid,
    pub strategy_id: String,
    pub model_version: String,
    pub item_id: i64,
    pub created_at: DateTime<Utc>,
    pub side: SimulatedOrderSide,
    pub quantity: i64,
    pub limit_price: Option<i64>,
    pub target_exit: Option<i64>,
    pub stop_loss: Option<i64>,
    pub horizon_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperBetOutcome {
    pub bet_id: Uuid,
    pub strategy_id: String,
    pub item_id: i64,
    pub entry_time: DateTime<Utc>,
    pub entry_price: i64,
    pub quantity: i64,
    pub target_exit: Option<i64>,
    pub stop_loss: Option<i64>,
    pub exit_time: Option<DateTime<Utc>>,
    pub exit_price: Option<i64>,
    pub tax_paid: i64,
    pub realized_profit_gp: Option<i64>,
    pub realized_roi: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub hit_reason: Option<String>,
    pub status: SimulatedOrderStatus,
    pub explanation: serde_json::Value,
}
