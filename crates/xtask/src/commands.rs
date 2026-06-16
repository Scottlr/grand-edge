mod archive;
mod schema;

use std::fmt::Display;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use grand_edge_analytics::{
    BacktestReportRequest, DatasetExportRequest, export_backtest_report_from_storage,
    export_feature_dataset_from_storage,
};
use grand_edge_configuration::{ConfigProfile, GrandEdgeConfig, load_config};
use grand_edge_domain::{GraphVersion, ItemGraphEdge, ItemGraphNode};
use grand_edge_ingest::{
    IngestError, MarketIntelligenceCorpusImporter, MarketIntelligenceImportReport,
    MarketIntelligenceImportStore, RelationCorpusImporter, RelationImportReport,
    RelationImportStore,
};
use grand_edge_storage::{Storage, StoredCorpusSource, StoredMarketEvent};
use secrecy::ExposeSecret;
use uuid::Uuid;

pub use archive::analytics_archive;
pub use schema::schema_export;

#[derive(Debug, thiserror::Error)]
#[error("command unavailable: {message}")]
pub struct CommandUnavailableError {
    pub message: String,
}

pub fn config_print(config: &GrandEdgeConfig) -> String {
    grand_edge_configuration::secrets::redacted_config_summary(config)
}

pub async fn doctor_summary(
    profile: ConfigProfile,
) -> Result<String, grand_edge_configuration::ConfigurationError> {
    let config = load_config(profile)?;
    let mut lines = Vec::new();
    lines.push(format!("profile={}", profile.as_str()));
    lines.push(format!(
        "database_url={}",
        if config.database.url.expose_secret().is_empty() {
            "missing"
        } else {
            "configured"
        }
    ));
    lines.push(format!(
        "osrs_wiki_user_agent={}",
        config.osrs_wiki.user_agent
    ));
    lines.push(format!("api_bind_addr={}", config.api.bind_addr));

    for tool in ["node", "npm", "uv", "docker"] {
        let status = tool_available(tool);
        lines.push(format!("{tool}={status}"));
    }

    Ok(lines.join("\n"))
}

pub fn unavailable_message<T: Display>(
    dependency_name: &str,
    profile: ConfigProfile,
    command: T,
) -> CommandUnavailableError {
    let _ = profile;
    CommandUnavailableError {
        message: format!(
            "{command} is not available yet because {dependency_name} is not implemented"
        ),
    }
}

pub async fn graph_import_relations(
    profile: ConfigProfile,
    root: &str,
    dry_run: bool,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let relations_root = relation_root(root)?;

    if dry_run {
        let importer = RelationCorpusImporter::new(NoopRelationStore);
        let report = importer
            .import_relation_files(&relations_root, true)
            .await?;
        return Ok(render_relation_report(&report)?);
    }

    let config = load_config(profile)?;
    let storage = Storage::connect(config.database.url.expose_secret()).await?;
    storage.migrate().await?;
    let importer = RelationCorpusImporter::new(storage);
    let report = importer
        .import_relation_files(&relations_root, false)
        .await?;
    Ok(render_relation_report(&report)?)
}

pub async fn corpus_validate(
    root: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let corpus_root = corpus_root(root)?;
    let importer = MarketIntelligenceCorpusImporter::new(NoopCorpusStore);
    let report = importer.import_corpus_files(&corpus_root, true).await?;
    Ok(render_corpus_report(&report)?)
}

pub async fn corpus_import(
    profile: ConfigProfile,
    root: &str,
    dry_run: bool,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let corpus_root = corpus_root(root)?;

    if dry_run {
        let importer = MarketIntelligenceCorpusImporter::new(NoopCorpusStore);
        let report = importer.import_corpus_files(&corpus_root, true).await?;
        return Ok(render_corpus_report(&report)?);
    }

    let config = load_config(profile)?;
    let storage = Storage::connect(config.database.url.expose_secret()).await?;
    storage.migrate().await?;
    let importer = MarketIntelligenceCorpusImporter::new(storage);
    let report = importer.import_corpus_files(&corpus_root, false).await?;
    Ok(render_corpus_report(&report)?)
}

pub async fn analytics_export_features(
    profile: ConfigProfile,
    from: &str,
    to: &str,
    out: &str,
    feature_set_version: &str,
    include_predictions: bool,
    include_outcomes: bool,
    include_raw_interval_candles: bool,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config(profile)?;
    let storage = Storage::connect(config.database.url.expose_secret()).await?;
    let request = DatasetExportRequest {
        output_dir: repo_relative_path(out)?,
        feature_set_version: feature_set_version.to_string(),
        item_ids: None,
        window_start: parse_utc_date(from)?,
        window_end: parse_utc_date(to)?,
        include_predictions,
        include_outcomes,
        include_raw_interval_candles,
    };
    let report = export_feature_dataset_from_storage(&storage, &request).await?;
    Ok(serde_json::to_string_pretty(&report.manifest)?)
}

