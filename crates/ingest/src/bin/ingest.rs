use std::{path::PathBuf, time::Duration};

use clap::{Parser, Subcommand, ValueEnum};
use config::{Environment, File, FileFormat};
use grand_edge_domain::PriceInterval;
use grand_edge_ingest::{
    IngestionJobConfig, IngestionJobs, OsrsWikiClient, OsrsWikiConfig, OsrsWikiRateLimitConfig,
};
use grand_edge_storage::Storage;
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(name = "ingest")]
struct Cli {
    #[arg(long, value_enum, default_value_t = ConfigProfile::Local, global = true)]
    profile: ConfigProfile,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ConfigProfile {
    Local,
    Test,
    Production,
}

#[derive(Debug, Subcommand)]
enum Command {
    SyncMapping,
    IngestLatest,
    IngestInterval {
        #[arg(long, value_enum)]
        interval: CliInterval,
    },
    BackfillTimeseries {
        #[arg(long, value_enum)]
        interval: CliInterval,
        #[arg(long = "item-id")]
        item_ids: Vec<i64>,
    },
    Poll,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliInterval {
    #[value(name = "5m")]
    FiveMinute,
    #[value(name = "1h")]
    OneHour,
    #[value(name = "6h")]
    SixHour,
    #[value(name = "24h")]
    TwentyFourHour,
}

#[derive(Debug, Deserialize)]
struct RuntimeConfig {
    database: DatabaseConfig,
    osrs_wiki: OsrsWikiSection,
    ingest: IngestSection,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    url: String,
}

#[derive(Debug, Deserialize)]
struct OsrsWikiSection {
    base_url: String,
    app_name: String,
    contact_email: String,
    user_agent: String,
    request_timeout_ms: u64,
    max_requests_per_second: f64,
    burst_size: u32,
    respect_retry_after: bool,
    max_retries: u32,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
}

#[derive(Debug, Deserialize)]
struct IngestSection {
    poll_latest_seconds: u64,
    poll_5m_seconds: u64,
    poll_1h_seconds: u64,
    sync_mapping_seconds: u64,
    max_timeseries_items_per_run: usize,
    max_timeseries_requests_per_minute: u32,
    item_allowlist: Option<Vec<i64>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let cli = Cli::parse();
    let config = load_runtime_config(cli.profile)?;
    let wiki = build_wiki_config(&config.osrs_wiki)?;
    let jobs_config = build_jobs_config(config.ingest);
    let storage = Storage::connect(&config.database.url).await?;
    storage.migrate().await?;
    let jobs = IngestionJobs::new(OsrsWikiClient::new(wiki)?, storage, jobs_config)?;

    match cli.command {
        Command::SyncMapping => print_report(jobs.sync_mapping().await?)?,
        Command::IngestLatest => print_report(jobs.ingest_latest_snapshot().await?)?,
        Command::IngestInterval { interval } => {
            print_report(jobs.ingest_interval_snapshot(interval.into()).await?)?
        }
        Command::BackfillTimeseries { interval, item_ids } => {
            print_report(jobs.backfill_timeseries(&item_ids, interval.into()).await?)?
        }
        Command::Poll => {
            jobs.run_polling_loop(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await?;
        }
    }

    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .try_init();
}

fn load_runtime_config(profile: ConfigProfile) -> anyhow::Result<RuntimeConfig> {
    let _ = dotenvy::dotenv();
    let root = repo_root()?;
    let config_dir = root.join("configs");
    let profile_name = profile.as_str();

    let mut builder = config::Config::builder()
        .add_source(File::from(config_dir.join("default.toml")).format(FileFormat::Toml))
        .add_source(
            File::from(config_dir.join(format!("{profile_name}.toml")))
                .format(FileFormat::Toml)
                .required(false),
        )
        .add_source(
            File::from(config_dir.join(format!("{profile_name}.local.toml")))
                .format(FileFormat::Toml)
                .required(false),
        )
        .add_source(
            File::from(config_dir.join("local.toml"))
                .format(FileFormat::Toml)
                .required(false),
        )
        .add_source(Environment::with_prefix("GRAND_EDGE").separator("__"));

    if let Ok(value) = std::env::var("DATABASE_URL") {
        builder = builder.set_override("database.url", value)?;
    }
    if let Ok(value) = std::env::var("GRAND_EDGE_USER_AGENT") {
        builder = builder.set_override("osrs_wiki.user_agent", value)?;
    }
    if let Ok(value) = std::env::var("OSRS_WIKI_BASE_URL") {
        builder = builder.set_override("osrs_wiki.base_url", value)?;
    }

    Ok(builder.build()?.try_deserialize()?)
}

fn build_wiki_config(section: &OsrsWikiSection) -> anyhow::Result<OsrsWikiConfig> {
    let config = OsrsWikiConfig {
        base_url: reqwest::Url::parse(&section.base_url)?,
        app_name: section.app_name.clone(),
        contact_email: section.contact_email.clone(),
        user_agent: section.user_agent.clone(),
        request_timeout: Duration::from_millis(section.request_timeout_ms),
        rate_limit: OsrsWikiRateLimitConfig {
            max_requests_per_second: section.max_requests_per_second,
            burst_size: section.burst_size,
            respect_retry_after: section.respect_retry_after,
            max_retries: section.max_retries,
            initial_backoff: Duration::from_millis(section.initial_backoff_ms),
            max_backoff: Duration::from_millis(section.max_backoff_ms),
        },
    };
    config.validate()?;
    Ok(config)
}

fn build_jobs_config(section: IngestSection) -> IngestionJobConfig {
    IngestionJobConfig {
        poll_latest_seconds: section.poll_latest_seconds,
        poll_5m_seconds: section.poll_5m_seconds,
        poll_1h_seconds: section.poll_1h_seconds,
        sync_mapping_seconds: section.sync_mapping_seconds,
        max_timeseries_items_per_run: section.max_timeseries_items_per_run,
        max_timeseries_requests_per_minute: section.max_timeseries_requests_per_minute,
        item_allowlist: section.item_allowlist,
    }
}

fn print_report(report: grand_edge_ingest::JobReport) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn repo_root() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

impl ConfigProfile {
    fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Test => "test",
            Self::Production => "production",
        }
    }
}

impl From<CliInterval> for PriceInterval {
    fn from(value: CliInterval) -> Self {
        match value {
            CliInterval::FiveMinute => Self::FiveMinute,
            CliInterval::OneHour => Self::OneHour,
            CliInterval::SixHour => Self::SixHour,
            CliInterval::TwentyFourHour => Self::TwentyFourHour,
        }
    }
}
