//! Offline analytics and report exports for Grand Edge.

pub mod archive;
pub mod datasets;
pub mod errors;
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
pub use errors::AnalyticsError;
pub use manifest::{ReportFile, ReportManifest, StrategyVersionRecord};
pub use object_store::{LocalFileObjectStore, ObjectStore};
pub use reports::{
    BacktestMetricsSummary, BacktestReportData, BacktestReportRequest, BacktestReportResult,
    export_backtest_report, export_backtest_report_from_storage,
};
pub use retention::RetentionPolicy;
