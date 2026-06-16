use grand_edge_domain::{ModelAccuracySnapshot, Rate};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelAccuracySummaryDto {
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

impl From<ModelAccuracySnapshot> for ModelAccuracySummaryDto {
    fn from(value: ModelAccuracySnapshot) -> Self {
        Self {
            strategy_id: value.strategy_id.0,
            model_version: value.model_version.0,
            lookback_window: value.lookback_window,
            sample_size: value.sample_size,
            directional_accuracy: value.directional_accuracy.map(Rate::get),
            brier_score: value.brier_score.map(Rate::get),
            avg_realized_roi: value.avg_realized_roi.map(Rate::get),
            max_drawdown: value.max_drawdown.map(Rate::get),
            calibration: value.calibration,
        }
    }
}
