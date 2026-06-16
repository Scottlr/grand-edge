use serde::{Deserialize, Serialize};

use crate::{ModelVersion, Rate, StrategyId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelAccuracySnapshot {
    pub strategy_id: StrategyId,
    pub model_version: ModelVersion,
    pub lookback_window: String,
    pub sample_size: i64,
    pub directional_accuracy: Option<Rate>,
    pub brier_score: Option<Rate>,
    pub avg_realized_roi: Option<Rate>,
    pub max_drawdown: Option<Rate>,
    pub calibration: serde_json::Value,
}
