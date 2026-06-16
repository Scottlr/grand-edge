use grand_edge_analytics::{
    ArchiveJob, LocalFileObjectStore, RetentionPolicy, fixture_archive_source_data, run_archive,
    run_archive_from_data,
};
use grand_edge_configuration::{ConfigProfile, load_config};
use grand_edge_storage::Storage;
use secrecy::ExposeSecret;

use crate::commands::repo_relative_path;

pub async fn analytics_archive(
    profile: ConfigProfile,
    as_of: chrono::DateTime<chrono::Utc>,
    out: &str,
    dry_run: bool,
    allow_hot_delete: bool,
    fixture: Option<&str>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let output_dir = repo_relative_path(out)?;
    let store = LocalFileObjectStore::new(output_dir)?;
    let job = ArchiveJob {
        as_of,
        policy: RetentionPolicy::default(),
        dry_run,
        allow_hot_delete,
    };

    let manifest = if fixture.is_some() {
        run_archive_from_data(&store, job, fixture_archive_source_data())?
    } else {
        let config = load_config(profile)?;
        let storage = Storage::connect(config.database.url.expose_secret()).await?;
        run_archive(&storage, &store, job).await?
    };

    Ok(serde_json::to_string_pretty(&manifest)?)
}
