use clap::{Parser, Subcommand};
use grand_edge_configuration::ConfigProfile;

#[derive(Debug, Parser)]
#[command(name = "grandedge")]
pub struct Cli {
    #[arg(
        long,
        env = "GRAND_EDGE_PROFILE",
        default_value = "local",
        global = true
    )]
    pub profile: ConfigProfile,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Doctor(DoctorCommand),
    Db {
        #[command(subcommand)]
        command: DbCommand,
    },
    Ingest {
        #[command(subcommand)]
        command: IngestCommand,
    },
    Features {
        #[command(subcommand)]
        command: FeaturesCommand,
    },
    Backtest {
        #[command(subcommand)]
        command: BacktestCommand,
    },
    Analytics {
        #[command(subcommand)]
        command: AnalyticsCommand,
    },
    Model {
        #[command(subcommand)]
        command: ModelCommand,
    },
    Schema {
        #[command(subcommand)]
        command: SchemaCommand,
    },
    Graph {
        #[command(subcommand)]
        command: GraphCommand,
    },
    Corpus {
        #[command(subcommand)]
        command: CorpusCommand,
    },
    Server {
        #[command(subcommand)]
        command: ServerCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommand {
    Print,
}

#[derive(Debug, Clone, Parser)]
pub struct DoctorCommand;

#[derive(Debug, Clone, Subcommand)]
pub enum DbCommand {
    Migrate,
}

#[derive(Debug, Clone, Subcommand)]
pub enum IngestCommand {
    Latest,
    Interval { interval: String },
}

#[derive(Debug, Clone, Subcommand)]
pub enum FeaturesCommand {
    Rebuild { item: i64 },
}

#[derive(Debug, Clone, Subcommand)]
pub enum BacktestCommand {
    Run {
        strategy: String,
        from: String,
        to: String,
    },
    Report {
        #[arg(long)]
        run_id: String,
        #[arg(long)]
        out: String,
        #[arg(long, default_value = "features_v1")]
        feature_set_version: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum AnalyticsCommand {
    ExportFeatures {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long)]
        out: String,
        #[arg(long, default_value = "features_v1")]
        feature_set_version: String,
        #[arg(long, default_value_t = false)]
        include_predictions: bool,
        #[arg(long, default_value_t = false)]
        include_outcomes: bool,
        #[arg(long, default_value_t = false)]
        include_raw_interval_candles: bool,
    },
    Archive {
        #[arg(long)]
        as_of: String,
        #[arg(long)]
        out: String,
        #[arg(long, default_value_t = true)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        allow_hot_delete: bool,
        #[arg(long)]
        fixture: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ModelCommand {
    Validate {
        artifact: String,
    },
    Evaluate {
        strategy: String,
        version: String,
    },
    Compare {
        #[arg(long = "strategy", required = true)]
        strategies: Vec<String>,
        #[arg(long)]
        window: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum SchemaCommand {
    Export {
        #[arg(long, default_value = "schemas")]
        out: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum GraphCommand {
    ImportRelations {
        #[arg(long, default_value = "data/relations")]
        root: String,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    DiscoverEdges {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, default_value = "granger_style")]
        method: String,
        #[arg(long, default_value_t = true)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        fixture: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum CorpusCommand {
    Validate {
        #[arg(long, default_value = "data/corpus")]
        root: String,
    },
    Import {
        #[arg(long, default_value = "data/corpus")]
        root: String,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServerCommand {
    Run,
}
