mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Command};
use commands::{
    config_print, corpus_import, corpus_validate, doctor_summary, graph_import_relations,
    unavailable_message,
};
use grand_edge_configuration::load_config;
use miette::IntoDiagnostic;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Config { command } => match command {
            cli::ConfigCommand::Print => {
                let config = load_config(cli.profile).into_diagnostic()?;
                println!("{}", config_print(&config));
            }
        },
        Command::Doctor(_) => {
            let report = doctor_summary(cli.profile).await.into_diagnostic()?;
            println!("{report}");
        }
        Command::Db { command } => match command {
            cli::DbCommand::Migrate => {
                let config = load_config(cli.profile).into_diagnostic()?;
                let storage = grand_edge_storage::Storage::connect(
                    secrecy::ExposeSecret::expose_secret(&config.database.url),
                )
                .await
                .into_diagnostic()?;
                storage.migrate().await.into_diagnostic()?;
                println!("database migration complete");
            }
        },
        Command::Ingest { .. } => {
            return Err(miette::miette!(
                "{}",
                unavailable_message("ingest", cli.profile, "ingest").message
            ));
        }
        Command::Features { .. } => {
            return Err(miette::miette!(
                "{}",
                unavailable_message("features", cli.profile, "features").message
            ));
        }
        Command::Backtest { .. } => {
            return Err(miette::miette!(
                "{}",
                unavailable_message("backtest", cli.profile, "backtest").message
            ));
        }
        Command::Analytics { .. } => {
            return Err(miette::miette!(
                "{}",
                unavailable_message("analytics", cli.profile, "analytics").message
            ));
        }
        Command::Model { .. } => {
            return Err(miette::miette!(
                "{}",
                unavailable_message("model_runtime", cli.profile, "model").message
            ));
        }
        Command::Schema { .. } => {
            return Err(miette::miette!(
                "{}",
                unavailable_message("schema export", cli.profile, "schema").message
            ));
        }
        Command::Graph { command } => match command {
            cli::GraphCommand::ImportRelations { root, dry_run } => {
                let report = graph_import_relations(cli.profile, &root, dry_run)
                    .await
                    .map_err(|error| miette::miette!("{error}"))?;
                println!("{report}");
            }
        },
        Command::Corpus { command } => match command {
            cli::CorpusCommand::Validate { root } => {
                let report = corpus_validate(&root)
                    .await
                    .map_err(|error| miette::miette!("{error}"))?;
                println!("{report}");
            }
            cli::CorpusCommand::Import { root, dry_run } => {
                let report = corpus_import(cli.profile, &root, dry_run)
                    .await
                    .map_err(|error| miette::miette!("{error}"))?;
                println!("{report}");
            }
        },
        Command::Server { .. } => {
            return Err(miette::miette!(
                "{}",
                unavailable_message("api server integration", cli.profile, "server").message
            ));
        }
    }

    Ok(())
}
