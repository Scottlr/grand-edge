use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};
use grand_edge_domain::{
    FeatureSnapshot, Prediction, ReasonAtom, ReasonDirection, ReasonOutcomeSummary,
};
use grand_edge_storage::{
    LinkedPredictionRecord, RecommendationEvidenceRecord, RecommendationGraphLinkSummary,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::recommendations::view::RecommendationDto;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationEvidenceDto {
    pub recommendation_id: Uuid,
    pub item_id: i64,
    pub as_of: DateTime<Utc>,
    pub stages: Vec<EvidenceStageDto>,
    pub feature_snapshot: Option<FeatureSnapshotDto>,
    pub predictions: Vec<PredictionEvidenceDto>,
    pub prediction_links: Vec<PredictionLinkDto>,
    pub recommendation: RecommendationDto,
    pub graph_version: Option<String>,
    pub graph_paths: Vec<GraphPathDto>,
    pub graph_sources: Vec<GraphSourceDto>,
    pub explanation: StructuredExplanationDto,
    pub outcome: Option<RecommendationOutcomeDto>,
    pub reason_performance: Vec<ReasonPerformanceDto>,
    pub model_cards: Vec<ModelCardRefDto>,
    pub data_state: EvidenceDataStateDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStageKindDto {
    MarketData,
    FeatureSnapshot,
    GraphContext,
    Prediction,
    Recommendation,
    Explanation,
    OutcomeEvaluation,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStageStatusDto {
    Present,
    Pending,
    Degraded,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDataStateStatusDto {
    Live,
    Pending,
    Stale,
    Degraded,
    Empty,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceStageDto {
    pub kind: EvidenceStageKindDto,
    pub label: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub status: EvidenceStageStatusDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceDataStateDto {
    pub status: EvidenceDataStateStatusDto,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureSnapshotDto {
    pub feature_snapshot_id: Uuid,
    pub item_id: i64,
    pub as_of: DateTime<Utc>,
    pub feature_set_version: String,
    pub graph_version: Option<String>,
    pub source_window_start: DateTime<Utc>,
    pub source_window_end: DateTime<Utc>,
    pub features: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PredictionEvidenceDto {
    pub prediction_id: Uuid,
    pub feature_snapshot_id: Uuid,
    pub item_id: i64,
    pub as_of: DateTime<Utc>,
    pub horizon_secs: i64,
    pub model_id: String,
    pub model_version: String,
    pub predicted_direction: String,
    pub predicted_return: Option<f64>,
    pub confidence: f64,
    pub prediction_interval_low: Option<f64>,
    pub prediction_interval_high: Option<f64>,
    pub explanation: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PredictionLinkDto {
    pub prediction_id: Uuid,
    pub contribution_weight: f64,
    pub model_id: String,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StructuredExplanationDto {
    pub summary: String,
    pub reason_atoms: Vec<ReasonAtomDto>,
    pub invalidation_rules: Vec<crate::recommendations::view::InvalidationRuleDto>,
    pub graph_version: Option<String>,
    pub graph_reason_path_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReasonAtomDto {
    pub reason_type: String,
    pub reason_key: String,
    pub label: String,
    pub direction: String,
    pub weight: f64,
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationOutcomeDto {
    pub evaluated_at: DateTime<Utc>,
    pub horizon_secs: i64,
    pub actual_return: Option<f64>,
    pub actual_net_gp: Option<i64>,
    pub direction_correct: Option<bool>,
    pub hit_take_profit: bool,
    pub hit_stop_loss: bool,
    pub outcome_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReasonPerformanceDto {
    pub reason_type: String,
    pub reason_key: String,
    pub model_version: String,
    pub sample_size: i64,
    pub win_rate: Option<f64>,
    pub avg_actual_return: Option<f64>,
    pub avg_net_gp: Option<i64>,
    pub calibration_error: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModelCardRefDto {
    pub model_id: String,
    pub model_version: String,
    pub artifact_hash: Option<String>,
    pub feature_schema_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GraphPathDto {
    pub source_item_id: i64,
    pub target_item_id: i64,
    pub relation_type: String,
    pub edge_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub contribution_weight: Option<f64>,
    pub explanation: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GraphSourceDto {
    pub relation_type: String,
    pub source_item_id: i64,
    pub target_item_id: i64,
    pub contribution_weight: Option<f64>,
}

impl RecommendationEvidenceDto {
    pub fn from_record(
        record: RecommendationEvidenceRecord,
        item: Option<grand_edge_domain::Item>,
        reason_performance: Vec<ReasonOutcomeSummary>,
    ) -> Self {
        let recommendation = RecommendationDto::from_parts(record.recommendation.clone(), item);
        let feature_snapshot = record
            .linked_predictions
            .first()
            .map(|value| FeatureSnapshotDto::from(value.feature_snapshot.clone()));
        let predictions = record
            .linked_predictions
            .iter()
            .cloned()
            .map(PredictionEvidenceDto::from)
            .collect::<Vec<_>>();
        let prediction_links = record
            .linked_predictions
            .iter()
            .map(|value| PredictionLinkDto {
                prediction_id: value.prediction.prediction_id.0,
                contribution_weight: value.contribution_weight,
                model_id: value.prediction.model_id.0.clone(),
                model_version: value.prediction.model_version.0.clone(),
            })
            .collect::<Vec<_>>();
        let graph_links = record
            .graph
            .as_ref()
            .map(|graph| graph.graph_links.clone())
            .unwrap_or_default();
        let graph_paths = graph_links
            .iter()
            .cloned()
            .map(GraphPathDto::from)
            .collect();
        let graph_sources = graph_links
            .iter()
            .cloned()
            .map(GraphSourceDto::from)
            .collect();
        let explanation = StructuredExplanationDto {
            summary: record
                .recommendation
                .explanation
                .structured_explanation
                .summary
                .clone(),
            reason_atoms: record
                .recommendation
                .explanation
                .structured_explanation
                .reason_atoms
                .iter()
                .cloned()
                .map(ReasonAtomDto::from)
                .collect(),
            invalidation_rules: record
                .recommendation
                .explanation
                .structured_explanation
                .invalidation_rules
                .iter()
                .cloned()
                .map(crate::recommendations::view::InvalidationRuleDto::from)
                .collect(),
            graph_version: record
                .recommendation
                .explanation
                .structured_explanation
                .graph_version
                .clone()
                .or_else(|| record.recommendation.explanation.graph_version.clone()),
            graph_reason_path_count: record
                .recommendation
                .explanation
                .structured_explanation
                .graph_reason_path_count,
        };
        let outcome = record
            .outcome
            .clone()
            .map(|value| RecommendationOutcomeDto::from(value));
        let model_cards = collect_model_cards(&record.linked_predictions);
        let data_state = derive_data_state(&record, &reason_performance, recommendation.data_state);
        let stages = build_stages(&record, &reason_performance);

        Self {
            recommendation_id: record.recommendation.recommendation_id.0,
            item_id: record.recommendation.item_id.0,
            as_of: record.recommendation.as_of,
            stages,
            feature_snapshot,
            predictions,
            prediction_links,
            recommendation,
            graph_version: record
                .graph
                .as_ref()
                .map(|value| value.graph_version.clone()),
            graph_paths,
            graph_sources,
            explanation,
            outcome,
            reason_performance: reason_performance
                .into_iter()
                .map(ReasonPerformanceDto::from)
                .collect(),
            model_cards,
            data_state,
        }
    }
}

impl From<FeatureSnapshot> for FeatureSnapshotDto {
    fn from(value: FeatureSnapshot) -> Self {
        Self {
            feature_snapshot_id: value.feature_snapshot_id,
            item_id: value.item_id.0,
            as_of: value.as_of,
            feature_set_version: value.feature_set_version,
            graph_version: value.graph_version,
            source_window_start: value.source_window_start,
            source_window_end: value.source_window_end,
            features: value.features.into_iter().collect(),
        }
    }
}

impl From<LinkedPredictionRecord> for PredictionEvidenceDto {
    fn from(value: LinkedPredictionRecord) -> Self {
        Self::from(value.prediction)
    }
}

impl From<Prediction> for PredictionEvidenceDto {
    fn from(value: Prediction) -> Self {
        Self {
            prediction_id: value.prediction_id.0,
            feature_snapshot_id: value.feature_snapshot_id,
            item_id: value.item_id.0,
            as_of: value.as_of,
            horizon_secs: value.horizon_secs.0,
            model_id: value.model_id.0,
            model_version: value.model_version.0,
            predicted_direction: serde_json::to_value(value.predicted_direction)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_else(|| "unknown".to_string()),
            predicted_return: value.predicted_return.map(|rate| rate.get()),
            confidence: value.confidence.get(),
            prediction_interval_low: value
                .prediction_interval
                .as_ref()
                .and_then(|interval| interval.low.map(|rate| rate.get())),
            prediction_interval_high: value
                .prediction_interval
                .as_ref()
                .and_then(|interval| interval.high.map(|rate| rate.get())),
            explanation: value.explanation,
        }
    }
}

impl From<ReasonAtom> for ReasonAtomDto {
    fn from(value: ReasonAtom) -> Self {
        Self {
            reason_type: serde_json::to_value(value.reason_type)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_else(|| "unknown".to_string()),
            reason_key: value.reason_key,
            label: value.label,
            direction: match value.direction {
                ReasonDirection::Positive => "positive",
                ReasonDirection::Negative => "negative",
                ReasonDirection::Neutral => "neutral",
            }
            .to_string(),
            weight: value.weight,
            evidence: value.evidence,
        }
    }
}

impl From<grand_edge_domain::RecommendationOutcome> for RecommendationOutcomeDto {
    fn from(value: grand_edge_domain::RecommendationOutcome) -> Self {
        Self {
            evaluated_at: value.evaluated_at,
            horizon_secs: value.horizon_secs.0,
            actual_return: value.actual_return.map(|rate| rate.get()),
            actual_net_gp: value.actual_net_gp.map(|gp| gp.0),
            direction_correct: value.direction_correct,
            hit_take_profit: value.hit_take_profit,
            hit_stop_loss: value.hit_stop_loss,
            outcome_label: serde_json::to_value(value.outcome_label)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_else(|| "unevaluable".to_string()),
        }
    }
}

impl From<ReasonOutcomeSummary> for ReasonPerformanceDto {
    fn from(value: ReasonOutcomeSummary) -> Self {
        Self {
            reason_type: serde_json::to_value(value.reason_type)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_else(|| "unknown".to_string()),
            reason_key: value.reason_key,
            model_version: value.model_version.0,
            sample_size: value.sample_size,
            win_rate: value.win_rate.map(|value| value.get()),
            avg_actual_return: value.avg_actual_return.map(|value| value.get()),
            avg_net_gp: value.avg_net_gp.map(|value| value.0),
            calibration_error: value.calibration_error,
        }
    }
}

impl From<RecommendationGraphLinkSummary> for GraphPathDto {
    fn from(value: RecommendationGraphLinkSummary) -> Self {
        Self {
            source_item_id: value.source_item_id,
            target_item_id: value.target_item_id,
            relation_type: value.relation_type,
            edge_id: value.edge_id,
            event_id: value.event_id,
            contribution_weight: value.weight,
            explanation: value.explanation,
        }
    }
}

impl From<RecommendationGraphLinkSummary> for GraphSourceDto {
    fn from(value: RecommendationGraphLinkSummary) -> Self {
        Self {
            relation_type: value.relation_type,
            source_item_id: value.source_item_id,
            target_item_id: value.target_item_id,
            contribution_weight: value.weight,
        }
    }
}

fn collect_model_cards(linked_predictions: &[LinkedPredictionRecord]) -> Vec<ModelCardRefDto> {
    let mut seen = BTreeSet::new();
    let mut model_cards = Vec::new();
    for linked in linked_predictions {
        let model_id = linked.prediction.model_id.0.clone();
        let model_version = linked.prediction.model_version.0.clone();
        if !seen.insert((model_id.clone(), model_version.clone())) {
            continue;
        }
        model_cards.push(ModelCardRefDto {
            artifact_hash: json_string(
                &linked.prediction.explanation,
                &["artifact_hash", "artifactHash"],
            ),
            feature_schema_hash: json_string(
                &linked.prediction.explanation,
                &["feature_schema_hash", "featureSchemaHash"],
            ),
            model_id,
            model_version,
        });
    }
    model_cards
}

fn json_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(key).and_then(serde_json::Value::as_str))
        .map(str::to_string)
}

fn derive_data_state(
    record: &RecommendationEvidenceRecord,
    reason_performance: &[ReasonOutcomeSummary],
    recommendation_state: crate::market::status_view::DataStateDto,
) -> EvidenceDataStateDto {
    if matches!(
        recommendation_state,
        crate::market::status_view::DataStateDto::Stale
    ) {
        return EvidenceDataStateDto {
            status: EvidenceDataStateStatusDto::Stale,
            reason: Some("Recommendation evidence is based on stale market data.".to_string()),
        };
    }

    if record.outcome.is_none() && outcome_is_pending(record) {
        return EvidenceDataStateDto {
            status: EvidenceDataStateStatusDto::Pending,
            reason: Some("Outcome horizon has not elapsed yet.".to_string()),
        };
    }

    let missing_predictions = record.linked_predictions.is_empty();
    let missing_reason_performance = !record
        .recommendation
        .explanation
        .structured_explanation
        .reason_atoms
        .is_empty()
        && reason_performance.is_empty();
    let missing_explanation = record
        .recommendation
        .explanation
        .structured_explanation
        .reason_atoms
        .is_empty()
        && record.recommendation.reasons.is_empty();

    if missing_predictions || missing_reason_performance || missing_explanation {
        return EvidenceDataStateDto {
            status: EvidenceDataStateStatusDto::Degraded,
            reason: Some("Some evidence layers are missing from this recommendation.".to_string()),
        };
    }

    EvidenceDataStateDto {
        status: EvidenceDataStateStatusDto::Live,
        reason: None,
    }
}

fn build_stages(
    record: &RecommendationEvidenceRecord,
    reason_performance: &[ReasonOutcomeSummary],
) -> Vec<EvidenceStageDto> {
    let first_prediction = record
        .linked_predictions
        .first()
        .map(|value| &value.prediction);
    vec![
        EvidenceStageDto {
            kind: EvidenceStageKindDto::MarketData,
            label: "Market data".to_string(),
            timestamp: Some(record.recommendation.as_of),
            status: EvidenceStageStatusDto::Present,
        },
        EvidenceStageDto {
            kind: EvidenceStageKindDto::FeatureSnapshot,
            label: "Feature snapshot".to_string(),
            timestamp: record
                .linked_predictions
                .first()
                .map(|value| value.feature_snapshot.created_at),
            status: if record.linked_predictions.is_empty() {
                EvidenceStageStatusDto::Degraded
            } else {
                EvidenceStageStatusDto::Present
            },
        },
        EvidenceStageDto {
            kind: EvidenceStageKindDto::GraphContext,
            label: "Graph context".to_string(),
            timestamp: Some(record.recommendation.as_of),
            status: if record.graph.is_some() {
                EvidenceStageStatusDto::Present
            } else {
                EvidenceStageStatusDto::Missing
            },
        },
        EvidenceStageDto {
            kind: EvidenceStageKindDto::Prediction,
            label: "Prediction".to_string(),
            timestamp: first_prediction.map(|value| value.created_at),
            status: if record.linked_predictions.is_empty() {
                EvidenceStageStatusDto::Missing
            } else {
                EvidenceStageStatusDto::Present
            },
        },
        EvidenceStageDto {
            kind: EvidenceStageKindDto::Recommendation,
            label: "Recommendation".to_string(),
            timestamp: Some(record.recommendation.as_of),
            status: EvidenceStageStatusDto::Present,
        },
        EvidenceStageDto {
            kind: EvidenceStageKindDto::Explanation,
            label: "Explanation".to_string(),
            timestamp: Some(record.recommendation.as_of),
            status: if record
                .recommendation
                .explanation
                .structured_explanation
                .reason_atoms
                .is_empty()
            {
                EvidenceStageStatusDto::Degraded
            } else {
                EvidenceStageStatusDto::Present
            },
        },
        EvidenceStageDto {
            kind: EvidenceStageKindDto::OutcomeEvaluation,
            label: "Outcome evaluation".to_string(),
            timestamp: record.outcome.as_ref().map(|value| value.evaluated_at),
            status: if record.outcome.is_some() {
                EvidenceStageStatusDto::Present
            } else if outcome_is_pending(record) {
                EvidenceStageStatusDto::Pending
            } else if reason_performance.is_empty() {
                EvidenceStageStatusDto::Missing
            } else {
                EvidenceStageStatusDto::Degraded
            },
        },
    ]
}

fn outcome_is_pending(record: &RecommendationEvidenceRecord) -> bool {
    let now = Utc::now();
    let horizon_secs = record
        .linked_predictions
        .iter()
        .map(|value| value.prediction.horizon_secs.0)
        .max()
        .or_else(|| {
            record
                .recommendation
                .explanation
                .strategy_votes
                .iter()
                .map(|value| value.horizon_secs.0)
                .max()
        })
        .unwrap_or_default();
    record.recommendation.as_of + Duration::seconds(horizon_secs) > now
}
