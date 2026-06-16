use grand_edge_storage::{StoredPaperBet, StoredPrediction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationBucket {
    pub min_probability: f64,
    pub max_probability: f64,
    pub sample_size: i64,
    pub predicted_probability: Option<f64>,
    pub realized_hit_rate: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CalibrationMetrics {
    pub buckets: Vec<CalibrationBucket>,
    pub execution_confidence_buckets: Vec<CalibrationBucket>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExecutionQualityMetrics {
    pub conservative_instant_sample_size: i64,
    pub passive_estimated_sample_size: i64,
    pub haircut_passive_sample_size: i64,
    pub avg_estimated_fill_probability: Option<f64>,
    pub realized_fill_rate: Option<f64>,
    pub estimated_vs_realized_fill_error: Option<f64>,
    pub liquidity_bucket_metrics: Vec<LiquidityBucketMetric>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiquidityBucketMetric {
    pub bucket_name: String,
    pub min_liquidity_confidence: Option<f64>,
    pub max_liquidity_confidence: Option<f64>,
    pub sample_size: i64,
    pub directional_accuracy: Option<f64>,
    pub net_profit_gp: i64,
    pub win_rate: Option<f64>,
}

pub fn calibration_buckets(outcomes: &[bool], probabilities: &[f64]) -> Vec<CalibrationBucket> {
    if outcomes.is_empty() || outcomes.len() != probabilities.len() {
        return Vec::new();
    }

    let mut buckets = Vec::new();
    for bucket_index in 0..10 {
        let min = bucket_index as f64 / 10.0;
        let max = (bucket_index + 1) as f64 / 10.0;
        let bucket_values = probabilities
            .iter()
            .copied()
            .zip(outcomes.iter().copied())
            .filter(|(probability, _)| {
                if bucket_index == 9 {
                    *probability >= min && *probability <= max
                } else {
                    *probability >= min && *probability < max
                }
            })
            .collect::<Vec<_>>();

        let sample_size = bucket_values.len() as i64;
        let predicted_probability = average(bucket_values.iter().map(|(value, _)| *value));
        let realized_hit_rate = average(
            bucket_values
                .iter()
                .map(|(_, outcome)| if *outcome { 1.0 } else { 0.0 }),
        );

        buckets.push(CalibrationBucket {
            min_probability: min,
            max_probability: max,
            sample_size,
            predicted_probability,
            realized_hit_rate,
        });
    }

    buckets
}

pub fn execution_quality(
    predictions: &[StoredPrediction],
    paper_bets: &[StoredPaperBet],
    matched_predictions: &[Option<usize>],
) -> ExecutionQualityMetrics {
    let mut metrics = ExecutionQualityMetrics::default();
    let mut estimated_fill_probabilities = Vec::new();
    let mut liquidity_groups: std::collections::BTreeMap<&'static str, Vec<usize>> =
        std::collections::BTreeMap::new();

    for (index, paper_bet) in paper_bets.iter().enumerate() {
        let execution_mode = paper_bet
            .explanation
            .get("execution_mode")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown");
        match execution_mode {
            "conservative_instant" => metrics.conservative_instant_sample_size += 1,
            "passive_estimated" => metrics.passive_estimated_sample_size += 1,
            "haircut_passive" => metrics.haircut_passive_sample_size += 1,
            _ => {}
        }

        if let Some(prediction_index) = matched_predictions.get(index).and_then(|value| *value) {
            if let Some(probability) = predictions[prediction_index]
                .explanation
                .get("estimated_fill_probability")
                .and_then(|value| value.as_f64())
            {
                estimated_fill_probabilities.push(probability);
            }
        }

        let observed_volume = paper_bet
            .explanation
            .get("entry")
            .and_then(|value| value.get("observed_volume"))
            .and_then(|value| value.as_i64());
        let bucket = match observed_volume {
            Some(volume) if volume >= 500 => "high",
            Some(volume) if volume >= 100 => "medium",
            Some(_) => "low",
            None => "unknown",
        };
        liquidity_groups.entry(bucket).or_default().push(index);
    }

    let realized_fill_rate = if paper_bets.is_empty() {
        None
    } else {
        Some(
            paper_bets
                .iter()
                .filter(|paper_bet| paper_bet.exit_time.is_some())
                .count() as f64
                / paper_bets.len() as f64,
        )
    };
    let avg_estimated_fill_probability = average(estimated_fill_probabilities.iter().copied());
    metrics.avg_estimated_fill_probability = avg_estimated_fill_probability;
    metrics.realized_fill_rate = realized_fill_rate;
    metrics.estimated_vs_realized_fill_error =
        match (avg_estimated_fill_probability, realized_fill_rate) {
            (Some(estimated), Some(realized)) => Some((estimated - realized).abs()),
            _ => None,
        };

    metrics.liquidity_bucket_metrics = liquidity_groups
        .into_iter()
        .map(|(bucket_name, indexes)| {
            let profits = indexes
                .iter()
                .filter_map(|index| paper_bets[*index].realized_profit_gp)
                .collect::<Vec<_>>();
            let wins = profits.iter().filter(|profit| **profit > 0).count();
            LiquidityBucketMetric {
                bucket_name: bucket_name.to_string(),
                min_liquidity_confidence: None,
                max_liquidity_confidence: None,
                sample_size: indexes.len() as i64,
                directional_accuracy: None,
                net_profit_gp: profits.iter().sum(),
                win_rate: if profits.is_empty() {
                    None
                } else {
                    Some(wins as f64 / profits.len() as f64)
                },
            }
        })
        .collect();

    metrics
}

fn average(values: impl Iterator<Item = f64>) -> Option<f64> {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }

    Some(values.iter().sum::<f64>() / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{Gp, ItemId, ModelVersion, StrategyId};
    use uuid::Uuid;

    use super::{calibration_buckets, execution_quality};
    use grand_edge_storage::{StoredPaperBet, StoredPrediction};

    #[test]
    fn liquidity_bucket_metrics_keep_sample_size_visible() {
        let paper_bets = vec![paper_bet(50, Some(100)), paper_bet(600, Some(-50))];
        let metrics = execution_quality(&[], &paper_bets, &[None, None]);
        assert_eq!(metrics.liquidity_bucket_metrics.len(), 2);
        assert!(
            metrics
                .liquidity_bucket_metrics
                .iter()
                .all(|bucket| bucket.sample_size == 1)
        );
    }

    #[test]
    fn estimated_fill_calibration_handles_missing_samples() {
        let metrics = execution_quality(&[], &[], &[]);
        assert_eq!(metrics.avg_estimated_fill_probability, None);
        assert_eq!(metrics.estimated_vs_realized_fill_error, None);
    }

    #[test]
    fn execution_quality_groups_by_execution_mode() {
        let predictions = vec![prediction()];
        let paper_bets = vec![
            paper_bet_with_mode("conservative_instant"),
            paper_bet_with_mode("passive_estimated"),
        ];
        let metrics = execution_quality(&predictions, &paper_bets, &[Some(0), Some(0)]);
        assert_eq!(metrics.conservative_instant_sample_size, 1);
        assert_eq!(metrics.passive_estimated_sample_size, 1);
    }

    #[test]
    fn calibration_buckets_preserve_bucket_count() {
        let buckets = calibration_buckets(&[true, false], &[0.2, 0.8]);
        assert_eq!(buckets.len(), 10);
    }

    fn prediction() -> StoredPrediction {
        StoredPrediction {
            strategy_id: StrategyId::new("momentum_v1").unwrap(),
            model_version: ModelVersion::new("v1").unwrap(),
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            horizon_secs: 21_600,
            side: grand_edge_domain::SignalSide::Buy,
            expected_return: grand_edge_domain::Rate::new(0.02).unwrap(),
            confidence: grand_edge_domain::Probability::new(0.8).unwrap(),
            expected_net_gp_per_unit: Gp(1200),
            target_entry: None,
            target_exit: None,
            stop_loss: None,
            take_profit: None,
            max_quantity: None,
            explanation: serde_json::json!({ "estimated_fill_probability": 0.6 }),
        }
    }

    fn paper_bet(observed_volume: i64, realized_profit_gp: Option<i64>) -> StoredPaperBet {
        let mut value = paper_bet_with_mode("passive_estimated");
        value.realized_profit_gp = realized_profit_gp;
        value.explanation["entry"]["observed_volume"] = serde_json::json!(observed_volume);
        value
    }

    fn paper_bet_with_mode(mode: &str) -> StoredPaperBet {
        StoredPaperBet {
            bet_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            recommendation_id: None,
            strategy_id: StrategyId::new("momentum_v1").unwrap(),
            model_version: ModelVersion::new("v1").unwrap(),
            item_id: ItemId(4151),
            entry_time: Utc.with_ymd_and_hms(2026, 6, 16, 13, 0, 0).unwrap(),
            entry_price: Gp(100_000),
            quantity: 10,
            target_exit: None,
            stop_loss: None,
            exit_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 14, 0, 0).unwrap()),
            exit_price: Some(Gp(101_000)),
            tax_paid: 0,
            realized_profit_gp: Some(100),
            realized_roi: Some(0.01),
            max_drawdown: Some(0.05),
            hit_reason: None,
            status: "closed".to_string(),
            explanation: serde_json::json!({
                "execution_mode": mode,
                "entry": {
                    "observed_volume": 120
                }
            }),
        }
    }
}
