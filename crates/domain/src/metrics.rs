use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelAccuracySnapshot {
    pub strategy_id: String,
    pub model_version: String,
    pub lookback_window: String,
    pub sample_size: i64,
    pub directional_accuracy: Option<f64>,
    pub brier_score: Option<f64>,
    pub avg_realized_roi: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub calibration: serde_json::Value,
}
