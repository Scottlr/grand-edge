use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use grand_edge_domain::{
    ExecutionMode, GraphPath, GraphRecommendationAction, OutcomeLabel, RecommendationOutcome,
};
use grand_edge_simulator::BlastScenarioMode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphMetricDimension {
    pub graph_version: String,
    pub graph_action: GraphRecommendationAction,
    pub edge_type: String,
    pub source_type: String,
    pub path_length: usize,
    pub execution_mode: String,
    pub confidence_bucket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPathOutcome {
    pub recommendation_id: Uuid,
    pub graph_version: String,
    pub graph_action: GraphRecommendationAction,
    pub path: GraphPath,
    pub path_confidence: f64,
    pub edge_type: String,
    pub source_type: String,
    pub execution_mode: String,
    pub recommendation_outcome: RecommendationOutcome,
    pub graph_reason_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPathMetricSummary {
    pub graph_version: String,
    pub graph_action: GraphRecommendationAction,
    pub edge_type: String,
    pub source_type: String,
    pub path_length: usize,
    pub execution_mode: String,
    pub confidence_bucket: String,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub sample_size: i64,
    pub hit_rate: Option<f64>,
    pub avg_realized_return: Option<f64>,
    pub avg_realized_net_gp: Option<f64>,
    pub calibration_error: Option<f64>,
    pub avg_path_confidence: Option<f64>,
    pub insufficient_sample: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusOutcome {
    pub graph_version: String,
    pub scenario_mode: BlastScenarioMode,
    pub horizon_secs: i64,
    pub predicted_impact: f64,
    pub realized_return: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusMetricSummary {
    pub graph_version: String,
    pub scenario_mode: String,
    pub horizon_secs: i64,
    pub sample_size: i64,
    pub impact_mae: Option<f64>,
    pub directional_accuracy: Option<f64>,
    pub confidence_bucket_error: Option<f64>,
    pub insufficient_sample: bool,
}

pub fn graph_metric_dimension(outcome: &GraphPathOutcome) -> GraphMetricDimension {
    GraphMetricDimension {
        graph_version: outcome.graph_version.clone(),
        graph_action: outcome.graph_action,
        edge_type: outcome.edge_type.clone(),
        source_type: outcome.source_type.clone(),
        path_length: outcome.path.steps.len(),
        execution_mode: outcome.execution_mode.clone(),
        confidence_bucket: confidence_bucket(outcome.path_confidence),
    }
}

pub fn summarize_graph_path_outcomes(
    outcomes: &[GraphPathOutcome],
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
    min_sample_size: i64,
) -> Vec<GraphPathMetricSummary> {
    let mut grouped = BTreeMap::<String, Vec<&GraphPathOutcome>>::new();
    for outcome in outcomes
        .iter()
        .filter(|outcome| !outcome.graph_reason_keys.is_empty())
    {
        grouped
            .entry(group_key(&graph_metric_dimension(outcome)))
            .or_default()
            .push(outcome);
    }

    grouped
        .into_values()
        .map(|group| {
            let first = group[0];
            let sample_size = group.len() as i64;
            let actual_returns = group
                .iter()
                .filter_map(|outcome| {
                    outcome
                        .recommendation_outcome
                        .actual_return
                        .map(|value| value.get())
                })
                .collect::<Vec<_>>();
            let net_gp = group
                .iter()
                .filter_map(|outcome| {
                    outcome
                        .recommendation_outcome
                        .actual_net_gp
                        .map(|value| value.0 as f64)
                })
                .collect::<Vec<_>>();
            let hits = graph_hits(&group);
            let confidences = group
                .iter()
                .map(|outcome| outcome.path_confidence)
                .collect::<Vec<_>>();
            GraphPathMetricSummary {
                graph_version: first.graph_version.clone(),
                graph_action: first.graph_action,
                edge_type: first.edge_type.clone(),
                source_type: first.source_type.clone(),
                path_length: first.path.steps.len(),
                execution_mode: first.execution_mode.clone(),
                confidence_bucket: confidence_bucket(first.path_confidence),
                window_start,
                window_end,
                sample_size,
                hit_rate: graph_path_hit_rate_from_hits(&hits),
                avg_realized_return: mean(&actual_returns),
                avg_realized_net_gp: mean(&net_gp),
                calibration_error: graph_confidence_calibration_error(&confidences, &hits),
                avg_path_confidence: mean(&confidences),
                insufficient_sample: sample_size < min_sample_size,
            }
        })
        .collect()
}

pub fn summarize_blast_radius_outcomes(
    outcomes: &[BlastRadiusOutcome],
    min_sample_size: i64,
) -> Vec<BlastRadiusMetricSummary> {
    let mut grouped = BTreeMap::<(String, String, i64), Vec<&BlastRadiusOutcome>>::new();
    for outcome in outcomes {
        grouped
            .entry((
                outcome.graph_version.clone(),
                scenario_mode_key(outcome.scenario_mode).to_string(),
                outcome.horizon_secs,
            ))
            .or_default()
            .push(outcome);
    }

    grouped
        .into_iter()
        .map(|((graph_version, scenario_mode, horizon_secs), group)| {
            let sample_size = group.len() as i64;
            let predicted = group
                .iter()
                .map(|outcome| outcome.predicted_impact)
                .collect::<Vec<_>>();
            let realized = group
                .iter()
                .map(|outcome| outcome.realized_return)
                .collect::<Vec<_>>();
            let confidences = group
                .iter()
                .map(|outcome| outcome.confidence)
                .collect::<Vec<_>>();
            let hits = predicted
                .iter()
                .zip(realized.iter())
                .map(|(predicted, realized)| predicted.signum() == realized.signum())
                .collect::<Vec<_>>();

            BlastRadiusMetricSummary {
                graph_version,
                scenario_mode,
                horizon_secs,
                sample_size,
                impact_mae: blast_impact_mae(&predicted, &realized),
                directional_accuracy: graph_path_hit_rate_from_hits(&hits),
                confidence_bucket_error: graph_confidence_calibration_error(&confidences, &hits),
                insufficient_sample: sample_size < min_sample_size,
            }
        })
        .collect()
}

pub fn graph_path_hit_rate(outcomes: &[GraphPathOutcome]) -> Option<f64> {
    graph_path_hit_rate_from_hits(&graph_hits(&outcomes.iter().collect::<Vec<_>>()))
}

pub fn blast_impact_mae(predicted: &[f64], realized: &[f64]) -> Option<f64> {
    if predicted.len() != realized.len() || predicted.is_empty() {
        return None;
    }

    let errors = predicted
        .iter()
        .zip(realized.iter())
        .map(|(predicted, realized)| (predicted - realized).abs())
        .collect::<Vec<_>>();
    mean(&errors)
}

pub fn graph_confidence_calibration_error(
    predicted_confidence: &[f64],
    hits: &[bool],
) -> Option<f64> {
    if predicted_confidence.len() != hits.len() || predicted_confidence.is_empty() {
        return None;
    }

    let errors = predicted_confidence
        .iter()
        .zip(hits.iter())
        .map(|(confidence, hit)| (confidence - if *hit { 1.0 } else { 0.0 }).abs())
        .collect::<Vec<_>>();
    mean(&errors)
}

fn graph_hits(outcomes: &[&GraphPathOutcome]) -> Vec<bool> {
    outcomes
        .iter()
        .map(|outcome| match outcome.graph_action {
            GraphRecommendationAction::ExploitConversion => outcome
                .recommendation_outcome
                .actual_net_gp
                .is_some_and(|value| value.0 > 0),
            GraphRecommendationAction::AvoidBlastRadius => outcome
                .recommendation_outcome
                .actual_return
                .is_some_and(|value| value.get() <= 0.0),
            GraphRecommendationAction::CashoutBeforeContagion => matches!(
                outcome.recommendation_outcome.outcome_label,
                OutcomeLabel::Win | OutcomeLabel::BreakEven
            ),
            GraphRecommendationAction::WatchSecondOrder => outcome
                .recommendation_outcome
                .direction_correct
                .unwrap_or_else(|| {
                    outcome
                        .recommendation_outcome
                        .actual_return
                        .is_some_and(|value| value.get() > 0.0)
                }),
            _ => outcome
                .recommendation_outcome
                .actual_net_gp
                .is_some_and(|value| value.0 > 0),
        })
        .collect()
}

fn graph_path_hit_rate_from_hits(hits: &[bool]) -> Option<f64> {
    if hits.is_empty() {
        return None;
    }
    Some(hits.iter().filter(|hit| **hit).count() as f64 / hits.len() as f64)
}

fn group_key(dimension: &GraphMetricDimension) -> String {
    format!(
        "{}|{:?}|{}|{}|{}|{}|{}",
        dimension.graph_version,
        dimension.graph_action,
        dimension.edge_type,
        dimension.source_type,
        dimension.path_length,
        dimension.execution_mode,
        dimension.confidence_bucket
    )
}

fn confidence_bucket(confidence: f64) -> String {
    match confidence {
        value if value < 0.40 => "0_40".to_string(),
        value if value < 0.55 => "40_55".to_string(),
        value if value < 0.70 => "55_70".to_string(),
        value if value < 0.85 => "70_85".to_string(),
        _ => "85_100".to_string(),
    }
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then_some(values.iter().sum::<f64>() / values.len() as f64)
}

fn scenario_mode_key(mode: BlastScenarioMode) -> &'static str {
    match mode {
        BlastScenarioMode::Conservative => "conservative",
        BlastScenarioMode::Balanced => "balanced",
        BlastScenarioMode::Optimistic => "optimistic",
    }
}

pub fn execution_mode_key(mode: ExecutionMode) -> String {
    serde_json::to_string(&mode)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        ExecutionMode, Gp, GraphPath, GraphPathStep, GraphRecommendationAction, OutcomeLabel, Rate,
        RecommendationId, RecommendationOutcome,
    };
    use uuid::Uuid;

    use super::{
        BlastRadiusOutcome, GraphPathOutcome, blast_impact_mae, execution_mode_key,
        graph_confidence_calibration_error, summarize_blast_radius_outcomes,
        summarize_graph_path_outcomes,
    };

    #[test]
    fn graph_path_metrics_group_by_edge_source_and_length() {
        let summaries = summarize_graph_path_outcomes(
            &[
                outcome(
                    "ingredient_of",
                    "mechanical",
                    GraphRecommendationAction::BuyLinked,
                ),
                outcome(
                    "ingredient_of",
                    "learned",
                    GraphRecommendationAction::BuyLinked,
                ),
            ],
            Utc.with_ymd_and_hms(2026, 6, 9, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            1,
        );

        assert_eq!(summaries.len(), 2);
        assert_ne!(summaries[0].source_type, summaries[1].source_type);
    }

    #[test]
    fn blast_impact_mae_matches_fixture() {
        let mae = blast_impact_mae(&[0.10, -0.05], &[0.08, -0.01]).unwrap();
        assert!((mae - 0.03).abs() < 1e-9);
    }

    #[test]
    fn graph_confidence_calibration_empty_samples_return_none() {
        assert_eq!(graph_confidence_calibration_error(&[], &[]), None);
    }

    #[test]
    fn learned_and_mechanical_edges_are_separate_metric_groups() {
        let summaries = summarize_graph_path_outcomes(
            &[
                outcome(
                    "shock_transmits_to",
                    "mechanical",
                    GraphRecommendationAction::BuyLinked,
                ),
                outcome(
                    "shock_transmits_to",
                    "learned",
                    GraphRecommendationAction::BuyLinked,
                ),
            ],
            Utc.with_ymd_and_hms(2026, 6, 9, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            1,
        );
        assert_eq!(summaries.len(), 2);
    }

    #[test]
    fn non_graph_recommendations_are_excluded_from_graph_metrics() {
        let mut non_graph = outcome(
            "ingredient_of",
            "mechanical",
            GraphRecommendationAction::BuyLinked,
        );
        non_graph.graph_reason_keys.clear();
        let summaries = summarize_graph_path_outcomes(
            &[non_graph],
            Utc.with_ymd_and_hms(2026, 6, 9, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            1,
        );
        assert!(summaries.is_empty());
    }

    #[test]
    fn conversion_success_uses_net_gp_after_tax_and_execution() {
        let summary = summarize_graph_path_outcomes(
            &[outcome(
                "dose_conversion",
                "mechanical",
                GraphRecommendationAction::ExploitConversion,
            )],
            Utc.with_ymd_and_hms(2026, 6, 9, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            1,
        );
        assert_eq!(summary[0].hit_rate, Some(1.0));
    }

    #[test]
    fn blast_watch_success_uses_forecast_error_not_trade_pnl() {
        let summaries = summarize_blast_radius_outcomes(
            &[BlastRadiusOutcome {
                graph_version: "graph_v1".to_string(),
                scenario_mode: grand_edge_simulator::BlastScenarioMode::Balanced,
                horizon_secs: 21_600,
                predicted_impact: -0.04,
                realized_return: -0.03,
                confidence: 0.7,
            }],
            1,
        );
        assert_eq!(summaries[0].directional_accuracy, Some(1.0));
    }

    #[test]
    fn low_sample_graph_metric_sets_insufficient_sample() {
        let summaries = summarize_graph_path_outcomes(
            &[outcome(
                "ingredient_of",
                "mechanical",
                GraphRecommendationAction::BuyLinked,
            )],
            Utc.with_ymd_and_hms(2026, 6, 9, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            2,
        );
        assert!(summaries[0].insufficient_sample);
    }

    fn outcome(
        edge_type: &str,
        source_type: &str,
        graph_action: GraphRecommendationAction,
    ) -> GraphPathOutcome {
        GraphPathOutcome {
            recommendation_id: RecommendationId(Uuid::new_v4()).0,
            graph_version: "graph_v1".to_string(),
            graph_action,
            path: GraphPath {
                source_item_id: grand_edge_domain::ItemId(4151),
                target_item_id: grand_edge_domain::ItemId(11840),
                steps: vec![GraphPathStep {
                    from_item_id: grand_edge_domain::ItemId(4151),
                    to_item_id: grand_edge_domain::ItemId(11840),
                    edge_id: Uuid::new_v4(),
                    edge_type: grand_edge_domain::GraphEdgeType::IngredientOf,
                    confidence: 0.72,
                    weight: 0.45,
                }],
                path_confidence: 0.72,
                expected_impact: Some(0.03),
            },
            path_confidence: 0.72,
            edge_type: edge_type.to_string(),
            source_type: source_type.to_string(),
            execution_mode: execution_mode_key(ExecutionMode::PassiveEstimated),
            recommendation_outcome: RecommendationOutcome {
                recommendation_id: RecommendationId(Uuid::new_v4()),
                evaluated_at: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
                horizon_secs: grand_edge_domain::HorizonSecs(21_600),
                actual_return: Some(Rate::new(0.02).unwrap()),
                actual_net_gp: Some(Gp(1_200)),
                direction_correct: Some(true),
                hit_take_profit: false,
                hit_stop_loss: false,
                max_favourable_excursion: Some(Rate::new(0.03).unwrap()),
                max_adverse_excursion: Some(Rate::new(-0.01).unwrap()),
                outcome_label: OutcomeLabel::Win,
            },
            graph_reason_keys: vec!["graph:fixture".to_string()],
        }
    }
}
