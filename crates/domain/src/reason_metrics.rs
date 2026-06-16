use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Gp, ModelVersion, Probability, Rate, ReasonType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasonOutcomeSummary {
    pub reason_type: ReasonType,
    pub reason_key: String,
    pub model_version: ModelVersion,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub sample_size: i64,
    pub win_rate: Option<Probability>,
    pub avg_actual_return: Option<Rate>,
    pub avg_net_gp: Option<Gp>,
    pub calibration_error: Option<f64>,
}
