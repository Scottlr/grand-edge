//! Offline analytics and report exports for Grand Edge.

pub mod datasets;
pub mod errors;
pub mod manifest;
pub mod parquet;
pub mod reports;

pub use datasets::{
    DatasetExportData, DatasetExportRequest, DatasetExportResult, export_feature_dataset,
    export_feature_dataset_from_storage,
};
pub use errors::AnalyticsError;
pub use manifest::{ReportFile, ReportManifest, StrategyVersionRecord};
pub use reports::{
    BacktestMetricsSummary, BacktestReportData, BacktestReportRequest, BacktestReportResult,
    export_backtest_report, export_backtest_report_from_storage,
};
