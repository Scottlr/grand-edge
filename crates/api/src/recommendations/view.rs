use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use grand_edge_domain::{
    ExecutionEstimate, InvalidationRule, Item, Probability, Rate, Recommendation,
    RecommendationAction, RecommendationExplanation, SignalSide, StrategySignal,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    market::status_view::{DataStateDto, MarketStatusDto},
    model_accuracy::summary_view::ModelAccuracySummaryDto,
    routes::items::ItemIconDto,
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationActionDto {
    Buy,
    Add,
    Hold,
    Cashout,
    Avoid,
    Watch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionConfidenceDto {
    pub observed_volume: i64,
    pub observed_volume_z: Option<f64>,
    pub estimated_fill_probability: Option<f64>,
    pub estimated_capacity: Option<i64>,
    pub liquidity_confidence: Option<f64>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyVoteDto {
    pub item_id: i64,
    pub strategy_id: String,
    pub model_version: String,
    pub as_of: DateTime<Utc>,
    pub side: String,
    pub horizon_secs: i64,
    pub confidence: f64,
    pub expected_return: f64,
    pub expected_net_gp_per_unit: i64,
    pub target_entry: Option<i64>,
    pub target_exit: Option<i64>,
    pub stop_loss: Option<i64>,
    pub take_profit: Option<i64>,
    pub max_quantity: Option<i64>,
    pub execution: Option<ExecutionConfidenceDto>,
    pub explanation: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScoreComponentDto {
    pub key: String,
    pub label: String,
    pub value: f64,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvalidationRuleDto {
    pub metric: String,
    pub operator: String,
    pub threshold: String,
    pub current_value: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfidenceBreakdownDto {
    pub confidence: f64,
    pub prediction_confidence: Option<f64>,
    pub execution_confidence: Option<f64>,
    pub recommendation_confidence: f64,
    pub model_agreement_label: String,
    pub recent_accuracy: Option<f64>,
    pub data_quality_label: String,
    pub execution_quality_label: Option<String>,
    pub regime_label: Option<String>,
    pub penalties: Vec<ScoreComponentDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationDto {
    pub recommendation_id: Uuid,
    pub user_id: Option<Uuid>,
    pub item_id: i64,
    pub item_name: String,
    pub item_icon: Option<ItemIconDto>,
    pub as_of: DateTime<Utc>,
    pub action: RecommendationActionDto,
    pub score: f64,
    pub confidence: f64,
    pub prediction_confidence: Option<f64>,
    pub execution_confidence: Option<f64>,
    pub recommendation_confidence: f64,
    pub execution: Option<ExecutionConfidenceDto>,
    pub expected_net_gp: Option<i64>,
    pub expected_roi: Option<f64>,
    pub risk_label: Option<String>,
    pub horizon_seconds: i64,
    pub primary_reason: String,
    pub reasons: Vec<String>,
    pub invalidation_rules: Vec<InvalidationRuleDto>,
    pub model_agreement: f64,
    pub confidence_breakdown: ConfidenceBreakdownDto,
    pub strategy_votes: Vec<StrategyVoteDto>,
    pub accuracy: Option<ModelAccuracySummaryDto>,
    pub data_state: DataStateDto,
    pub market_status: MarketStatusDto,
}

impl RecommendationDto {
    pub fn from_parts(value: Recommendation, item: Option<Item>) -> Self {
        let explanation = value.explanation.clone();
        let primary_vote = explanation.strategy_votes.iter().max_by(|left, right| {
            left.confidence
                .get()
                .partial_cmp(&right.confidence.get())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let execution = primary_vote
            .and_then(|vote| vote.execution_estimate.as_ref())
            .map(ExecutionConfidenceDto::from);
        let model_agreement = model_agreement(&explanation.strategy_votes);
        let market_status = market_status(&value, &explanation);
        let penalties = explanation
            .score_components
            .iter()
            .filter(|component| component.value.get().is_sign_negative())
            .cloned()
            .map(ScoreComponentDto::from)
            .collect::<Vec<_>>();

        let accuracy = value
            .explanation
            .accuracy_snapshot
            .clone()
            .map(ModelAccuracySummaryDto::from);
        let item_name = item
            .as_ref()
            .map(|item| item.name.clone())
            .unwrap_or_else(|| format!("Item {}", value.item_id.0));
        let item_icon = item.and_then(|item| item.icon.map(ItemIconDto::from));

        Self {
            recommendation_id: value.recommendation_id.0,
            user_id: value.user_id.map(|user_id| user_id.0),
            item_id: value.item_id.0,
            item_name,
            item_icon,
            as_of: value.as_of,
            action: value.action.into(),
            score: value.score.get(),
            confidence: value.recommendation_confidence.get(),
            prediction_confidence: value.prediction_confidence.map(Probability::get),
            execution_confidence: value.execution_confidence.map(Probability::get),
            recommendation_confidence: value.recommendation_confidence.get(),
            execution,
            expected_net_gp: value.expected_net_gp.map(|gp| gp.0),
            expected_roi: value.expected_roi.map(Rate::get),
            risk_label: value.risk_label,
            horizon_seconds: primary_vote
                .map(|vote| vote.horizon_secs.0)
                .unwrap_or_default(),
            primary_reason: value
                .reasons
                .first()
                .cloned()
                .unwrap_or_else(|| value.explanation.structured_explanation.summary.clone()),
            reasons: value.reasons,
            invalidation_rules: explanation
                .structured_explanation
                .invalidation_rules
                .iter()
                .cloned()
                .map(InvalidationRuleDto::from)
                .collect(),
            model_agreement,
            confidence_breakdown: ConfidenceBreakdownDto {
                confidence: value.recommendation_confidence.get(),
                prediction_confidence: value.prediction_confidence.map(Probability::get),
                execution_confidence: value.execution_confidence.map(Probability::get),
                recommendation_confidence: value.recommendation_confidence.get(),
                model_agreement_label: model_agreement_label(model_agreement),
                recent_accuracy: accuracy
                    .as_ref()
                    .and_then(|snapshot| snapshot.directional_accuracy),
                data_quality_label: data_quality_label(&market_status),
                execution_quality_label: execution_quality_label(primary_vote),
                regime_label: None,
                penalties,
            },
            strategy_votes: explanation
                .strategy_votes
                .into_iter()
                .map(StrategyVoteDto::from)
                .collect(),
            accuracy,
            data_state: market_status.data_state,
            market_status,
        }
    }
}

impl From<ExecutionEstimate> for ExecutionConfidenceDto {
    fn from(value: ExecutionEstimate) -> Self {
        Self {
            observed_volume: value.observed_liquidity.observed_volume.0,
            observed_volume_z: value.observed_liquidity.observed_volume_z.map(Rate::get),
            estimated_fill_probability: value.estimated_fill_probability.map(Probability::get),
            estimated_capacity: value.estimated_capacity.map(|quantity| quantity.0),
            liquidity_confidence: value.liquidity_confidence.map(Probability::get),
            note: value.observed_liquidity.note,
        }
    }
}

impl From<&ExecutionEstimate> for ExecutionConfidenceDto {
    fn from(value: &ExecutionEstimate) -> Self {
        Self {
            observed_volume: value.observed_liquidity.observed_volume.0,
            observed_volume_z: value.observed_liquidity.observed_volume_z.map(Rate::get),
            estimated_fill_probability: value.estimated_fill_probability.map(Probability::get),
            estimated_capacity: value.estimated_capacity.map(|quantity| quantity.0),
            liquidity_confidence: value.liquidity_confidence.map(Probability::get),
            note: value.observed_liquidity.note.clone(),
        }
    }
}

impl From<StrategySignal> for StrategyVoteDto {
    fn from(value: StrategySignal) -> Self {
        Self {
            item_id: value.item_id.0,
            strategy_id: value.strategy_id.0,
            model_version: value.model_version.0,
            as_of: value.as_of,
            side: signal_side_label(value.side),
            horizon_secs: value.horizon_secs.0,
            confidence: value.confidence.get(),
            expected_return: value.expected_return.get(),
            expected_net_gp_per_unit: value.expected_net_gp_per_unit.0,
            target_entry: value.target_entry.map(|gp| gp.0),
            target_exit: value.target_exit.map(|gp| gp.0),
            stop_loss: value.stop_loss.map(|gp| gp.0),
            take_profit: value.take_profit.map(|gp| gp.0),
            max_quantity: value.max_quantity.map(|quantity| quantity.0),
            execution: value.execution_estimate.map(ExecutionConfidenceDto::from),
            explanation: value.explanation,
        }
    }
}

impl From<grand_edge_domain::ScoreComponent> for ScoreComponentDto {
    fn from(value: grand_edge_domain::ScoreComponent) -> Self {
        Self {
            key: value.key,
            label: value.label,
            value: value.value.get(),
            weight: value.weight.map(Rate::get),
        }
    }
}

impl From<InvalidationRule> for InvalidationRuleDto {
    fn from(value: InvalidationRule) -> Self {
        Self {
            metric: value.metric,
            operator: value.operator,
            threshold: value.threshold,
            current_value: value.current_value,
            reason: value.label,
        }
    }
}

impl From<RecommendationActionDto> for RecommendationAction {
    fn from(value: RecommendationActionDto) -> Self {
        match value {
            RecommendationActionDto::Buy => Self::Buy,
            RecommendationActionDto::Add => Self::Add,
            RecommendationActionDto::Hold => Self::Hold,
            RecommendationActionDto::Cashout => Self::Cashout,
            RecommendationActionDto::Avoid => Self::Avoid,
            RecommendationActionDto::Watch => Self::Watch,
        }
    }
}

impl From<RecommendationAction> for RecommendationActionDto {
    fn from(value: RecommendationAction) -> Self {
        match value {
            RecommendationAction::Buy => Self::Buy,
            RecommendationAction::Add => Self::Add,
            RecommendationAction::Hold => Self::Hold,
            RecommendationAction::Cashout => Self::Cashout,
            RecommendationAction::Avoid => Self::Avoid,
            RecommendationAction::Watch => Self::Watch,
        }
    }
}

fn market_status(
    recommendation: &Recommendation,
    explanation: &RecommendationExplanation,
) -> MarketStatusDto {
    let data_quality_atom = explanation
        .structured_explanation
        .reason_atoms
        .iter()
        .find(|atom| atom.reason_key == "data_quality:freshness_completeness");
    let stale = data_quality_atom
        .and_then(|atom| atom.evidence.get("stale"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let missing_inputs = data_quality_atom
        .and_then(|atom| atom.evidence.get("missing_inputs"))
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let degraded_reason = if recommendation.reasons.is_empty()
        || explanation.strategy_votes.is_empty()
        || explanation.structured_explanation.reason_atoms.is_empty()
    {
        Some("Recommendation evidence is incomplete.".to_string())
    } else {
        None
    };

    if stale {
        MarketStatusDto {
            data_state: DataStateDto::Stale,
            stale_reason: Some(
                "Recommendation evidence is based on stale market data.".to_string(),
            ),
            degraded_reason: None,
        }
    } else if let Some(reason) = degraded_reason {
        MarketStatusDto {
            data_state: DataStateDto::Degraded,
            stale_reason: None,
            degraded_reason: Some(reason),
        }
    } else if recommendation.reasons.is_empty() && missing_inputs.is_empty() {
        MarketStatusDto {
            data_state: DataStateDto::Empty,
            stale_reason: None,
            degraded_reason: Some("Recommendation explanation is empty.".to_string()),
        }
    } else {
        MarketStatusDto {
            data_state: DataStateDto::Live,
            stale_reason: None,
            degraded_reason: None,
        }
    }
}

fn data_quality_label(status: &MarketStatusDto) -> String {
    match status.data_state {
        DataStateDto::Stale => "stale".to_string(),
        DataStateDto::Degraded => "degraded".to_string(),
        DataStateDto::Empty => "empty".to_string(),
        DataStateDto::Error => "error".to_string(),
        DataStateDto::Loading => "loading".to_string(),
        DataStateDto::Live => "live".to_string(),
    }
}

fn execution_quality_label(primary_vote: Option<&StrategySignal>) -> Option<String> {
    let estimate = primary_vote.and_then(|vote| vote.execution_estimate.as_ref())?;
    let fill_probability = estimate
        .estimated_fill_probability
        .map(Probability::get)
        .unwrap_or_default();

    Some(if fill_probability >= 0.7 {
        "strong".to_string()
    } else if fill_probability >= 0.45 {
        "estimated".to_string()
    } else {
        "uncertain".to_string()
    })
}

fn signal_side_label(value: SignalSide) -> String {
    match value {
        SignalSide::Buy => "buy",
        SignalSide::Sell => "sell",
        SignalSide::Hold => "hold",
        SignalSide::Avoid => "avoid",
        SignalSide::Cashout => "cashout",
        SignalSide::Watch => "watch",
    }
    .to_string()
}

fn model_agreement(votes: &[StrategySignal]) -> f64 {
    if votes.len() <= 1 {
        return if votes.is_empty() { 0.0 } else { 1.0 };
    }

    let mut counts = BTreeMap::<String, usize>::new();
    for vote in votes {
        *counts.entry(signal_side_label(vote.side)).or_default() += 1;
    }

    let max = counts.values().copied().max().unwrap_or_default();
    (max as f64) / (votes.len() as f64)
}

fn model_agreement_label(agreement: f64) -> String {
    if agreement >= 0.85 {
        "high agreement".to_string()
    } else if agreement >= 0.6 {
        "mixed agreement".to_string()
    } else {
        "divergent".to_string()
    }
}