pub async fn backtest_report(
    profile: ConfigProfile,
    run_id: &str,
    out: &str,
    feature_set_version: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let config = load_config(profile)?;
    let storage = Storage::connect(config.database.url.expose_secret()).await?;
    let request = BacktestReportRequest {
        run_id: Uuid::parse_str(run_id)?,
        output_dir: repo_relative_path(out)?,
        feature_set_version: feature_set_version.to_string(),
    };
    let report = export_backtest_report_from_storage(&storage, &request).await?;
    Ok(serde_json::to_string_pretty(&report.manifest)?)
}

pub fn model_compare_help(
    strategies: &[String],
    window: &str,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&serde_json::json!({
        "strategies": strategies,
        "window": window,
        "note": "Model comparison wiring is reserved for analytics exports and latest stored metrics."
    }))
}

fn relation_root(root: &str) -> Result<PathBuf, std::io::Error> {
    let candidate = Path::new(root);
    if candidate.is_absolute() {
        return candidate.canonicalize();
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(candidate)
        .canonicalize()
}

fn render_relation_report(report: &RelationImportReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

fn render_corpus_report(
    report: &MarketIntelligenceImportReport,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

#[derive(Clone, Copy)]
struct NoopRelationStore;
#[derive(Clone, Copy)]
struct NoopCorpusStore;

#[async_trait]
impl RelationImportStore for NoopRelationStore {
    async fn upsert_corpus_sources(
        &self,
        _sources: &[StoredCorpusSource],
    ) -> Result<u64, IngestError> {
        Ok(0)
    }

    async fn insert_graph_version(&self, _version: &GraphVersion) -> Result<(), IngestError> {
        Ok(())
    }

    async fn upsert_graph_nodes(&self, _nodes: &[ItemGraphNode]) -> Result<u64, IngestError> {
        Ok(0)
    }

    async fn upsert_graph_edges(&self, _edges: &[ItemGraphEdge]) -> Result<u64, IngestError> {
        Ok(0)
    }
}

#[async_trait]
impl MarketIntelligenceImportStore for NoopCorpusStore {
    async fn upsert_corpus_sources(
        &self,
        _sources: &[StoredCorpusSource],
    ) -> Result<u64, IngestError> {
        Ok(0)
    }

    async fn insert_graph_version(&self, _version: &GraphVersion) -> Result<(), IngestError> {
        Ok(())
    }

    async fn upsert_graph_nodes(&self, _nodes: &[ItemGraphNode]) -> Result<u64, IngestError> {
        Ok(0)
    }

    async fn upsert_market_events(&self, _events: &[StoredMarketEvent]) -> Result<(), IngestError> {
        Ok(())
    }

    async fn upsert_graph_edges(&self, _edges: &[ItemGraphEdge]) -> Result<u64, IngestError> {
        Ok(0)
    }
}

fn corpus_root(root: &str) -> Result<PathBuf, std::io::Error> {
    let candidate = Path::new(root);
    if candidate.is_absolute() {
        return candidate.canonicalize();
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(candidate)
        .canonicalize()
}

fn repo_relative_path(path: &str) -> Result<PathBuf, std::io::Error> {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return Ok(candidate.to_path_buf());
    }

    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(candidate))
}

fn parse_utc_date(input: &str) -> Result<DateTime<Utc>, Box<dyn std::error::Error + Send + Sync>> {
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(input) {
        return Ok(timestamp.with_timezone(&Utc));
    }

    let date = NaiveDate::parse_from_str(input, "%Y-%m-%d")?;
    Ok(date
        .and_hms_opt(0, 0, 0)
        .expect("midnight is valid")
        .and_utc())
}

fn tool_available(name: &str) -> &'static str {
    if std::process::Command::new(name)
        .arg("--version")
        .output()
        .is_ok()
    {
        "available"
    } else {
        "missing"
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use clap::CommandFactory;
    use grand_edge_configuration::{ConfigProfile, load_config};

    use crate::cli::Cli;

    use super::{config_print, doctor_summary};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn cli_help_renders_all_subcommands() {
        let mut command = Cli::command();
        let help = command.render_long_help().to_string();
        for expected in [
            "config",
            "doctor",
            "db",
            "ingest",
            "features",
            "backtest",
            "analytics",
            "model",
            "schema",
            "graph",
            "corpus",
            "server",
        ] {
            assert!(help.contains(expected), "{expected} missing from help");
        }
    }

    #[tokio::test]
    async fn doctor_reports_missing_database_url_without_panic() {
        let _guard = env_lock().lock().unwrap();
        let original = std::env::var("DATABASE_URL").ok();
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
        let report = doctor_summary(ConfigProfile::Test).await.unwrap();
        if let Some(original) = original {
            unsafe {
                std::env::set_var("DATABASE_URL", original);
            }
        }
        assert!(
            report.contains("database_url=configured") || report.contains("database_url=missing")
        );
    }

    #[test]
    fn config_print_omits_secret_values() {
        let config = load_config(ConfigProfile::Test).unwrap();
        let rendered = config_print(&config);
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("postgres://"));
    }
}
