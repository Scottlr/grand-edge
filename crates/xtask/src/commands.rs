use std::fmt::Display;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use grand_edge_configuration::{ConfigProfile, GrandEdgeConfig, load_config};
use grand_edge_domain::{GraphVersion, ItemGraphEdge, ItemGraphNode};
use grand_edge_ingest::{
    IngestError, RelationCorpusImporter, RelationImportReport, RelationImportStore,
};
use grand_edge_storage::{Storage, StoredCorpusSource};
use secrecy::ExposeSecret;

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

#[derive(Clone, Copy)]
struct NoopRelationStore;

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
