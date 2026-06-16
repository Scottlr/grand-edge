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
}

#[derive(Debug, Clone, Subcommand)]
pub enum AnalyticsCommand {
    ExportFeatures {
        from: String,
        to: String,
        out: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ModelCommand {
    Validate { artifact: String },
    Evaluate { strategy: String, version: String },
}

#[derive(Debug, Clone, Subcommand)]
pub enum SchemaCommand {
    Export,
}

#[derive(Debug, Clone, Subcommand)]
pub enum GraphCommand {
    ImportRelations {
        #[arg(long, default_value = "data/relations")]
        root: String,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ServerCommand {
    Run,
}
