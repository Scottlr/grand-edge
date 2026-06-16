use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use grand_edge_domain::{
    Probability, Rate, Recommendation, RecommendationAction, RecommendationExplanation,
    RecommendationId, StrategySignal,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{errors::ApiError, state::AppState};

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

#[derive(Debug, Deserialize, IntoParams)]
pub struct RecommendationQuery {
    pub user_id: Option<Uuid>,
    pub action: Option<RecommendationActionDto>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategySignalDto {
    pub item_id: i64,
    pub strategy_id: String,
    pub model_version: String,
    pub as_of: DateTime<Utc>,
    pub side: String,
    pub horizon_secs: i64,
    pub confidence: f64,
    pub expected_return: f64,
    pub expected_net_gp_per_unit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScoreComponentDto {
    pub key: String,
    pub label: String,
    pub value: f64,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationExplanationDto {
    pub feature_set_version: String,
    pub market_rules_version: String,
    pub strategy_votes: Vec<StrategySignalDto>,
    pub score_components: Vec<ScoreComponentDto>,
    pub accuracy_snapshot: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationDto {
    pub recommendation_id: Uuid,
    pub user_id: Option<Uuid>,
    pub item_id: i64,
    pub as_of: DateTime<Utc>,
    pub action: RecommendationActionDto,
    pub score: f64,
    pub prediction_confidence: Option<f64>,
    pub execution_confidence: Option<f64>,
    pub recommendation_confidence: f64,
    pub expected_net_gp: Option<i64>,
    pub expected_roi: Option<f64>,
    pub risk_label: Option<String>,
    pub reasons: Vec<String>,
    pub explanation: RecommendationExplanationDto,
}

fn default_limit() -> i64 {
    50
}

#[utoipa::path(
    get,
    path = "/api/recommendations",
    params(RecommendationQuery),
    responses((status = 200, body = [RecommendationDto]))
)]
pub async fn list_recommendations(
    State(state): State<AppState>,
    Query(query): Query<RecommendationQuery>,
) -> Result<Json<Vec<RecommendationDto>>, ApiError> {
    let recommendations = state
        .services
        .list_recommendations(
            query.user_id.map(grand_edge_domain::UserId),
            query.action.map(RecommendationAction::from),
            query.limit,
            query.offset,
        )
        .await?;

    Ok(Json(
        recommendations
            .into_iter()
            .map(RecommendationDto::from)
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/recommendations/{id}/explanation",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = RecommendationExplanationDto), (status = 404))
)]
pub async fn get_recommendation_explanation(
    State(state): State<AppState>,
    Path(recommendation_id): Path<Uuid>,
) -> Result<Json<RecommendationExplanationDto>, ApiError> {
    let explanation = state
        .services
        .get_recommendation_explanation(RecommendationId(recommendation_id))
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "recommendation {} was not found",
                recommendation_id
            ))
        })?;

    Ok(Json(RecommendationExplanationDto::from(explanation)))
}

impl From<Recommendation> for RecommendationDto {
    fn from(value: Recommendation) -> Self {
        Self {
            recommendation_id: value.recommendation_id.0,
            user_id: value.user_id.map(|user_id| user_id.0),
            item_id: value.item_id.0,
            as_of: value.as_of,
            action: value.action.into(),
            score: value.score.get(),
            prediction_confidence: value.prediction_confidence.map(Probability::get),
            execution_confidence: value.execution_confidence.map(Probability::get),
            recommendation_confidence: value.recommendation_confidence.get(),
            expected_net_gp: value.expected_net_gp.map(|gp| gp.0),
            expected_roi: value.expected_roi.map(Rate::get),
            risk_label: value.risk_label,
            reasons: value.reasons,
            explanation: RecommendationExplanationDto::from(value.explanation),
        }
    }
}

impl From<RecommendationExplanation> for RecommendationExplanationDto {
    fn from(value: RecommendationExplanation) -> Self {
        Self {
            feature_set_version: value.feature_set_version,
            market_rules_version: value.market_rules_version,
            strategy_votes: value
                .strategy_votes
                .into_iter()
                .map(StrategySignalDto::from)
                .collect(),
            score_components: value
                .score_components
                .into_iter()
                .map(|component| ScoreComponentDto {
                    key: component.key,
                    label: component.label,
                    value: component.value.get(),
                    weight: component.weight.map(Rate::get),
                })
                .collect(),
            accuracy_snapshot: value
                .accuracy_snapshot
                .map(|snapshot| serde_json::to_value(snapshot).unwrap_or(serde_json::Value::Null)),
        }
    }
}

impl From<StrategySignal> for StrategySignalDto {
    fn from(value: StrategySignal) -> Self {
        Self {
            item_id: value.item_id.0,
            strategy_id: value.strategy_id.0,
            model_version: value.model_version.0,
            as_of: value.as_of,
            side: serde_json::to_value(value.side)
                .ok()
                .and_then(|value| value.as_str().map(ToString::to_string))
                .unwrap_or_else(|| "hold".to_string()),
            horizon_secs: value.horizon_secs.0,
            confidence: value.confidence.get(),
            expected_return: value.expected_return.get(),
            expected_net_gp_per_unit: value.expected_net_gp_per_unit.0,
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
