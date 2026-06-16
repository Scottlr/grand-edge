use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use grand_edge_domain::{
    ExecutionMode, Gp, ModelVersion, OutcomeLabel, Probability, Rate, ReasonAtom,
    ReasonOutcomeSummary, RecommendationAction, RecommendationOutcome, StrategySignal,
};
use grand_edge_storage::Storage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::MetricsError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasonMetricsWindow {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub min_sample_size: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasonOutcomeInput {
    pub recommendation_id: Uuid,
    pub model_version: ModelVersion,
    pub recommendation_action: RecommendationAction,
    pub execution_mode: Option<ExecutionMode>,
    pub confidence_bucket: Option<String>,
    pub reason_atom: ReasonAtom,
    pub outcome: RecommendationOutcome,
    pub prediction_confidence: Option<Probability>,
}

pub fn compute_reason_outcome_summaries(
    inputs: &[ReasonOutcomeInput],
    window: ReasonMetricsWindow,
) -> Result<Vec<ReasonOutcomeSummary>, MetricsError> {
    let mut grouped = BTreeMap::<ReasonGroupingKey, Vec<&ReasonOutcomeInput>>::new();
    for input in inputs {
        grouped
            .entry(ReasonGroupingKey::from_input(input))
            .or_default()
            .push(input);
    }

    grouped
        .into_iter()
        .map(|(key, values)| build_summary(key, &values, &window))
        .collect()
}

pub async fn refresh_reason_outcomes(
    storage: &Storage,
    window: ReasonMetricsWindow,
) -> Result<Vec<ReasonOutcomeSummary>, MetricsError> {
    let records = storage
        .recommendations()
        .list_evaluated_between(window.window_start, window.window_end)
        .await?;
    let mut inputs = Vec::new();

    for record in records {
        let recommendation = record.recommendation;
        let outcome = record.outcome;
        let reason_atoms = &recommendation
            .explanation
            .structured_explanation
            .reason_atoms;
        if reason_atoms.is_empty() {
            continue;
        }

        let Some(signal) = selected_signal(
            recommendation.action,
            &recommendation.explanation.strategy_votes,
        ) else {
            continue;
        };
        let execution_mode = execution_mode_from_signal(signal);
        let confidence_bucket = confidence_bucket(recommendation.prediction_confidence);

        for atom in reason_atoms {
            inputs.push(ReasonOutcomeInput {
                recommendation_id: recommendation.recommendation_id.0,
                model_version: signal.model_version.clone(),
                recommendation_action: recommendation.action,
                execution_mode,
                confidence_bucket: confidence_bucket.clone(),
                reason_atom: atom.clone(),
                outcome: outcome.clone(),
                prediction_confidence: recommendation.prediction_confidence,
            });
        }
    }

    let summaries = compute_reason_outcome_summaries(&inputs, window)?;
    storage
        .reason_outcomes()
        .upsert_reason_outcome_summaries(&summaries)
        .await?;
    Ok(summaries)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ReasonGroupingKey {
    reason_type: String,
    reason_key: String,
    model_version: String,
    recommendation_action: String,
    execution_mode: Option<String>,
    confidence_bucket: Option<String>,
}

impl ReasonGroupingKey {
    fn from_input(input: &ReasonOutcomeInput) -> Self {
        Self {
            reason_type: serde_json::to_string(&input.reason_atom.reason_type)
                .expect("reason type serializes"),
            reason_key: input.reason_atom.reason_key.clone(),
            model_version: input.model_version.0.clone(),
            recommendation_action: serde_json::to_string(&input.recommendation_action)
                .expect("action serializes"),
            execution_mode: input
                .execution_mode
                .map(|value| serde_json::to_string(&value).expect("execution mode serializes")),
            confidence_bucket: input.confidence_bucket.clone(),
        }
    }
}

fn build_summary(
    key: ReasonGroupingKey,
    inputs: &[&ReasonOutcomeInput],
    window: &ReasonMetricsWindow,
) -> Result<ReasonOutcomeSummary, MetricsError> {
    let evaluable = inputs
        .iter()
        .copied()
        .filter(|input| input.outcome.outcome_label != OutcomeLabel::Unevaluable)
        .collect::<Vec<_>>();
    let sample_size = evaluable.len() as i64;
    let win_rate = if evaluable.is_empty() {
        None
    } else {
        Some(Probability::new(
            evaluable
                .iter()
                .filter(|input| input.outcome.outcome_label == OutcomeLabel::Win)
                .count() as f64
                / evaluable.len() as f64,
        )?)
    };
    let avg_actual_return = average_rate(
        evaluable
            .iter()
            .filter_map(|input| input.outcome.actual_return.map(|value| value.get())),
    )?;
    let avg_net_gp = average_gp(
        evaluable
            .iter()
            .filter_map(|input| input.outcome.actual_net_gp.map(|value| value.as_i64())),
    );
    let calibration_error = average_f64(evaluable.iter().filter_map(|input| {
        let realized = match input.outcome.outcome_label {
            OutcomeLabel::Win => 1.0,
            OutcomeLabel::Loss | OutcomeLabel::BreakEven | OutcomeLabel::Expired => 0.0,
            OutcomeLabel::Unevaluable => return None,
        };
        let predicted = input.prediction_confidence?.get();
        Some((predicted - realized).abs())
    }));

    Ok(ReasonOutcomeSummary {
        reason_type: serde_json::from_str(&key.reason_type)?,
        reason_key: key.reason_key,
        model_version: ModelVersion::new(key.model_version)?,
        recommendation_action: serde_json::from_str(&key.recommendation_action)?,
        execution_mode: key
            .execution_mode
            .map(|value| serde_json::from_str(&value))
            .transpose()?,
        confidence_bucket: key.confidence_bucket,
        window_start: window.window_start,
        window_end: window.window_end,
        sample_size,
        publishable: sample_size >= window.min_sample_size as i64,
        win_rate,
        avg_actual_return,
        avg_net_gp,
        calibration_error,
    })
}

fn selected_signal<'a>(
    action: RecommendationAction,
    strategy_votes: &'a [StrategySignal],
) -> Option<&'a StrategySignal> {
    strategy_votes
        .iter()
        .find(|signal| signal_matches_action(signal, action))
        .or_else(|| strategy_votes.first())
}

