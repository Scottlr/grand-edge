mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Command};
use commands::{
    analytics_export_features, backtest_report, config_print, corpus_import, corpus_validate,
    doctor_summary, graph_import_relations, model_compare_help, schema_export, unavailable_message,
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
        Command::Backtest { command } => match command {
            cli::BacktestCommand::Run { .. } => {
                return Err(miette::miette!(
                    "{}",
                    unavailable_message("backtest execution", cli.profile, "backtest").message
                ));
            }
            cli::BacktestCommand::Report {
                run_id,
                out,
                feature_set_version,
            } => {
                let report = backtest_report(cli.profile, &run_id, &out, &feature_set_version)
                    .await
                    .map_err(|error| miette::miette!("{error}"))?;
                println!("{report}");
            }
        },
        Command::Analytics { command } => match command {
            cli::AnalyticsCommand::ExportFeatures {
                from,
                to,
                out,
                feature_set_version,
                include_predictions,
                include_outcomes,
                include_raw_interval_candles,
            } => {
                let report = analytics_export_features(
                    cli.profile,
                    &from,
                    &to,
                    &out,
                    &feature_set_version,
                    include_predictions,
                    include_outcomes,
                    include_raw_interval_candles,
                )
                .await
                .map_err(|error| miette::miette!("{error}"))?;
                println!("{report}");
            }
        },
        Command::Model { command } => match command {
            cli::ModelCommand::Validate { .. } => {
                return Err(miette::miette!(
                    "{}",
                    unavailable_message("model_runtime", cli.profile, "model validate").message
                ));
            }
            cli::ModelCommand::Evaluate { .. } => {
                return Err(miette::miette!(
                    "{}",
                    unavailable_message("model_runtime", cli.profile, "model evaluate").message
                ));
            }
            cli::ModelCommand::Compare { strategies, window } => {
                let report = model_compare_help(&strategies, &window)
                    .map_err(|error| miette::miette!("{error}"))?;
                println!("{report}");
            }
        },
        Command::Schema { command } => match command {
            cli::SchemaCommand::Export { out } => {
                let report = schema_export(&out)
                    .await
                    .map_err(|error| miette::miette!("{error}"))?;
                println!("{report}");
            }
        },
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
