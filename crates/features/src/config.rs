use serde::{Deserialize, Serialize};

use crate::graph::GraphFeatureConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEngineConfig {
    pub rolling_window_5m: usize,
    pub rolling_window_1h: usize,
    pub ewma_lambda: f64,
    pub stale_after_secs: i64,
    pub graph_version: Option<String>,
    pub graph: GraphFeatureConfig,
}

impl Default for FeatureEngineConfig {
    fn default() -> Self {
        Self {
            rolling_window_5m: 12,
            rolling_window_1h: 24,
            ewma_lambda: 0.94,
            stale_after_secs: 900,
            graph_version: None,
            graph: GraphFeatureConfig::default(),
        }
    }
}
