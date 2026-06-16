use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use grand_edge_domain::{FeatureVector, IntervalPrice, ItemId, PriceInterval};
use grand_edge_storage::{EvaluatedRecommendationRecord, Storage, StoredPrediction};
use polars::prelude::{Column, DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AnalyticsError;
use crate::manifest::{ReportFile, ReportManifest, StrategyVersionRecord};
use crate::parquet::{sha256_file, write_parquet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetExportRequest {
    pub output_dir: PathBuf,
    pub feature_set_version: String,
    pub item_ids: Option<Vec<ItemId>>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub include_predictions: bool,
    pub include_outcomes: bool,
    pub include_raw_interval_candles: bool,
}

#[derive(Debug, Clone)]
pub struct DatasetExportData {
    pub features: Vec<FeatureVector>,
    pub interval_prices: Vec<IntervalPrice>,
    pub predictions: Vec<StoredPrediction>,
    pub outcomes: Vec<EvaluatedRecommendationRecord>,
}

#[derive(Debug, Clone)]
pub struct DatasetExportResult {
    pub manifest: ReportManifest,
    pub output_dir: PathBuf,
}

pub async fn export_feature_dataset_from_storage(
    storage: &Storage,
    request: &DatasetExportRequest,
) -> Result<DatasetExportResult, AnalyticsError> {
    validate_window(request.window_start, request.window_end)?;

    let features = storage
        .features()
        .list_between(
            &request.feature_set_version,
            request.item_ids.as_deref(),
            request.window_start,
            request.window_end,
        )
        .await?;

    let interval_prices = if request.include_raw_interval_candles {
        storage
            .prices()
            .list_between(
                request.item_ids.as_deref(),
                None,
                request.window_start,
                request.window_end,
            )
            .await?
    } else {
        Vec::new()
    };

    let predictions = if request.include_predictions {
        storage
            .strategies()
            .list_predictions_between(
                request.item_ids.as_deref(),
                request.window_start,
                request.window_end,
            )
            .await?
    } else {
        Vec::new()
    };

    let outcomes = if request.include_outcomes {
        storage
            .recommendations()
            .list_evaluated_between(request.window_start, request.window_end)
            .await?
    } else {
        Vec::new()
    };

    export_feature_dataset(
        request,
        DatasetExportData {
            features,
            interval_prices,
            predictions,
            outcomes,
        },
    )
}

pub fn export_feature_dataset(
    request: &DatasetExportRequest,
    data: DatasetExportData,
) -> Result<DatasetExportResult, AnalyticsError> {
    validate_window(request.window_start, request.window_end)?;
    fs::create_dir_all(&request.output_dir)
        .map_err(|_| AnalyticsError::CreateDirectory(request.output_dir.clone()))?;

    let report_id = Uuid::new_v4();
    let generated_at = Utc::now();
    let mut files = Vec::new();

    let mut features_frame = features_frame(&data.features)?;
    let features_path = request.output_dir.join("features.parquet");
    let feature_rows = write_parquet(&features_path, &mut features_frame)?;
    files.push(file_record(
        &request.output_dir,
        &features_path,
        feature_rows,
    )?);

    if request.include_raw_interval_candles {
        for interval in [PriceInterval::FiveMinute, PriceInterval::OneHour] {
            let rows = data
                .interval_prices
                .iter()
                .filter(|row| row.interval == interval)
                .cloned()
                .collect::<Vec<_>>();
            if rows.is_empty() {
                continue;
            }
            let mut frame = interval_prices_frame(&rows)?;
            let file_name = match interval {
                PriceInterval::FiveMinute => "interval_prices_5m.parquet",
                PriceInterval::OneHour => "interval_prices_1h.parquet",
                PriceInterval::SixHour => "interval_prices_6h.parquet",
                PriceInterval::TwentyFourHour => "interval_prices_24h.parquet",
            };
            let path = request.output_dir.join(file_name);
            let row_count = write_parquet(&path, &mut frame)?;
            files.push(file_record(&request.output_dir, &path, row_count)?);
        }
    }

    if request.include_predictions && !data.predictions.is_empty() {
        let mut frame = predictions_frame(&data.predictions)?;
        let path = request.output_dir.join("predictions.parquet");
        let row_count = write_parquet(&path, &mut frame)?;
        files.push(file_record(&request.output_dir, &path, row_count)?);
    }

    if request.include_outcomes && !data.outcomes.is_empty() {
        let mut frame = outcomes_frame(&data.outcomes)?;
        let path = request.output_dir.join("recommendation_outcomes.parquet");
        let row_count = write_parquet(&path, &mut frame)?;
        files.push(file_record(&request.output_dir, &path, row_count)?);
    }

    let manifest = ReportManifest {
        report_id,
        generated_at,
        source_window_start: request.window_start,
        source_window_end: request.window_end,
        raw_candle_window_start: request
            .include_raw_interval_candles
            .then_some(request.window_start),
        raw_candle_window_end: request
            .include_raw_interval_candles
            .then_some(request.window_end),
        feature_set_version: request.feature_set_version.clone(),
        strategy_versions: distinct_strategy_versions(&data.predictions),
        execution_modes: Vec::new(),
        files,
    };
    write_manifest(&request.output_dir, &manifest)?;

    Ok(DatasetExportResult {
        manifest,
        output_dir: request.output_dir.clone(),
    })
}

fn features_frame(rows: &[FeatureVector]) -> Result<DataFrame, AnalyticsError> {
    let mut keys = BTreeSet::new();
    for row in rows {
        for key in row.values.keys() {
            if !is_future_label_key(key) {
                keys.insert(key.clone());
            }
        }
    }

    let mut columns: Vec<Column> = vec![
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
            "feature_set_version".into(),
            rows.iter()
                .map(|row| row.feature_set_version.clone())
                .collect::<Vec<_>>(),
        )
        .into(),
    ];

    for key in keys {
        let values = rows
            .iter()
            .map(|row| {
                row.values
                    .get(&key)
                    .map(serde_json::Value::to_string)
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();
        columns.push(Series::new(key.into(), values).into());
    }

    Ok(DataFrame::new(columns)?)
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
                .map(|row| {
                    serde_json::to_string(&row.interval)
                        .unwrap()
                        .trim_matches('"')
                        .to_string()
                })
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
            "horizon_secs".into(),
            rows.iter().map(|row| row.horizon_secs).collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "side".into(),
            rows.iter()
                .map(|row| {
                    serde_json::to_string(&row.side)
                        .unwrap()
                        .trim_matches('"')
                        .to_string()
                })
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "expected_return".into(),
            rows.iter()
                .map(|row| row.expected_return.get())
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
            "action".into(),
            rows.iter()
                .map(|row| {
                    serde_json::to_string(&row.recommendation.action)
                        .unwrap()
                        .trim_matches('"')
                        .to_string()
                })
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
        Series::new(
            "actual_return".into(),
            rows.iter()
                .map(|row| row.outcome.actual_return.map(|value| value.get()))
                .collect::<Vec<_>>(),
        )
        .into(),
        Series::new(
            "actual_net_gp".into(),
            rows.iter()
                .map(|row| row.outcome.actual_net_gp.map(|value| value.0))
                .collect::<Vec<_>>(),
        )
        .into(),
    ])?)
}

