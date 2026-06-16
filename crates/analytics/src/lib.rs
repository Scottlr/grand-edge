//! Offline analytics and report exports for Grand Edge.

pub mod archive;
pub mod datasets;
pub mod edge_stability;
pub mod errors;
pub mod learned_edges;
pub mod manifest;
pub mod object_store;
pub mod parquet;
pub mod reports;
pub mod retention;

pub use archive::{
    ArchiveDataset, ArchiveFile, ArchiveJob, ArchiveManifest, ArchivePartition, ArchivePlan,
    ArchivePlanEntry, ArchiveSourceData, DeleteEligibility, fixture_archive_source_data,
    run_archive, run_archive_from_data,
};
pub use datasets::{
    DatasetExportData, DatasetExportRequest, DatasetExportResult, export_feature_dataset,
    export_feature_dataset_from_storage,
};
pub use edge_stability::edge_stability_score;
pub use errors::AnalyticsError;
pub use learned_edges::{
    LearnedEdgeCandidate, LearnedEdgeDiscoveryConfig, LearnedEdgeDiscoveryReport,
    LearnedEdgeDiscoveryRequest, LearnedEdgeStatistic, discover_fixture_edges,
    discover_learned_edges, granger_style_predictive_test, lead_lag_regression_score,
    persist_learned_edge_candidates, rolling_correlation,
};
pub use manifest::{ReportFile, ReportManifest, StrategyVersionRecord};
pub use object_store::{LocalFileObjectStore, ObjectStore};
pub use reports::{
    BacktestMetricsSummary, BacktestReportData, BacktestReportRequest, BacktestReportResult,
    export_backtest_report, export_backtest_report_from_storage,
};
pub use retention::RetentionPolicy;