fn signal_matches_action(signal: &StrategySignal, action: RecommendationAction) -> bool {
    use grand_edge_domain::SignalSide;

    matches!(
        (action, signal.side),
        (RecommendationAction::Buy, SignalSide::Buy)
            | (RecommendationAction::Add, SignalSide::Buy)
            | (RecommendationAction::Hold, SignalSide::Hold)
            | (RecommendationAction::Hold, SignalSide::Buy)
            | (RecommendationAction::Cashout, SignalSide::Cashout)
            | (RecommendationAction::Cashout, SignalSide::Sell)
            | (RecommendationAction::Avoid, SignalSide::Avoid)
            | (RecommendationAction::Avoid, SignalSide::Sell)
            | (RecommendationAction::Watch, SignalSide::Watch)
            | (RecommendationAction::Watch, SignalSide::Buy)
    )
}

fn execution_mode_from_signal(signal: &StrategySignal) -> Option<ExecutionMode> {
    signal
        .explanation
        .get("execution_mode")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
}

fn confidence_bucket(confidence: Option<Probability>) -> Option<String> {
    let confidence = confidence?;
    let lower = (confidence.get() * 10.0).floor() / 10.0;
    let upper = (lower + 0.1).min(1.0);
    Some(format!("{lower:.1}-{upper:.1}"))
}

fn average_rate(values: impl Iterator<Item = f64>) -> Result<Option<Rate>, MetricsError> {
    Ok(average_f64(values).map(Rate::new).transpose()?)
}

fn average_gp(values: impl Iterator<Item = i64>) -> Option<Gp> {
    average_f64(values.map(|value| value as f64)).map(|value| Gp(value.round() as i64))
}

