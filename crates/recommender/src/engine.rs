use std::collections::HashMap;

use chrono::{DateTime, Utc};
use grand_edge_domain::{
    FeatureVector, ItemId, LatestPrice, Probability, Rate, Recommendation, RecommendationAction,
    RecommendationId, StrategySignal, UserId, UserPosition,
};
use grand_edge_metrics::{MetricWindow, MetricsEngine};
use grand_edge_simulator::SimulationEngine;
use grand_edge_storage::{Storage, StoredPrediction};
use uuid::Uuid;

use crate::{
    RecommendationConfig, RecommendationError, RecommendationScore,
    actions::map_action,
    explanations::{build_explanation, build_reasons},
    quantity::size_quantity,
    scoring::score_candidate,
};

#[derive(Debug, Clone)]
pub struct RecommendationInput {
    pub user_id: Option<UserId>,
    pub as_of: DateTime<Utc>,
    pub latest: LatestPrice,
    pub feature_vector: FeatureVector,
    pub primary_signal: StrategySignal,
    pub strategy_votes: Vec<StrategySignal>,
    pub accuracy_snapshot: Option<grand_edge_domain::ModelAccuracySnapshot>,
    pub existing_position: Option<UserPosition>,
}

pub struct RecommendationEngine {
    storage: Storage,
    metrics: MetricsEngine,
    simulator: SimulationEngine,
    config: RecommendationConfig,
}

impl RecommendationEngine {
    pub fn new(
        storage: Storage,
        metrics: MetricsEngine,
        simulator: SimulationEngine,
        config: RecommendationConfig,
    ) -> Self {
        Self {
            storage,
            metrics,
            simulator,
            config,
        }
    }

