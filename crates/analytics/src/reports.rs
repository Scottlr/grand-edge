use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use grand_edge_storage::{Storage, StoredPaperBet};
use polars::prelude::{DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AnalyticsError;
use crate::manifest::{ReportFile, ReportManifest, StrategyVersionRecord};
use crate::parquet::{sha256_file, write_parquet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestReportRequest {
    pub run_id: Uuid,
    pub output_dir: PathBuf,
    pub feature_set_version: String,
}

#[derive(Debug, Clone)]
pub struct BacktestReportData {
    pub run_name: String,
    pub run_status: String,
    pub paper_bets: Vec<StoredPaperBet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetricsSummary {
    pub run_id: Uuid,
    pub run_name: String,
    pub run_status: String,
    pub sample_size: u64,
    pub closed_bet_count: u64,
    pub net_profit_gp: i64,
    pub average_realized_roi: Option<f64>,
    pub execution_modes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BacktestReportResult {
    pub manifest: ReportManifest,
    pub metrics: BacktestMetricsSummary,
    pub output_dir: PathBuf,
}

pub async fn export_backtest_report_from_storage(
    storage: &Storage,
    request: &BacktestReportRequest,
) -> Result<BacktestReportResult, AnalyticsError> {
    let run = storage
        .simulations()
        .get_run(grand_edge_domain::RunId(request.run_id))
        .await?
        .ok_or(AnalyticsError::MissingRun(request.run_id))?;
    let paper_bets = storage
        .simulations()
        .list_paper_bets_for_run(request.run_id)
        .await?;

    export_backtest_report(
        request,
        BacktestReportData {
            run_name: run.name,
            run_status: run.status,
            paper_bets,
        },
    )
}

pub fn export_backtest_report(
    request: &BacktestReportRequest,
    data: BacktestReportData,
) -> Result<BacktestReportResult, AnalyticsError> {
    let (window_start, window_end) = report_bounds(&data.paper_bets)?;
    fs::create_dir_all(&request.output_dir)
        .map_err(|_| AnalyticsError::CreateDirectory(request.output_dir.clone()))?;

    let mut paper_bets_frame = paper_bets_frame(&data.paper_bets)?;
    let paper_bets_path = request.output_dir.join("paper_bets.parquet");
    let paper_bet_rows = write_parquet(&paper_bets_path, &mut paper_bets_frame)?;

    let metrics = metrics_summary(request.run_id, &data);
    let metrics_path = request.output_dir.join("metrics.json");
    fs::write(&metrics_path, serde_json::to_vec_pretty(&metrics)?)?;

    let mut files = vec![file_record(
        &request.output_dir,
        &paper_bets_path,
        paper_bet_rows,
    )?];
    files.push(ReportFile {
        path: PathBuf::from("metrics.json"),
        sha256: sha256_file(&metrics_path)?,
        row_count: metrics.sample_size,
    });

    let manifest = ReportManifest {
        report_id: request.run_id,
        generated_at: Utc::now(),
        source_window_start: window_start,
        source_window_end: window_end,
        raw_candle_window_start: Some(window_start),
        raw_candle_window_end: Some(window_end),
        feature_set_version: request.feature_set_version.clone(),
        strategy_versions: distinct_strategy_versions(&data.paper_bets),
        execution_modes: metrics.execution_modes.clone(),
        files,
    };
    fs::write(
        request.output_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;

    Ok(BacktestReportResult {
        manifest,
        metrics,
        output_dir: request.output_dir.clone(),
    })
}

fn report_bounds(
    paper_bets: &[StoredPaperBet],
) -> Result<(DateTime<Utc>, DateTime<Utc>), AnalyticsError> {
    let window_start = paper_bets
        .iter()
        .map(|bet| bet.entry_time)
        .min()
        .ok_or(AnalyticsError::InvalidWindow)?;
    let window_end = paper_bets
        .iter()
        .map(|bet| bet.exit_time.unwrap_or(bet.entry_time))
        .max()
        .ok_or(AnalyticsError::InvalidWindow)?;
    if window_end <= window_start {
        return Err(AnalyticsError::InvalidWindow);
    }
    Ok((window_start, window_end))
}

fn paper_bets_frame(rows: &[StoredPaperBet]) -> Result<DataFrame, AnalyticsError> {
    Ok(DataFrame::new(vec![
        Series::new(
            "bet_id".into(),
            rows.iter()
                .map(|row| row.bet_id.to_string())
                .collect::<Vec<_>>(),
        )
        .into(),
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
            "entry_time".into(),
            rows.iter()
                .map(|row| row.entry_time.to_rfc3339())
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "entry_price".into(),
            rows.iter().map(|row| row.entry_price.0).collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "quantity".into(),
            rows.iter().map(|row| row.quantity).collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "tax_paid".into(),
            rows.iter().map(|row| row.tax_paid).collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "realized_profit_gp".into(),
            rows.iter()
                .map(|row| row.realized_profit_gp)
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "realized_roi".into(),
            rows.iter().map(|row| row.realized_roi).collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "status".into(),
            rows.iter()
                .map(|row| row.status.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn metrics_summary(run_id: Uuid, data: &BacktestReportData) -> BacktestMetricsSummary {
    let realized_rois = data
        .paper_bets
        .iter()
        .filter_map(|bet| bet.realized_roi)
        .collect::<Vec<_>>();
    let net_profit_gp = data
        .paper_bets
        .iter()
        .filter_map(|bet| bet.realized_profit_gp)
        .sum();
    let average_realized_roi = (!realized_rois.is_empty())
        .then_some(realized_rois.iter().sum::<f64>() / realized_rois.len() as f64);

    BacktestMetricsSummary {
        run_id,
        run_name: data.run_name.clone(),
        run_status: data.run_status.clone(),
        sample_size: data.paper_bets.len() as u64,
        closed_bet_count: data
            .paper_bets
            .iter()
            .filter(|bet| bet.status == "closed")
            .count() as u64,
        net_profit_gp,
        average_realized_roi,
        execution_modes: distinct_execution_modes(&data.paper_bets),
    }
}

fn distinct_execution_modes(rows: &[StoredPaperBet]) -> Vec<String> {
    let mut modes = BTreeSet::new();
    for row in rows {
        if let Some(mode) = row
            .explanation
            .get("execution_mode")
            .and_then(serde_json::Value::as_str)
        {
            modes.insert(mode.to_string());
        }
    }
    modes.into_iter().collect()
}

fn distinct_strategy_versions(rows: &[StoredPaperBet]) -> Vec<StrategyVersionRecord> {
    let mut versions = BTreeSet::new();
    for row in rows {
        versions.insert((row.strategy_id.0.clone(), row.model_version.0.clone()));
    }
    versions
        .into_iter()
        .map(|(strategy_id, model_version)| StrategyVersionRecord {
            strategy_id,
            model_version,
        })
        .collect()
}

fn file_record(
    output_dir: &Path,
    path: &Path,
    row_count: u64,
) -> Result<ReportFile, AnalyticsError> {
    Ok(ReportFile {
        path: path
            .strip_prefix(output_dir)
            .map_err(|_| AnalyticsError::MissingFileName(path.to_path_buf()))?
            .to_path_buf(),
        sha256: sha256_file(path)?,
        row_count,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{Gp, ItemId, ModelVersion, StrategyId};
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::{BacktestReportData, BacktestReportRequest, export_backtest_report};

    #[test]
    fn manifest_records_execution_modes_and_source_windows() {
        let dir = tempdir().unwrap();
        let request = BacktestReportRequest {
            run_id: Uuid::new_v4(),
            output_dir: dir.path().to_path_buf(),
            feature_set_version: "features_v1".to_string(),
        };
        let result = export_backtest_report(&request, fixture_data()).unwrap();
        assert_eq!(
            result.manifest.execution_modes,
            vec!["conservative_instant".to_string()]
        );
        assert!(result.manifest.source_window_end > result.manifest.source_window_start);
        assert!(dir.path().join("manifest.json").is_file());
        assert!(dir.path().join("metrics.json").is_file());
        assert!(dir.path().join("paper_bets.parquet").is_file());
    }

    fn fixture_data() -> BacktestReportData {
        BacktestReportData {
            run_name: "fixture run".to_string(),
            run_status: "finished".to_string(),
            paper_bets: vec![grand_edge_storage::StoredPaperBet {
                bet_id: Uuid::new_v4(),
                run_id: Uuid::new_v4(),
                recommendation_id: None,
                strategy_id: StrategyId::new("momentum_v1").unwrap(),
                model_version: ModelVersion::new("v1").unwrap(),
                item_id: ItemId(4151),
                entry_time: Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap(),
                entry_price: Gp(100_000),
                quantity: 10,
                target_exit: Some(Gp(103_000)),
                stop_loss: Some(Gp(99_000)),
                exit_time: Some(Utc.with_ymd_and_hms(2026, 6, 1, 18, 0, 0).unwrap()),
                exit_price: Some(Gp(101_000)),
                tax_paid: 2060,
                realized_profit_gp: Some(9400),
                realized_roi: Some(0.0094),
                max_drawdown: Some(0.04),
                hit_reason: Some("take_profit".to_string()),
                status: "closed".to_string(),
                explanation: serde_json::json!({"execution_mode": "conservative_instant"}),
            }],
        }
    }
}
