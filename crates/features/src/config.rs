#[derive(Debug, Clone)]
pub struct FeatureEngineConfig {
    pub rolling_window_5m: usize,
    pub rolling_window_1h: usize,
    pub ewma_lambda: f64,
    pub stale_after_secs: i64,
}

impl Default for FeatureEngineConfig {
    fn default() -> Self {
        Self {
            rolling_window_5m: 12,
            rolling_window_1h: 24,
            ewma_lambda: 0.94,
            stale_after_secs: 900,
        }
    }
}