fn validate_window(
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
) -> Result<(), AnalyticsError> {
    if window_end <= window_start {
        return Err(AnalyticsError::InvalidWindow);
    }
    Ok(())
}

fn write_manifest(output_dir: &Path, manifest: &ReportManifest) -> Result<(), AnalyticsError> {
    let path = output_dir.join("manifest.json");
    let bytes = serde_json::to_vec_pretty(manifest)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn file_record(
    output_dir: &Path,
    path: &Path,
    row_count: u64,
) -> Result<ReportFile, AnalyticsError> {
    let relative = path
        .strip_prefix(output_dir)
        .map_err(|_| AnalyticsError::MissingFileName(path.to_path_buf()))?
        .to_path_buf();
    Ok(ReportFile {
        path: relative,
        sha256: sha256_file(path)?,
        row_count,
    })
}

fn distinct_strategy_versions(rows: &[StoredPrediction]) -> Vec<StrategyVersionRecord> {
    let mut set = BTreeSet::new();
    for row in rows {
        set.insert((row.strategy_id.0.clone(), row.model_version.0.clone()));
    }
    set.into_iter()
        .map(|(strategy_id, model_version)| StrategyVersionRecord {
            strategy_id,
            model_version,
        })
        .collect()
}

fn is_future_label_key(key: &str) -> bool {
    key.starts_with("future_")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::{TimeZone, Utc};
    use tempfile::tempdir;

    use super::{DatasetExportData, DatasetExportRequest, export_feature_dataset, validate_window};
    use crate::parquet::read_parquet;
    use grand_edge_domain::{
        FeatureVector, Gp, IntervalPrice, ItemId, PriceInterval, Probability, Rate, Recommendation,
        RecommendationAction, RecommendationExplanation, RecommendationId, RecommendationOutcome,
        StrategyId, StructuredRecommendationExplanation, UserId,
    };
    use grand_edge_storage::{EvaluatedRecommendationRecord, StoredPrediction};
    use uuid::Uuid;

    #[test]
    fn feature_export_writes_parquet_fixture() {
        let dir = tempdir().unwrap();
        let request = DatasetExportRequest {
            output_dir: dir.path().to_path_buf(),
            feature_set_version: "features_v1".to_string(),
            item_ids: None,
            window_start: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 2, 0, 0, 0).unwrap(),
            include_predictions: true,
            include_outcomes: true,
            include_raw_interval_candles: true,
        };

        let result = export_feature_dataset(&request, fixture_data()).unwrap();
        assert!(dir.path().join("features.parquet").is_file());
        let frame = read_parquet(&dir.path().join("features.parquet")).unwrap();
        assert_eq!(frame.height(), 1);
        assert!(
            result
                .manifest
                .files
                .iter()
                .any(|file| file.path == PathBuf::from("features.parquet"))
        );
    }

    #[test]
    fn raw_interval_export_preserves_observed_high_low_side_fields() {
        let dir = tempdir().unwrap();
        let request = DatasetExportRequest {
            output_dir: dir.path().to_path_buf(),
            feature_set_version: "features_v1".to_string(),
            item_ids: None,
            window_start: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 2, 0, 0, 0).unwrap(),
            include_predictions: false,
            include_outcomes: false,
            include_raw_interval_candles: true,
        };

        export_feature_dataset(&request, fixture_data()).unwrap();
        let frame = read_parquet(&dir.path().join("interval_prices_1h.parquet")).unwrap();
        assert!(
            frame
                .get_column_names()
                .iter()
                .any(|name| *name == "high_price_volume")
        );
        assert!(
            frame
                .get_column_names()
                .iter()
                .any(|name| *name == "low_price_volume")
        );
    }

    #[test]
    fn manifest_records_hashes_and_row_counts() {
        let dir = tempdir().unwrap();
        let request = DatasetExportRequest {
            output_dir: dir.path().to_path_buf(),
            feature_set_version: "features_v1".to_string(),
            item_ids: None,
            window_start: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 2, 0, 0, 0).unwrap(),
            include_predictions: true,
            include_outcomes: true,
            include_raw_interval_candles: true,
        };

        let result = export_feature_dataset(&request, fixture_data()).unwrap();
        assert!(
            result
                .manifest
                .files
                .iter()
                .all(|file| file.row_count > 0 && file.sha256.len() == 64)
        );
    }

    #[test]
    fn export_does_not_include_future_label_columns_in_features() {
        let dir = tempdir().unwrap();
        let request = DatasetExportRequest {
            output_dir: dir.path().to_path_buf(),
            feature_set_version: "features_v1".to_string(),
            item_ids: None,
            window_start: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 2, 0, 0, 0).unwrap(),
            include_predictions: false,
            include_outcomes: false,
            include_raw_interval_candles: false,
        };

        export_feature_dataset(&request, fixture_data()).unwrap();
        let frame = read_parquet(&dir.path().join("features.parquet")).unwrap();
        assert!(
            !frame
                .get_column_names()
                .iter()
                .any(|name| name.starts_with("future_"))
        );
    }

    #[test]
    fn report_rejects_window_end_before_start() {
        assert!(
            validate_window(
                Utc.with_ymd_and_hms(2026, 6, 2, 0, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            )
            .is_err()
        );
    }

    fn fixture_data() -> DatasetExportData {
        let mut values = serde_json::Map::new();
        values.insert("spread_pct".to_string(), serde_json::json!(0.02));
        values.insert("future_return_6h".to_string(), serde_json::json!(0.5));

        DatasetExportData {
            features: vec![FeatureVector {
                item_id: ItemId(4151),
                as_of: Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap(),
                feature_set_version: "features_v1".to_string(),
                values,
            }],
            interval_prices: vec![IntervalPrice {
                item_id: ItemId(4151),
                bucket_start: Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap(),
                interval: PriceInterval::OneHour,
                avg_high_price: Some(Gp(100_000)),
                high_price_volume: 42,
                avg_low_price: Some(Gp(99_500)),
                low_price_volume: 36,
            }],
            predictions: vec![StoredPrediction {
                strategy_id: StrategyId::new("momentum_v1").unwrap(),
                model_version: grand_edge_domain::ModelVersion::new("v1").unwrap(),
                item_id: ItemId(4151),
                as_of: Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap(),
                horizon_secs: 21_600,
                side: grand_edge_domain::SignalSide::Buy,
                expected_return: Rate::new(0.03).unwrap(),
                confidence: Probability::new(0.7).unwrap(),
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
                    as_of: Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap(),
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
                        graph_version: None,
                        graph_context: None,
                        strategy_votes: Vec::new(),
                        score_components: Vec::new(),
                        accuracy_snapshot: None,
                        structured_explanation: StructuredRecommendationExplanation::default(),
                    },
                },
                outcome: RecommendationOutcome {
                    recommendation_id: RecommendationId(Uuid::new_v4()),
                    evaluated_at: Utc.with_ymd_and_hms(2026, 6, 1, 18, 0, 0).unwrap(),
                    horizon_secs: grand_edge_domain::HorizonSecs(21_600),
                    actual_return: Some(Rate::new(0.02).unwrap()),
                    actual_net_gp: Some(Gp(900)),
                    direction_correct: Some(true),
                    hit_take_profit: false,
                    hit_stop_loss: false,
                    max_favourable_excursion: Some(Rate::new(0.04).unwrap()),
                    max_adverse_excursion: Some(Rate::new(-0.01).unwrap()),
                    outcome_label: grand_edge_domain::OutcomeLabel::Win,
                },
            }],
        }
    }
}
