use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use grand_edge_domain::{
    EdgeObservation, GraphVersion, IntervalPrice, ItemGraphEdge, MarketEventNode, PriceInterval,
    ReasonOutcomeSummary,
};
use grand_edge_storage::{
    EvaluatedRecommendationRecord, Storage, StoredMarketEvent, StoredPrediction,
};
use polars::prelude::{Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::object_store::ObjectStore;
use crate::parquet::{parquet_bytes, sha256_bytes};
use crate::{AnalyticsError, RetentionPolicy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveDataset {
    IntervalPrices,
    FeatureSnapshots,
    Predictions,
    Recommendations,
    RecommendationOutcomes,
    ReasonOutcomes,
    StrategySummaries,
    ModelCards,
    GraphVersions,
    ItemEdges,
    EdgeObservations,
    MarketEvents,
    BlastSimulations,
    GraphPathMetrics,
}

impl ArchiveDataset {
    fn directory_name(&self) -> &'static str {
        match self {
            Self::IntervalPrices => "interval_prices",
            Self::FeatureSnapshots => "feature_snapshots",
            Self::Predictions => "predictions",
            Self::Recommendations => "recommendations",
            Self::RecommendationOutcomes => "recommendation_outcomes",
            Self::ReasonOutcomes => "reason_outcomes",
            Self::StrategySummaries => "strategy_summaries",
            Self::ModelCards => "model_cards",
            Self::GraphVersions => "graph_versions",
            Self::ItemEdges => "item_edges",
            Self::EdgeObservations => "edge_observations",
            Self::MarketEvents => "market_events",
            Self::BlastSimulations => "blast_simulations",
            Self::GraphPathMetrics => "graph_path_metrics",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivePartition {
    pub dataset: ArchiveDataset,
    pub interval: Option<String>,
    pub date: NaiveDate,
    pub model_id: Option<String>,
    pub model_version: Option<String>,
    pub graph_version: Option<String>,
    pub method: Option<String>,
    pub window_end: Option<NaiveDate>,
}

impl ArchivePartition {
    pub fn object_path(&self) -> PathBuf {
        let mut path = PathBuf::from(self.dataset.directory_name());
        if let Some(interval) = &self.interval {
            path.push(format!("interval={interval}"));
        }
        if let Some(model_id) = &self.model_id {
            path.push(format!("model_id={model_id}"));
        }
        if let Some(model_version) = &self.model_version {
            path.push(format!("model_version={model_version}"));
        }
        if let Some(graph_version) = &self.graph_version {
            path.push(format!("graph_version={graph_version}"));
        }
        if let Some(method) = &self.method {
            path.push(format!("method={method}"));
        }
        if let Some(window_end) = self.window_end {
            path.push(format!("window_end={window_end}"));
        } else {
            path.push(format!("date={}", self.date));
        }
        path.push("part-000.parquet.zst");
        path
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveFile {
    pub path: PathBuf,
    pub row_count: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub manifest_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub policy: RetentionPolicy,
    pub partitions: Vec<ArchivePartition>,
    pub files: Vec<ArchiveFile>,
    pub file_format: String,
    pub compression: String,
    pub row_count: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveJob {
    pub as_of: DateTime<Utc>,
    pub policy: RetentionPolicy,
    pub dry_run: bool,
    pub allow_hot_delete: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteEligibility {
    pub eligible: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchivePlanEntry {
    pub partition: ArchivePartition,
    pub path: PathBuf,
    pub row_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchivePlan {
    pub job: ArchiveJob,
    pub entries: Vec<ArchivePlanEntry>,
    pub delete_eligibility: DeleteEligibility,
}

#[derive(Debug, Clone, Default)]
pub struct ArchiveSourceData {
    pub interval_prices: Vec<IntervalPrice>,
    pub predictions: Vec<StoredPrediction>,
    pub outcomes: Vec<EvaluatedRecommendationRecord>,
    pub reason_outcomes: Vec<ReasonOutcomeSummary>,
    pub graph_versions: Vec<GraphVersion>,
    pub item_edges: Vec<ItemGraphEdge>,
    pub edge_observations: Vec<EdgeObservation>,
    pub market_events: Vec<StoredMarketEvent>,
    pub blast_simulations: Vec<serde_json::Value>,
    pub graph_path_metrics: Vec<serde_json::Value>,
    pub outcome_summaries_present: bool,
    pub reason_summaries_present: bool,
}

pub async fn plan_archive(
    _storage: &Storage,
    job: &ArchiveJob,
) -> Result<ArchivePlan, AnalyticsError> {
    plan_archive_from_data(job, &ArchiveSourceData::default())
}

pub async fn run_archive(
    storage: &Storage,
    object_store: &dyn ObjectStore,
    job: ArchiveJob,
) -> Result<ArchiveManifest, AnalyticsError> {
    let _ = storage;
    run_archive_from_data(object_store, job, ArchiveSourceData::default())
}

pub fn plan_archive_from_data(
    job: &ArchiveJob,
    data: &ArchiveSourceData,
) -> Result<ArchivePlan, AnalyticsError> {
    job.policy.validate()?;
    let entries = build_plan_entries(job, data);
    Ok(ArchivePlan {
        job: job.clone(),
        delete_eligibility: delete_eligibility(
            job,
            data,
            false,
            entries.iter().all(|entry| entry.row_count > 0),
        ),
        entries,
    })
}

pub fn run_archive_from_data(
    object_store: &dyn ObjectStore,
    job: ArchiveJob,
    data: ArchiveSourceData,
) -> Result<ArchiveManifest, AnalyticsError> {
    let plan = plan_archive_from_data(&job, &data)?;
    let files = write_archive_files(object_store, &plan.entries, &data)?;
    let total_rows = files.iter().map(|file| file.row_count).sum();
    let manifest = ArchiveManifest {
        manifest_id: Uuid::new_v4(),
        created_at: Utc::now(),
        policy: job.policy.clone(),
        partitions: plan
            .entries
            .iter()
            .map(|entry| entry.partition.clone())
            .collect(),
        row_count: total_rows,
        checksum: aggregate_checksum(&files),
        file_format: "parquet".to_string(),
        compression: "zstd".to_string(),
        files,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    object_store.put(Path::new("manifest.json"), &manifest_bytes)?;

    let delete = delete_eligibility(
        &job,
        &data,
        true,
        manifest
            .files
            .iter()
            .all(|file| file.row_count > 0 && !file.sha256.is_empty()),
    );
    if !job.dry_run && job.allow_hot_delete && !delete.eligible {
        return Err(AnalyticsError::ArchiveDeleteBlocked(
            delete.blockers.join("; "),
        ));
    }

    Ok(manifest)
}

fn build_plan_entries(job: &ArchiveJob, data: &ArchiveSourceData) -> Vec<ArchivePlanEntry> {
    let mut entries = Vec::new();
    let date = job.as_of.date_naive();

    for interval in distinct_intervals(&data.interval_prices) {
        let row_count = data
            .interval_prices
            .iter()
            .filter(|row| row.interval == interval)
            .count() as u64;
        entries.push(entry(
            ArchivePartition {
                dataset: ArchiveDataset::IntervalPrices,
                interval: Some(interval_key(interval).to_string()),
                date,
                model_id: None,
                model_version: None,
                graph_version: None,
                method: None,
                window_end: None,
            },
            row_count,
        ));
    }

    for (model_id, model_version) in distinct_prediction_models(&data.predictions) {
        let row_count = data
            .predictions
            .iter()
            .filter(|row| row.strategy_id.0 == model_id && row.model_version.0 == model_version)
            .count() as u64;
        entries.push(entry(
            ArchivePartition {
                dataset: ArchiveDataset::Predictions,
                interval: None,
                date,
                model_id: Some(model_id),
                model_version: Some(model_version),
                graph_version: None,
                method: None,
                window_end: None,
            },
            row_count,
        ));
    }

    entries.push(entry(
        ArchivePartition {
            dataset: ArchiveDataset::RecommendationOutcomes,
            interval: None,
            date,
            model_id: None,
            model_version: None,
            graph_version: None,
            method: None,
            window_end: None,
        },
        data.outcomes.len() as u64,
    ));

    for summary in &data.reason_outcomes {
        entries.push(entry(
            ArchivePartition {
                dataset: ArchiveDataset::ReasonOutcomes,
                interval: None,
                date,
                model_id: None,
                model_version: Some(summary.model_version.0.clone()),
                graph_version: None,
                method: None,
                window_end: Some(summary.window_end.date_naive()),
            },
            1,
        ));
    }

    for version in &data.graph_versions {
        entries.push(entry(
            ArchivePartition {
                dataset: ArchiveDataset::GraphVersions,
                interval: None,
                date,
                model_id: None,
                model_version: None,
                graph_version: Some(version.graph_version.clone()),
                method: None,
                window_end: None,
            },
            1,
        ));
    }

    for graph_version in distinct_edge_graph_versions(&data.item_edges) {
        let row_count = data
            .item_edges
            .iter()
            .filter(|edge| edge.graph_version == graph_version)
            .count() as u64;
        entries.push(entry(
            ArchivePartition {
                dataset: ArchiveDataset::ItemEdges,
                interval: None,
                date,
                model_id: None,
                model_version: None,
                graph_version: Some(graph_version),
                method: None,
                window_end: None,
            },
            row_count,
        ));
    }

    for method in distinct_observation_methods(&data.edge_observations) {
        let row_count = data
            .edge_observations
            .iter()
            .filter(|observation| observation_method_key(observation) == method)
            .count() as u64;
        entries.push(entry(
            ArchivePartition {
                dataset: ArchiveDataset::EdgeObservations,
                interval: None,
                date,
                model_id: None,
                model_version: None,
                graph_version: None,
                method: Some(method),
                window_end: None,
            },
            row_count,
        ));
    }

    for graph_version in distinct_market_event_graph_versions(&data.market_events) {
        let row_count = data
            .market_events
            .iter()
            .filter(|event| event.event.graph_version == graph_version)
            .count() as u64;
        entries.push(entry(
            ArchivePartition {
                dataset: ArchiveDataset::MarketEvents,
                interval: None,
                date,
                model_id: None,
                model_version: None,
                graph_version: Some(graph_version),
                method: None,
                window_end: None,
            },
            row_count,
        ));
    }

    entries.push(entry(
        ArchivePartition {
            dataset: ArchiveDataset::BlastSimulations,
            interval: None,
            date,
            model_id: None,
            model_version: None,
            graph_version: first_graph_version(
                &data.graph_versions,
                &data.item_edges,
                &data.market_events,
            ),
            method: None,
            window_end: None,
        },
        data.blast_simulations.len() as u64,
    ));
    entries.push(entry(
        ArchivePartition {
            dataset: ArchiveDataset::GraphPathMetrics,
            interval: None,
            date,
            model_id: None,
            model_version: None,
            graph_version: first_graph_version(
                &data.graph_versions,
                &data.item_edges,
                &data.market_events,
            ),
            method: None,
            window_end: None,
        },
        data.graph_path_metrics.len() as u64,
    ));

    entries
}

fn write_archive_files(
    object_store: &dyn ObjectStore,
    entries: &[ArchivePlanEntry],
    data: &ArchiveSourceData,
) -> Result<Vec<ArchiveFile>, AnalyticsError> {
    let mut files = Vec::new();
    for entry in entries {
        let mut frame = frame_for_entry(entry, data)?;
        let bytes = parquet_bytes(&mut frame)?;
        object_store.put(&entry.path, &bytes)?;
        files.push(ArchiveFile {
            path: entry.path.clone(),
            row_count: entry.row_count,
            sha256: sha256_bytes(&bytes),
        });
    }
    Ok(files)
}

fn frame_for_entry(
    entry: &ArchivePlanEntry,
    data: &ArchiveSourceData,
) -> Result<DataFrame, AnalyticsError> {
    match entry.partition.dataset {
        ArchiveDataset::IntervalPrices => interval_prices_frame(
            &data
                .interval_prices
                .iter()
                .filter(|row| {
                    Some(interval_key(row.interval).to_string()) == entry.partition.interval
                })
                .cloned()
                .collect::<Vec<_>>(),
        ),
        ArchiveDataset::Predictions => predictions_frame(
            &data
                .predictions
                .iter()
                .filter(|row| {
                    Some(row.strategy_id.0.clone()) == entry.partition.model_id
                        && Some(row.model_version.0.clone()) == entry.partition.model_version
                })
                .cloned()
                .collect::<Vec<_>>(),
        ),
        ArchiveDataset::RecommendationOutcomes => outcomes_frame(&data.outcomes),
        ArchiveDataset::ReasonOutcomes => reason_outcomes_frame(
            &data
                .reason_outcomes
                .iter()
                .filter(|row| {
                    Some(row.model_version.0.clone()) == entry.partition.model_version
                        && Some(row.window_end.date_naive()) == entry.partition.window_end
                })
                .cloned()
                .collect::<Vec<_>>(),
        ),
        ArchiveDataset::GraphVersions => graph_versions_frame(
            &data
                .graph_versions
                .iter()
                .filter(|row| Some(row.graph_version.clone()) == entry.partition.graph_version)
                .cloned()
                .collect::<Vec<_>>(),
        ),
        ArchiveDataset::ItemEdges => item_edges_frame(
            &data
                .item_edges
                .iter()
                .filter(|row| Some(row.graph_version.clone()) == entry.partition.graph_version)
                .cloned()
                .collect::<Vec<_>>(),
        ),
        ArchiveDataset::EdgeObservations => edge_observations_frame(
            &data
                .edge_observations
                .iter()
                .filter(|row| Some(observation_method_key(row)) == entry.partition.method)
                .cloned()
                .collect::<Vec<_>>(),
        ),
        ArchiveDataset::MarketEvents => market_events_frame(
            &data
                .market_events
                .iter()
                .filter(|row| {
                    Some(row.event.graph_version.clone()) == entry.partition.graph_version
                })
                .cloned()
                .collect::<Vec<_>>(),
        ),
        ArchiveDataset::BlastSimulations => json_values_frame("payload", &data.blast_simulations),
        ArchiveDataset::GraphPathMetrics => json_values_frame("payload", &data.graph_path_metrics),
        _ => json_values_frame("payload", &[]),
    }
}

fn delete_eligibility(
    job: &ArchiveJob,
    data: &ArchiveSourceData,
    manifest_written: bool,
    archive_rows_present: bool,
) -> DeleteEligibility {
    let mut blockers = Vec::new();
    if job.dry_run {
        blockers.push("dry_run is enabled".to_string());
    }
    if !job.allow_hot_delete {
        blockers.push("allow_hot_delete is false".to_string());
    }
    if !manifest_written {
        blockers.push("manifest checksum has not been written".to_string());
    }
    if !archive_rows_present {
        blockers.push("archive files are missing or empty".to_string());
    }
    if !data.outcome_summaries_present {
        blockers.push("outcome summaries are missing for the archive window".to_string());
    }
    if !data.reason_summaries_present {
        blockers.push("reason summaries are missing for the archive window".to_string());
    }

    DeleteEligibility {
        eligible: blockers.is_empty(),
        blockers,
    }
}

fn aggregate_checksum(files: &[ArchiveFile]) -> String {
    let joined = files
        .iter()
        .map(|file| format!("{}:{}:{}", file.path.display(), file.row_count, file.sha256))
        .collect::<Vec<_>>()
        .join("|");
    sha256_bytes(joined.as_bytes())
}

fn entry(partition: ArchivePartition, row_count: u64) -> ArchivePlanEntry {
    let path = partition.object_path();
    ArchivePlanEntry {
        partition,
        path,
        row_count,
    }
}

fn distinct_intervals(rows: &[IntervalPrice]) -> Vec<PriceInterval> {
    let mut keys = BTreeSet::new();
    for row in rows {
        keys.insert(interval_key(row.interval));
    }
    keys.into_iter().filter_map(parse_interval_key).collect()
}

fn distinct_prediction_models(rows: &[StoredPrediction]) -> Vec<(String, String)> {
    let mut keys = BTreeSet::new();
    for row in rows {
        keys.insert((row.strategy_id.0.clone(), row.model_version.0.clone()));
    }
    keys.into_iter().collect()
}

fn distinct_edge_graph_versions(rows: &[ItemGraphEdge]) -> Vec<String> {
    let mut keys = BTreeSet::new();
    for row in rows {
        keys.insert(row.graph_version.clone());
    }
    keys.into_iter().collect()
}

fn distinct_market_event_graph_versions(rows: &[StoredMarketEvent]) -> Vec<String> {
    let mut keys = BTreeSet::new();
    for row in rows {
        keys.insert(row.event.graph_version.clone());
    }
    keys.into_iter().collect()
}

fn distinct_observation_methods(rows: &[EdgeObservation]) -> Vec<String> {
    let mut keys = BTreeSet::new();
    for row in rows {
        keys.insert(observation_method_key(row));
    }
    keys.into_iter().collect()
}

fn first_graph_version(
    versions: &[GraphVersion],
    edges: &[ItemGraphEdge],
    events: &[StoredMarketEvent],
) -> Option<String> {
    versions
        .first()
        .map(|value| value.graph_version.clone())
        .or_else(|| edges.first().map(|value| value.graph_version.clone()))
        .or_else(|| {
            events
                .first()
                .map(|value| value.event.graph_version.clone())
        })
}

fn interval_key(interval: PriceInterval) -> &'static str {
    match interval {
        PriceInterval::FiveMinute => "5m",
        PriceInterval::OneHour => "1h",
        PriceInterval::SixHour => "6h",
        PriceInterval::TwentyFourHour => "24h",
    }
}

fn parse_interval_key(value: &str) -> Option<PriceInterval> {
    match value {
        "5m" => Some(PriceInterval::FiveMinute),
        "1h" => Some(PriceInterval::OneHour),
        "6h" => Some(PriceInterval::SixHour),
        "24h" => Some(PriceInterval::TwentyFourHour),
        _ => None,
    }
}

fn observation_method_key(observation: &EdgeObservation) -> String {
    serde_json::to_string(&observation.method)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

fn interval_prices_frame(rows: &[IntervalPrice]) -> Result<DataFrame, AnalyticsError> {
    Ok(DataFrame::new(vec![
        Series::new(
            "item_id".into(),
            rows.iter().map(|row| row.item_id.0).collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "bucket_start".into(),
            rows.iter()
                .map(|row| row.bucket_start.to_rfc3339())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "interval".into(),
            rows.iter()
                .map(|row| interval_key(row.interval).to_string())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "avg_high_price".into(),
            rows.iter()
                .map(|row| row.avg_high_price.map(|value| value.0))
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "high_price_volume".into(),
            rows.iter()
                .map(|row| row.high_price_volume)
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "avg_low_price".into(),
            rows.iter()
                .map(|row| row.avg_low_price.map(|value| value.0))
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "low_price_volume".into(),
            rows.iter()
                .map(|row| row.low_price_volume)
                .collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn predictions_frame(rows: &[StoredPrediction]) -> Result<DataFrame, AnalyticsError> {
    Ok(DataFrame::new(vec![
        Series::new(
            "strategy_id".into(),
            rows.iter()
                .map(|row| row.strategy_id.0.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "model_version".into(),
            rows.iter()
                .map(|row| row.model_version.0.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "item_id".into(),
            rows.iter().map(|row| row.item_id.0).collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "as_of".into(),
            rows.iter()
                .map(|row| row.as_of.to_rfc3339())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "confidence".into(),
            rows.iter()
                .map(|row| row.confidence.get())
                .collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn outcomes_frame(rows: &[EvaluatedRecommendationRecord]) -> Result<DataFrame, AnalyticsError> {
    Ok(DataFrame::new(vec![
        Series::new(
            "recommendation_id".into(),
            rows.iter()
                .map(|row| row.recommendation.recommendation_id.0.to_string())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "item_id".into(),
            rows.iter()
                .map(|row| row.recommendation.item_id.0)
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "evaluated_at".into(),
            rows.iter()
                .map(|row| row.outcome.evaluated_at.to_rfc3339())
                .collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn reason_outcomes_frame(rows: &[ReasonOutcomeSummary]) -> Result<DataFrame, AnalyticsError> {
    Ok(DataFrame::new(vec![
        Series::new(
            "reason_key".into(),
            rows.iter()
                .map(|row| row.reason_key.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "model_version".into(),
            rows.iter()
                .map(|row| row.model_version.0.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "window_end".into(),
            rows.iter()
                .map(|row| row.window_end.to_rfc3339())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "sample_size".into(),
            rows.iter().map(|row| row.sample_size).collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn graph_versions_frame(rows: &[GraphVersion]) -> Result<DataFrame, AnalyticsError> {
    Ok(DataFrame::new(vec![
        Series::new(
            "graph_version".into(),
            rows.iter()
                .map(|row| row.graph_version.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "source_hash".into(),
            rows.iter()
                .map(|row| row.source_hash.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "created_at".into(),
            rows.iter()
                .map(|row| row.created_at.to_rfc3339())
                .collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn item_edges_frame(rows: &[ItemGraphEdge]) -> Result<DataFrame, AnalyticsError> {
    Ok(DataFrame::new(vec![
        Series::new(
            "edge_id".into(),
            rows.iter()
                .map(|row| row.edge_id.to_string())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "graph_version".into(),
            rows.iter()
                .map(|row| row.graph_version.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "from_item_id".into(),
            rows.iter()
                .map(|row| row.from_item_id.0)
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "to_item_id".into(),
            rows.iter().map(|row| row.to_item_id.0).collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn edge_observations_frame(rows: &[EdgeObservation]) -> Result<DataFrame, AnalyticsError> {
    Ok(DataFrame::new(vec![
        Series::new(
            "edge_id".into(),
            rows.iter()
                .map(|row| row.edge_id.to_string())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "method".into(),
            rows.iter().map(observation_method_key).collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "observed_at".into(),
            rows.iter()
                .map(|row| row.observed_at.to_rfc3339())
                .collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn market_events_frame(rows: &[StoredMarketEvent]) -> Result<DataFrame, AnalyticsError> {
    Ok(DataFrame::new(vec![
        Series::new(
            "event_id".into(),
            rows.iter()
                .map(|row| row.event.event_id.to_string())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "graph_version".into(),
            rows.iter()
                .map(|row| row.event.graph_version.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "occurred_at".into(),
            rows.iter()
                .map(|row| row.event.occurred_at.to_rfc3339())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "title".into(),
            rows.iter()
                .map(|row| row.event.title.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn json_values_frame(
    column_name: &str,
    rows: &[serde_json::Value],
) -> Result<DataFrame, AnalyticsError> {
    let columns: Vec<Column> = vec![
        Series::new(
            column_name.into(),
            rows.iter()
                .map(serde_json::Value::to_string)
                .collect::<Vec<_>>(),
        )
        .into(),
    ];
    Ok(DataFrame::new(columns)?)
}

pub fn fixture_archive_source_data() -> ArchiveSourceData {
    use chrono::TimeZone;
    use grand_edge_domain::{
        EdgeObservationMethod, Gp, GraphEdgeDirection, GraphEdgeSourceType, GraphEdgeType, ItemId,
        MarketEventType, ModelVersion, OutcomeLabel, Probability, Rate, ReasonType, Recommendation,
        RecommendationAction, RecommendationExplanation, RecommendationId, RecommendationOutcome,
        SignalSide, StrategyId, StructuredRecommendationExplanation, UserId,
    };

    let as_of = Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();
    let graph_version = "relations_2026_06_16_v3".to_string();
    let edge_id = Uuid::new_v4();

    ArchiveSourceData {
        interval_prices: vec![IntervalPrice {
            item_id: ItemId(4151),
            bucket_start: as_of,
            interval: PriceInterval::FiveMinute,
            avg_high_price: Some(Gp(100_000)),
            high_price_volume: 24,
            avg_low_price: Some(Gp(99_750)),
            low_price_volume: 21,
        }],
        predictions: vec![StoredPrediction {
            strategy_id: StrategyId::new("baseline_momentum").unwrap(),
            model_version: ModelVersion::new("v1").unwrap(),
            item_id: ItemId(4151),
            as_of,
            horizon_secs: 21_600,
            side: SignalSide::Buy,
            expected_return: Rate::new(0.03).unwrap(),
            confidence: Probability::new(0.74).unwrap(),
            expected_net_gp_per_unit: Gp(1200),
            target_entry: None,
            target_exit: None,
            stop_loss: None,
            take_profit: None,
            max_quantity: None,
            explanation: serde_json::json!({}),
        }],
        outcomes: vec![EvaluatedRecommendationRecord {
            recommendation: Recommendation {
                recommendation_id: RecommendationId(Uuid::new_v4()),
                user_id: Some(UserId(Uuid::new_v4())),
                item_id: ItemId(4151),
                as_of,
                action: RecommendationAction::Buy,
                score: Rate::new(0.4).unwrap(),
                prediction_confidence: Some(Probability::new(0.7).unwrap()),
                execution_confidence: Some(Probability::new(0.6).unwrap()),
                recommendation_confidence: Probability::new(0.65).unwrap(),
                expected_net_gp: Some(Gp(1200)),
                expected_roi: Some(Rate::new(0.03).unwrap()),
                risk_label: Some("medium".to_string()),
                reasons: vec!["fixture".to_string()],
                explanation: RecommendationExplanation {
                    feature_set_version: "features_v1".to_string(),
                    market_rules_version: "rules_v1".to_string(),
                    graph_version: Some(graph_version.clone()),
                    graph_context: None,
                    strategy_votes: Vec::new(),
                    score_components: Vec::new(),
                    accuracy_snapshot: None,
                    structured_explanation: StructuredRecommendationExplanation::default(),
                },
            },
            outcome: RecommendationOutcome {
                recommendation_id: RecommendationId(Uuid::new_v4()),
                evaluated_at: as_of,
                horizon_secs: grand_edge_domain::HorizonSecs(21_600),
                actual_return: Some(Rate::new(0.02).unwrap()),
                actual_net_gp: Some(Gp(900)),
                direction_correct: Some(true),
                hit_take_profit: false,
                hit_stop_loss: false,
                max_favourable_excursion: Some(Rate::new(0.04).unwrap()),
                max_adverse_excursion: Some(Rate::new(-0.01).unwrap()),
                outcome_label: OutcomeLabel::Win,
            },
        }],
        reason_outcomes: vec![ReasonOutcomeSummary {
            reason_type: ReasonType::GraphRelationship,
            reason_key: "graph:shock_transmits_to".to_string(),
            model_version: ModelVersion::new("v1").unwrap(),
            recommendation_action: RecommendationAction::Buy,
            execution_mode: None,
            confidence_bucket: Some("0.7-0.8".to_string()),
            window_start: Utc.with_ymd_and_hms(2026, 6, 9, 0, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            sample_size: 6,
            publishable: true,
            win_rate: Some(Probability::new(0.66).unwrap()),
            avg_actual_return: Some(Rate::new(0.018).unwrap()),
            avg_net_gp: Some(Gp(8_500)),
            calibration_error: Some(0.09),
        }],
        graph_versions: vec![GraphVersion {
            graph_version: graph_version.clone(),
            source_hash: "abc123".to_string(),
            created_at: as_of,
            description: "fixture graph".to_string(),
        }],
        item_edges: vec![ItemGraphEdge {
            edge_id,
            graph_version: graph_version.clone(),
            from_item_id: ItemId(4151),
            to_item_id: ItemId(11840),
            edge_type: GraphEdgeType::ShockTransmitsTo,
            direction: GraphEdgeDirection::Downstream,
            sign: 1.0,
            weight: 0.6,
            lag_seconds: Some(300),
            confidence: 0.8,
            source_type: GraphEdgeSourceType::Curated,
            source_ref: Some("fixture".to_string()),
            observations: Vec::new(),
            formula: serde_json::json!({"kind": "fixture"}),
            requires_review: false,
            active: true,
            created_at: as_of,
            updated_at: as_of,
        }],
        edge_observations: vec![EdgeObservation {
            edge_id,
            observed_at: as_of,
            method: EdgeObservationMethod::LeadLagRegression,
            window_start: Utc.with_ymd_and_hms(2026, 6, 10, 0, 0, 0).unwrap(),
            window_end: as_of,
            statistic: Some(0.31),
            p_value: Some(0.04),
            estimated_lag_seconds: Some(300),
            estimated_effect: Some(0.05),
            confidence: 0.71,
            metadata: serde_json::json!({"fixture": true}),
        }],
        market_events: vec![StoredMarketEvent {
            event: MarketEventNode {
                event_id: Uuid::new_v4(),
                graph_version,
                event_type: MarketEventType::GameUpdate,
                title: "Fixture patch".to_string(),
                occurred_at: as_of,
                source_ref: "corpus:fixture".to_string(),
                affected_item_ids: vec![ItemId(4151)],
                metadata: serde_json::json!({"severity": "medium"}),
            },
            item_links: Vec::new(),
        }],
        blast_simulations: vec![serde_json::json!({
            "graph_version": "relations_2026_06_16_v3",
            "source_item_id": 4151,
            "impacted_item_count": 3
        })],
        graph_path_metrics: vec![serde_json::json!({
            "graph_version": "relations_2026_06_16_v3",
            "source_item_id": 4151,
            "target_item_id": 11840,
            "path_confidence": 0.72
        })],
        outcome_summaries_present: true,
        reason_summaries_present: true,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::{TimeZone, Utc};

    use super::{
        ArchiveDataset, ArchiveJob, ArchivePartition, fixture_archive_source_data,
        plan_archive_from_data, run_archive_from_data,
    };
    use crate::{LocalFileObjectStore, RetentionPolicy};

    #[test]
    fn archive_partition_path_includes_date_interval_and_model_version() {
        let interval = ArchivePartition {
            dataset: ArchiveDataset::IntervalPrices,
            interval: Some("5m".to_string()),
            date: Utc
                .with_ymd_and_hms(2026, 6, 16, 0, 0, 0)
                .unwrap()
                .date_naive(),
            model_id: None,
            model_version: None,
            graph_version: None,
            method: None,
            window_end: None,
        };
        let prediction = ArchivePartition {
            dataset: ArchiveDataset::Predictions,
            interval: None,
            date: Utc
                .with_ymd_and_hms(2026, 6, 16, 0, 0, 0)
                .unwrap()
                .date_naive(),
            model_id: Some("baseline_momentum".to_string()),
            model_version: Some("v1".to_string()),
            graph_version: None,
            method: None,
            window_end: None,
        };

        assert_eq!(
            interval.object_path(),
            PathBuf::from("interval_prices/interval=5m/date=2026-06-16/part-000.parquet.zst")
        );
        assert_eq!(
            prediction.object_path(),
            PathBuf::from(
                "predictions/model_id=baseline_momentum/model_version=v1/date=2026-06-16/part-000.parquet.zst"
            )
        );
    }

    #[test]
    fn archive_partition_path_includes_graph_version_for_graph_datasets() {
        let partition = ArchivePartition {
            dataset: ArchiveDataset::ItemEdges,
            interval: None,
            date: Utc
                .with_ymd_and_hms(2026, 6, 16, 0, 0, 0)
                .unwrap()
                .date_naive(),
            model_id: None,
            model_version: None,
            graph_version: Some("relations_2026_06_16_v3".to_string()),
            method: None,
            window_end: None,
        };

        assert_eq!(
            partition.object_path(),
            PathBuf::from(
                "item_edges/graph_version=relations_2026_06_16_v3/date=2026-06-16/part-000.parquet.zst"
            )
        );
    }

    #[test]
    fn dry_run_never_deletes_hot_records() {
        let job = ArchiveJob {
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            policy: RetentionPolicy::default(),
            dry_run: true,
            allow_hot_delete: true,
        };
        let plan = plan_archive_from_data(&job, &fixture_archive_source_data()).unwrap();
        assert!(!plan.delete_eligibility.eligible);
        assert!(
            plan.delete_eligibility
                .blockers
                .iter()
                .any(|blocker| blocker.contains("dry_run"))
        );
    }

    #[test]
    fn delete_requires_reason_and_outcome_summaries() {
        let job = ArchiveJob {
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            policy: RetentionPolicy::default(),
            dry_run: false,
            allow_hot_delete: true,
        };
        let mut data = fixture_archive_source_data();
        data.reason_summaries_present = false;
        data.outcome_summaries_present = false;

        let plan = plan_archive_from_data(&job, &data).unwrap();
        assert!(!plan.delete_eligibility.eligible);
        assert_eq!(plan.delete_eligibility.blockers.len(), 3);
    }

    #[test]
    fn archive_manifest_records_row_count_and_checksum() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalFileObjectStore::new(dir.path().to_path_buf()).unwrap();
        let manifest = run_archive_from_data(
            &store,
            ArchiveJob {
                as_of: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
                policy: RetentionPolicy::default(),
                dry_run: true,
                allow_hot_delete: false,
            },
            fixture_archive_source_data(),
        )
        .unwrap();

        assert!(manifest.row_count > 0);
        assert_eq!(manifest.checksum.len(), 64);
        assert!(dir.path().join("manifest.json").is_file());
    }
}
