use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyVersionRecord {
    pub strategy_id: String,
    pub model_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportFile {
    pub path: PathBuf,
    pub sha256: String,
    pub row_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportManifest {
    pub report_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub source_window_start: DateTime<Utc>,
    pub source_window_end: DateTime<Utc>,
    pub raw_candle_window_start: Option<DateTime<Utc>>,
    pub raw_candle_window_end: Option<DateTime<Utc>>,
    pub feature_set_version: String,
    pub strategy_versions: Vec<StrategyVersionRecord>,
    pub execution_modes: Vec<String>,
    pub files: Vec<ReportFile>,
}