fn average_f64(values: impl Iterator<Item = f64>) -> Option<f64> {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }

    Some(values.iter().sum::<f64>() / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};
    use grand_edge_domain::{
        ExecutionMode, Gp, HorizonSecs, ModelVersion, OutcomeLabel, Probability, Rate, ReasonAtom,
        ReasonDirection, ReasonType, RecommendationAction, RecommendationId,
    };
    use uuid::Uuid;

    use super::{ReasonMetricsWindow, ReasonOutcomeInput, compute_reason_outcome_summaries};

    #[test]
    fn reason_metrics_group_by_stable_reason_key() {
        let summaries = compute_reason_outcome_summaries(
            &[
                input(
                    "liquidity:volume_capacity",
                    RecommendationAction::Buy,
                    OutcomeLabel::Win,
                ),
                input(
                    "liquidity:volume_capacity",
                    RecommendationAction::Buy,
                    OutcomeLabel::Loss,
                ),
            ],
            window(1),
        )
        .unwrap();

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].reason_key, "liquidity:volume_capacity");
        assert_eq!(summaries[0].sample_size, 2);
    }

    #[test]
    fn reason_metrics_exclude_unevaluable_from_win_rate() {
        let summaries = compute_reason_outcome_summaries(
            &[
                input(
                    "model_signal:spread_edge",
                    RecommendationAction::Buy,
                    OutcomeLabel::Win,
                ),
                input(
                    "model_signal:spread_edge",
                    RecommendationAction::Buy,
                    OutcomeLabel::Unevaluable,
                ),
            ],
            window(1),
        )
        .unwrap();

        assert_eq!(summaries[0].sample_size, 1);
        assert_eq!(summaries[0].win_rate.unwrap().get(), 1.0);
    }

    #[test]
    fn reason_metrics_compute_avg_net_gp() {
        let first = input(
            "risk:profile_limit",
            RecommendationAction::Buy,
            OutcomeLabel::Win,
        );
        let mut second = input(
            "risk:profile_limit",
            RecommendationAction::Buy,
            OutcomeLabel::Loss,
        );
        second.outcome.actual_net_gp = Some(Gp(-4_000));

        let summaries = compute_reason_outcome_summaries(&[first, second], window(1)).unwrap();
        assert_eq!(summaries[0].avg_net_gp, Some(Gp(1_000)));
    }

    #[test]
    fn reason_metrics_compute_calibration_error() {
        let mut first = input(
            "cost:tax_and_spread",
            RecommendationAction::Buy,
            OutcomeLabel::Win,
        );
        first.prediction_confidence = Some(Probability::new(0.8).unwrap());
        let mut second = input(
            "cost:tax_and_spread",
            RecommendationAction::Buy,
            OutcomeLabel::Loss,
        );
        second.prediction_confidence = Some(Probability::new(0.3).unwrap());

        let summaries = compute_reason_outcome_summaries(&[first, second], window(1)).unwrap();
        assert!(
            (summaries[0].calibration_error.unwrap() - 0.25).abs() < 1e-9,
            "expected calibration error near 0.25, got {:?}",
            summaries[0].calibration_error
        );
    }

    #[test]
    fn reason_metrics_require_min_sample_size_for_publishable_summary() {
        let summaries = compute_reason_outcome_summaries(
            &[input(
                "data_quality:freshness_completeness",
                RecommendationAction::Watch,
                OutcomeLabel::Expired,
            )],
            window(2),
        )
        .unwrap();

        assert!(!summaries[0].publishable);
    }

    #[test]
    fn reason_metrics_keep_buy_and_sell_groups_separate() {
        let buy = input(
            "model_signal:spread_edge",
            RecommendationAction::Buy,
            OutcomeLabel::Win,
        );
        let sell = input(
            "model_signal:spread_edge",
            RecommendationAction::Cashout,
            OutcomeLabel::Win,
        );

        let summaries = compute_reason_outcome_summaries(&[buy, sell], window(1)).unwrap();
        assert_eq!(summaries.len(), 2);
    }

    #[test]
    fn reason_metrics_zero_denominator_is_none_not_zero() {
        let summaries = compute_reason_outcome_summaries(
            &[input(
                "liquidity:volume_capacity",
                RecommendationAction::Buy,
                OutcomeLabel::Unevaluable,
            )],
            window(1),
        )
        .unwrap();

        assert_eq!(summaries[0].sample_size, 0);
        assert_eq!(summaries[0].win_rate, None);
        assert_eq!(summaries[0].avg_actual_return, None);
        assert_eq!(summaries[0].avg_net_gp, None);
    }

    fn input(
        reason_key: &str,
        recommendation_action: RecommendationAction,
        outcome_label: OutcomeLabel,
    ) -> ReasonOutcomeInput {
        ReasonOutcomeInput {
            recommendation_id: Uuid::new_v4(),
            model_version: ModelVersion::new("2026-06-16.1").unwrap(),
            recommendation_action,
            execution_mode: Some(ExecutionMode::ConservativeInstant),
            confidence_bucket: Some("0.6-0.7".to_string()),
            reason_atom: ReasonAtom {
                reason_type: ReasonType::ModelSignal,
                reason_key: reason_key.to_string(),
                label: "Fixture".to_string(),
                direction: ReasonDirection::Positive,
                weight: 0.6,
                evidence: serde_json::json!({}),
            },
            outcome: grand_edge_domain::RecommendationOutcome {
                recommendation_id: RecommendationId(Uuid::new_v4()),
                evaluated_at: Utc.with_ymd_and_hms(2026, 6, 16, 18, 0, 0).unwrap(),
                horizon_secs: HorizonSecs(3_600),
                actual_return: Some(Rate::new(0.02).unwrap()),
                actual_net_gp: Some(Gp(6_000)),
                direction_correct: Some(true),
                hit_take_profit: false,
                hit_stop_loss: false,
                max_favourable_excursion: Some(Rate::new(0.03).unwrap()),
                max_adverse_excursion: Some(Rate::new(-0.01).unwrap()),
                outcome_label,
            },
            prediction_confidence: Some(Probability::new(0.7).unwrap()),
        }
    }

    fn window(min_sample_size: usize) -> ReasonMetricsWindow {
        ReasonMetricsWindow {
            window_start: Utc.with_ymd_and_hms(2026, 6, 9, 0, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap() + Duration::hours(1),
            min_sample_size,
        }
    }
}
