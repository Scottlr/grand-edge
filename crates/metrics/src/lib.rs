//! Accuracy and risk evaluation for Grand Edge.

pub mod calibration;
pub mod engine;
pub mod errors;
pub mod forecast;
pub mod risk;
pub mod trading;
pub mod windows;

pub use calibration::{
    CalibrationBucket, CalibrationMetrics, ExecutionQualityMetrics, LiquidityBucketMetric,
};
pub use engine::{MetricsEngine, StrategyMetricSummary};
pub use errors::MetricsError;
pub use forecast::ForecastMetrics;
pub use risk::{OverfitRiskMetrics, RiskAdjustedMetrics};
pub use trading::TradingMetrics;
pub use windows::MetricWindow;