    pub async fn recommend_all(
        &self,
        user_id: Option<Uuid>,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<Recommendation>, RecommendationError> {
        let feature_vectors = self
            .storage
            .features()
            .latest_features(&self.config.feature_set_version)
            .await?;
        let features_by_item = feature_vectors
            .into_iter()
            .map(|feature_vector| (feature_vector.item_id, feature_vector))
            .collect::<HashMap<_, _>>();
        let latest_prices = self.storage.prices().latest_snapshot().await?;
        let latest_by_item = latest_prices
            .into_iter()
            .map(|latest| (latest.item_id, latest))
            .collect::<HashMap<_, _>>();
        let stored_predictions = self
            .storage
            .strategies()
            .list_latest_predictions(as_of)
            .await?;
        let grouped_predictions = group_predictions(stored_predictions);
        let user_id = user_id.map(UserId);
        let positions_by_item = if let Some(user_id) = user_id {
            self.storage
                .positions()
                .active_positions_for_user(user_id)
                .await?
                .into_iter()
                .map(|position| (position.item_id, position))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        let mut recommendations = Vec::new();
        for (item_id, predictions) in grouped_predictions {
            let recommendation = self
                .recommend_for_item_group(
                    user_id,
                    as_of,
                    item_id,
                    predictions,
                    &features_by_item,
                    &latest_by_item,
                    &positions_by_item,
                )
                .await?;
            recommendations.push(recommendation);
        }

        self.storage
            .recommendations()
            .insert_recommendations(&recommendations)
            .await?;
        Ok(recommendations)
    }

    pub async fn recommend_item(
        &self,
        user_id: Option<Uuid>,
        item_id: i64,
        as_of: DateTime<Utc>,
    ) -> Result<Option<Recommendation>, RecommendationError> {
        let item_id = ItemId(item_id);
        let predictions = self
            .storage
            .strategies()
            .list_latest_predictions_for_item(item_id, as_of)
            .await?;
        if predictions.is_empty() {
            return Ok(None);
        }

        let feature_vector = self
            .storage
            .features()
            .latest_features(&self.config.feature_set_version)
            .await?
            .into_iter()
            .find(|vector| vector.item_id == item_id)
            .ok_or(RecommendationError::MissingFeatures(item_id.0))?;
        let latest = self
            .storage
            .prices()
            .latest_snapshot()
            .await?
            .into_iter()
            .find(|price| price.item_id == item_id)
            .ok_or(RecommendationError::MissingLatestPrice(item_id.0))?;
        let existing_position = if let Some(user_id) = user_id.map(UserId) {
            self.storage
                .positions()
                .active_position_for_user_item(user_id, item_id)
                .await?
        } else {
            None
        };

        self.recommend_from_parts(
            user_id.map(UserId),
            as_of,
            latest,
            feature_vector,
            predictions,
            existing_position,
        )
        .await
        .map(Some)
    }

    pub fn build_recommendation(
        &self,
        input: RecommendationInput,
    ) -> Result<Recommendation, RecommendationError> {
        let score = score_candidate(
            &input.primary_signal,
            &input.feature_vector,
            input.accuracy_snapshot.as_ref(),
            &self.config,
        );
        let action = map_action(
            &input.primary_signal,
            &input.latest,
            &score,
            input.existing_position.as_ref(),
            &self.config.market_rules,
            &self.config,
        );
        let recommendation_confidence = Probability::new(score.recommendation_confidence)
            .map_err(|_| RecommendationError::InvalidConfidence(score.recommendation_confidence))?;
        let score_rate = Rate::new(score.final_score)
            .map_err(|_| RecommendationError::InvalidRate(score.final_score))?;
        let sized_quantity = size_quantity(&input.primary_signal);
        let expected_net_gp = sized_quantity.map(|quantity| {
            grand_edge_domain::Gp(
                quantity.as_i64() * input.primary_signal.expected_net_gp_per_unit.as_i64(),
            )
        });
        let reasons = build_reasons(
            action,
            &input.primary_signal,
            &input.latest,
            &score,
            input.existing_position.as_ref(),
            &self.config.market_rules,
        );
        let explanation = build_explanation(
            &input.feature_vector,
            &self.config.market_rules,
            input.strategy_votes,
            &score.components,
            input.accuracy_snapshot,
        );
        let risk_label = risk_label(&score);
        let expected_roi = Some(input.primary_signal.expected_return);
        let _simulation_path = match action {
            RecommendationAction::Buy
            | RecommendationAction::Add
            | RecommendationAction::Cashout => Some("eligible_via_simulator"),
            _ => Some("skipped_without_trade_action"),
        };

        Ok(Recommendation {
            recommendation_id: RecommendationId(Uuid::new_v4()),
            user_id: input.user_id,
            item_id: input.primary_signal.item_id,
            as_of: input.as_of,
            action,
            score: score_rate,
            prediction_confidence: score
                .prediction_confidence
                .map(Probability::new)
                .transpose()
                .map_err(|_| {
                    RecommendationError::InvalidConfidence(
                        score.prediction_confidence.unwrap_or_default(),
                    )
                })?,
            execution_confidence: score
                .execution_confidence
                .map(Probability::new)
                .transpose()
                .map_err(|_| {
                    RecommendationError::InvalidConfidence(
                        score.execution_confidence.unwrap_or_default(),
                    )
                })?,
            recommendation_confidence,
            expected_net_gp,
            expected_roi,
            risk_label,
            reasons,
            explanation,
        })
    }

    async fn recommend_for_item_group(
        &self,
        user_id: Option<UserId>,
        as_of: DateTime<Utc>,
        item_id: ItemId,
        predictions: Vec<StoredPrediction>,
        features_by_item: &HashMap<ItemId, FeatureVector>,
        latest_by_item: &HashMap<ItemId, LatestPrice>,
        positions_by_item: &HashMap<ItemId, UserPosition>,
    ) -> Result<Recommendation, RecommendationError> {
        let feature_vector = features_by_item
            .get(&item_id)
            .cloned()
            .ok_or(RecommendationError::MissingFeatures(item_id.0))?;
        let latest = latest_by_item
            .get(&item_id)
            .cloned()
            .ok_or(RecommendationError::MissingLatestPrice(item_id.0))?;
        let existing_position = positions_by_item.get(&item_id).cloned();
        self.recommend_from_parts(
            user_id,
            as_of,
            latest,
            feature_vector,
            predictions,
            existing_position,
        )
        .await
    }

    async fn recommend_from_parts(
        &self,
        user_id: Option<UserId>,
        as_of: DateTime<Utc>,
        latest: LatestPrice,
        feature_vector: FeatureVector,
        predictions: Vec<StoredPrediction>,
        existing_position: Option<UserPosition>,
    ) -> Result<Recommendation, RecommendationError> {
        let mut votes = Vec::new();
        let mut best_signal = None;
        let mut best_accuracy = None;
        let mut best_score = f64::NEG_INFINITY;

        for prediction in predictions {
            let signal = to_strategy_signal(prediction)?;
            let accuracy = self
                .metrics
                .latest_accuracy_snapshot(
                    &signal.strategy_id.0,
                    &signal.model_version.0,
                    MetricWindow::SevenDays,
                )
                .await?;
            let score = score_candidate(&signal, &feature_vector, accuracy.as_ref(), &self.config);
            if score.final_score > best_score {
                best_score = score.final_score;
                best_accuracy = accuracy.clone();
                best_signal = Some(signal.clone());
            }
            votes.push(signal);
        }

        let primary_signal =
            best_signal.ok_or(RecommendationError::MissingFeatures(latest.item_id.0))?;
        let input = RecommendationInput {
            user_id,
            as_of,
            latest,
            feature_vector,
            primary_signal,
            strategy_votes: votes,
            accuracy_snapshot: best_accuracy,
            existing_position,
        };

        let recommendation = self.build_recommendation(input)?;
        let _simulator_reference = &self.simulator;
        Ok(recommendation)
    }
}

fn group_predictions(predictions: Vec<StoredPrediction>) -> HashMap<ItemId, Vec<StoredPrediction>> {
    let mut grouped = HashMap::new();
    for prediction in predictions {
        grouped
            .entry(prediction.item_id)
            .or_insert_with(Vec::new)
            .push(prediction);
    }
    grouped
}

fn risk_label(score: &RecommendationScore) -> Option<String> {
    let combined_penalty = score.risk_penalty + score.liquidity_penalty;
    Some(if combined_penalty >= 0.20 {
        "high".to_string()
    } else if combined_penalty >= 0.10 {
        "medium".to_string()
    } else {
        "low".to_string()
    })
}

fn to_strategy_signal(prediction: StoredPrediction) -> Result<StrategySignal, RecommendationError> {
    Ok(StrategySignal {
        item_id: prediction.item_id,
        strategy_id: prediction.strategy_id,
        model_version: prediction.model_version,
        as_of: prediction.as_of,
        side: prediction.side,
        horizon_secs: grand_edge_domain::HorizonSecs::try_from(prediction.horizon_secs)?,
        confidence: prediction.confidence,
        expected_return: prediction.expected_return,
        expected_net_gp_per_unit: prediction.expected_net_gp_per_unit,
        target_entry: prediction.target_entry,
        target_exit: prediction.target_exit,
        stop_loss: prediction.stop_loss,
        take_profit: prediction.take_profit,
        max_quantity: prediction.max_quantity,
        execution_estimate: None,
        explanation: prediction.explanation,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        FeatureVector, Gp, HorizonSecs, ItemId, LatestPrice, ModelAccuracySnapshot, ModelVersion,
        PositionId, Probability, Quantity, Rate, RecommendationAction, StrategyId, StrategySignal,
        UserId, UserPosition,
    };
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    use crate::{RecommendationConfig, RecommendationEngine};

    fn engine() -> RecommendationEngine {
        let storage = grand_edge_storage::Storage::new(
            PgPoolOptions::new()
                .connect_lazy("postgres://grandedge:grandedge@localhost/grandedge")
                .unwrap(),
        );
        let metrics = grand_edge_metrics::MetricsEngine::new(storage.clone());
        let simulator = grand_edge_simulator::SimulationEngine::new(
            storage.clone(),
            grand_edge_simulator::SimulatorConfig::default(),
        );
        RecommendationEngine::new(storage, metrics, simulator, RecommendationConfig::default())
    }

    #[tokio::test]
    async fn buy_requires_positive_tax_adjusted_edge() {
        let recommendation = engine()
            .build_recommendation(input(
                base_signal(0.20, 1_200, Some(0.9), grand_edge_domain::SignalSide::Buy),
                None,
                Some(accuracy()),
            ))
            .unwrap();
        assert_eq!(recommendation.action, RecommendationAction::Buy);
        assert!(
            recommendation
                .expected_net_gp
                .is_some_and(|value| value.as_i64() > 0)
        );
        assert_eq!(recommendation.explanation.strategy_votes.len(), 1);
        assert!(recommendation.explanation.accuracy_snapshot.is_some());
    }

    #[tokio::test]
    async fn cashout_uses_existing_position_profit() {
        let mut signal = base_signal(-0.01, 100, Some(0.3), grand_edge_domain::SignalSide::Avoid);
        signal.target_exit = Some(Gp(101_000));
        let mut input = input(signal, Some(position()), Some(accuracy()));
        input.latest.low = Some(Gp(105_000));
        input.latest.high = Some(Gp(106_000));
        let recommendation = engine().build_recommendation(input).unwrap();
        assert_eq!(recommendation.action, RecommendationAction::Cashout);
        assert!(
            recommendation
                .reasons
                .iter()
                .any(|reason| reason.contains("after-tax profit"))
        );
    }

    #[tokio::test]
    async fn hold_when_edge_positive_but_exit_not_reached() {
        let recommendation = engine()
            .build_recommendation(input(
                base_signal(0.01, 500, Some(0.3), grand_edge_domain::SignalSide::Buy),
                Some(position()),
                Some(accuracy()),
            ))
            .unwrap();
        assert_eq!(recommendation.action, RecommendationAction::Hold);
    }

    #[tokio::test]
    async fn avoid_explains_liquidity_blocker() {
        let recommendation = engine()
            .build_recommendation(input(
                base_signal(0.02, -100, Some(0.2), grand_edge_domain::SignalSide::Buy),
                None,
                None,
            ))
            .unwrap();
        assert_eq!(recommendation.action, RecommendationAction::Avoid);
        assert!(
            recommendation
                .reasons
                .iter()
                .any(|reason| reason.contains("Liquidity") || reason.contains("Tax-adjusted"))
        );
    }

    #[tokio::test]
    async fn watch_when_prediction_positive_but_execution_confidence_weak() {
        let recommendation = engine()
            .build_recommendation(input(
                base_signal(0.03, 900, Some(0.2), grand_edge_domain::SignalSide::Buy),
                None,
                Some(accuracy()),
            ))
            .unwrap();
        assert_eq!(recommendation.action, RecommendationAction::Watch);
        assert!(
            recommendation
                .reasons
                .iter()
                .any(|reason| reason.contains("execution quality is uncertain"))
        );
    }

    fn input(
        signal: StrategySignal,
        existing_position: Option<UserPosition>,
        accuracy_snapshot: Option<ModelAccuracySnapshot>,
    ) -> crate::RecommendationInput {
        crate::RecommendationInput {
            user_id: Some(UserId(Uuid::new_v4())),
            as_of: signal.as_of,
            latest: LatestPrice {
                item_id: signal.item_id,
                high: Some(Gp(103_000)),
                high_time: Some(signal.as_of),
                low: Some(Gp(101_000)),
                low_time: Some(signal.as_of),
                observed_at: signal.as_of,
            },
            feature_vector: feature_vector(signal.item_id),
            primary_signal: signal.clone(),
            strategy_votes: vec![signal],
            accuracy_snapshot,
            existing_position,
        }
    }

    fn base_signal(
        expected_return: f64,
        expected_net_gp_per_unit: i64,
        execution_confidence: Option<f64>,
        side: grand_edge_domain::SignalSide,
    ) -> StrategySignal {
        StrategySignal {
            item_id: ItemId(4151),
            strategy_id: StrategyId("spread_edge_v1".to_string()),
            model_version: ModelVersion("v1".to_string()),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            side,
            horizon_secs: HorizonSecs(3_600),
            confidence: Probability::new(0.8).unwrap(),
            expected_return: Rate::new(expected_return).unwrap(),
            expected_net_gp_per_unit: Gp(expected_net_gp_per_unit),
            target_entry: Some(Gp(100_000)),
            target_exit: Some(Gp(104_000)),
            stop_loss: Some(Gp(99_000)),
            take_profit: Some(Gp(104_000)),
            max_quantity: Some(Quantity(10)),
            execution_estimate: Some(grand_edge_domain::ExecutionEstimate {
                observed_liquidity: grand_edge_domain::ObservedLiquidityProxy {
                    observed_volume: Quantity(400),
                    observed_high_side_volume: Quantity(220),
                    observed_low_side_volume: Quantity(180),
                    observed_volume_z: Some(Rate::new(1.0).unwrap()),
                    observed_volume_reliability: Some(Probability::new(0.7).unwrap()),
                    high_low_volume_ratio: Some(Rate::new(1.22).unwrap()),
                    note: "proxy".to_string(),
                },
                estimated_fill_probability: execution_confidence
                    .and_then(|value| Probability::new(value).ok()),
                liquidity_confidence: execution_confidence
                    .and_then(|value| Probability::new(value).ok()),
                estimated_capacity: Some(Quantity(8)),
                participation_rate: Some(Probability::new(0.05).unwrap()),
                confidence_haircut: Some(Probability::new(0.5).unwrap()),
                spread_pct: Some(Rate::new(0.02).unwrap()),
                price_staleness_seconds: Some(HorizonSecs(60)),
                volatility: Some(Rate::new(0.03).unwrap()),
            }),
            explanation: serde_json::json!({ "reason": "fixture" }),
        }
    }

    fn feature_vector(item_id: ItemId) -> FeatureVector {
        let mut values = serde_json::Map::new();
        values.insert("ewma_volatility_24h".to_string(), serde_json::json!(0.03));
        values.insert("spread_pct".to_string(), serde_json::json!(0.02));
        values.insert("price_staleness_secs".to_string(), serde_json::json!(60.0));
        FeatureVector {
            item_id,
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values,
        }
    }

    fn accuracy() -> ModelAccuracySnapshot {
        ModelAccuracySnapshot {
            strategy_id: StrategyId("spread_edge_v1".to_string()),
            model_version: ModelVersion("v1".to_string()),
            lookback_window: "seven_days".to_string(),
            sample_size: 20,
            directional_accuracy: Some(Rate::new(0.7).unwrap()),
            brier_score: Some(Rate::new(0.2).unwrap()),
            avg_realized_roi: Some(Rate::new(0.03).unwrap()),
            max_drawdown: Some(Rate::new(0.1).unwrap()),
            calibration: serde_json::json!({}),
        }
    }

    fn position() -> UserPosition {
        UserPosition {
            position_id: PositionId(Uuid::new_v4()),
            user_id: UserId(Uuid::new_v4()),
            item_id: ItemId(4151),
            quantity: Quantity(5),
            avg_buy_price: Gp(100_000),
            bought_at: Some(Utc.with_ymd_and_hms(2026, 6, 15, 12, 0, 0).unwrap()),
            notes: None,
        }
    }
}
