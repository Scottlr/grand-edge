//! Accuracy and risk evaluation for Grand Edge.

pub mod calibration;
pub mod engine;
pub mod errors;
pub mod forecast;
pub mod graph;
pub mod reason_metrics;
pub mod recommendation_outcomes;
pub mod risk;
pub mod trading;
pub mod windows;

pub use calibration::{
    CalibrationBucket, CalibrationMetrics, ExecutionQualityMetrics, LiquidityBucketMetric,
};
pub use engine::{MetricsEngine, StrategyMetricSummary};
pub use errors::MetricsError;
pub use forecast::ForecastMetrics;
pub use graph::{
    BlastRadiusMetricSummary, BlastRadiusOutcome, GraphMetricDimension, GraphPathMetricSummary,
    GraphPathOutcome, blast_impact_mae, execution_mode_key, graph_confidence_calibration_error,
    graph_metric_dimension, graph_path_hit_rate, summarize_blast_radius_outcomes,
    summarize_graph_path_outcomes,
};
pub use reason_metrics::{
    ReasonMetricsWindow, ReasonOutcomeInput, compute_reason_outcome_summaries,
    refresh_reason_outcomes,
};
pub use recommendation_outcomes::{
    ActionOutcomeRule, EvaluationPriceMode, OutcomeEvaluationConfig, OutcomeEvaluationJob,
    OutcomeEvaluationResult, evaluate_due_recommendations, evaluate_recommendation_outcome,
};
pub use risk::{OverfitRiskMetrics, RiskAdjustedMetrics};
pub use trading::TradingMetrics;
pub use windows::MetricWindow;
