use chrono::{Duration, TimeZone, Utc};
use grand_edge_domain::{
    FeatureSnapshot, Gp, HorizonSecs, IntervalPrice, Item, ItemId, MarketRules, OutcomeLabel,
    Prediction, PredictionDirection, PredictionId, Probability, Rate, Recommendation,
    RecommendationAction, RecommendationExplanation, RecommendationId,
    RecommendationPredictionLink, ScoreComponent, SignalSide, StrategyId, StrategySignal,
    StructuredRecommendationExplanation, UserId,
};
use grand_edge_metrics::{
    OutcomeEvaluationConfig, OutcomeEvaluationJob, evaluate_due_recommendations,
    evaluate_recommendation_outcome,
};
use grand_edge_storage::Storage;
use uuid::Uuid;

#[test]
fn evaluate_buy_recommendation_marks_win_after_net_profit() {
    let recommendation = sample_recommendation(RecommendationAction::Buy, SignalSide::Buy);
    let outcome = evaluate_recommendation_outcome(
        &recommendation,
        &[
            interval(1, 100_500, 100_000),
            interval(2, 104_500, 103_500),
            interval(3, 104_000, 103_000),
        ],
        &MarketRules::default(),
        fixed_time() + Duration::hours(3),
    )
    .unwrap();

    assert_eq!(outcome.outcome_label, OutcomeLabel::Win);
    assert_eq!(outcome.actual_net_gp, Some(Gp(2_840)));
    assert_eq!(outcome.direction_correct, Some(true));
    assert!(outcome.hit_take_profit);
    assert!(!outcome.hit_stop_loss);
}

#[test]
fn evaluate_cashout_recommendation_marks_direction_correct() {
    let recommendation = sample_recommendation(RecommendationAction::Cashout, SignalSide::Cashout);
    let outcome = evaluate_recommendation_outcome(
        &recommendation,
        &[
            interval(1, 101_000, 100_500),
            interval(2, 99_000, 98_500),
            interval(3, 98_500, 98_000),
        ],
        &MarketRules::default(),
        fixed_time() + Duration::hours(3),
    )
    .unwrap();

    assert_eq!(outcome.outcome_label, OutcomeLabel::Win);
    assert_eq!(outcome.direction_correct, Some(true));
    assert!(outcome.actual_return.unwrap().get() < 0.0);
}

#[test]
fn missing_future_history_returns_unevaluable() {
    let recommendation = sample_recommendation(RecommendationAction::Buy, SignalSide::Buy);
    let outcome = evaluate_recommendation_outcome(
        &recommendation,
        &[interval(1, 100_500, 100_000)],
        &MarketRules::default(),
        fixed_time() + Duration::hours(3),
    )
    .unwrap();

    assert_eq!(outcome.outcome_label, OutcomeLabel::Unevaluable);
    assert!(outcome.actual_return.is_none());
}

#[test]
fn history_at_as_of_is_not_used_for_outcome() {
    let recommendation = sample_recommendation(RecommendationAction::Buy, SignalSide::Buy);
    let outcome = evaluate_recommendation_outcome(
        &recommendation,
        &[
            interval(0, 200_000, 50_000),
            interval(1, 100_500, 100_000),
            interval(2, 104_500, 104_000),
        ],
        &MarketRules::default(),
        fixed_time() + Duration::hours(3),
    )
    .unwrap();

    assert_eq!(outcome.outcome_label, OutcomeLabel::Win);
    assert!(outcome.actual_return.unwrap().get() > 0.0);
}

#[test]
fn watch_recommendation_scores_forecast_not_trade_pnl() {
    let recommendation = sample_recommendation(RecommendationAction::Watch, SignalSide::Watch);
    let outcome = evaluate_recommendation_outcome(
        &recommendation,
        &[
            interval(1, 100_500, 100_000),
            interval(2, 100_600, 100_100),
            interval(3, 100_700, 100_200),
        ],
        &MarketRules::default(),
        fixed_time() + Duration::hours(3),
    )
    .unwrap();

    assert_eq!(outcome.outcome_label, OutcomeLabel::Expired);
    assert!(outcome.actual_net_gp.is_none());
    assert_eq!(outcome.direction_correct, None);
}

#[test]
fn avoid_recommendation_succeeds_when_skipped_trade_loses() {
    let recommendation = sample_recommendation(RecommendationAction::Avoid, SignalSide::Avoid);
    let outcome = evaluate_recommendation_outcome(
        &recommendation,
        &[
            interval(1, 100_500, 100_000),
            interval(2, 99_500, 98_900),
            interval(3, 99_200, 98_700),
        ],
        &MarketRules::default(),
        fixed_time() + Duration::hours(3),
    )
    .unwrap();

    assert_eq!(outcome.outcome_label, OutcomeLabel::Win);
    assert_eq!(outcome.direction_correct, Some(true));
    assert!(outcome.hit_stop_loss);
}

#[test]
fn sell_recommendation_uses_position_profit_after_tax() {
    let recommendation = sample_recommendation(RecommendationAction::Hold, SignalSide::Hold);
    let outcome = evaluate_recommendation_outcome(
        &recommendation,
        &[
            interval(1, 100_500, 100_000),
            interval(2, 103_500, 103_000),
            interval(3, 104_000, 103_500),
        ],
        &MarketRules::default(),
        fixed_time() + Duration::hours(3),
    )
    .unwrap();

    assert_eq!(outcome.outcome_label, OutcomeLabel::Win);
    assert!(outcome.actual_net_gp.unwrap().as_i64() > 0);
}

#[test]
fn loss_and_break_even_labels_are_covered() {
    let buy = sample_recommendation(RecommendationAction::Buy, SignalSide::Buy);
    let loss = evaluate_recommendation_outcome(
        &buy,
        &[
            interval(1, 100_500, 100_000),
            interval(2, 99_400, 98_900),
            interval(3, 99_000, 98_800),
        ],
        &MarketRules::default(),
        fixed_time() + Duration::hours(3),
    )
    .unwrap();
    let break_even = evaluate_recommendation_outcome(
        &buy,
        &[
            interval(1, 100_500, 100_000),
            interval(2, 102_551, 102_551),
            interval(3, 102_551, 102_551),
        ],
        &MarketRules::default(),
        fixed_time() + Duration::hours(3),
    )
    .unwrap();

    assert_eq!(loss.outcome_label, OutcomeLabel::Loss);
    assert_eq!(break_even.outcome_label, OutcomeLabel::BreakEven);
}

#[tokio::test]
#[ignore]
async fn outcome_job_is_idempotent() {
    let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
        return;
    };
    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    seed_item(&storage).await;

    let snapshot = sample_feature_snapshot();
    storage
        .evidence()
        .insert_feature_snapshot(&snapshot)
        .await
        .unwrap();
    let prediction = sample_prediction(snapshot.feature_snapshot_id);
    storage
        .predictions()
        .insert_predictions(std::slice::from_ref(&prediction))
        .await
        .unwrap();
    let recommendation = sample_recommendation(RecommendationAction::Buy, SignalSide::Buy);
    storage
        .recommendations()
        .insert_recommendation_with_links(
            &recommendation,
            &[RecommendationPredictionLink::new(
                recommendation.recommendation_id,
                prediction.prediction_id,
                1.0,
            )
            .unwrap()],
        )
        .await
        .unwrap();
    storage
        .prices()
        .upsert_interval_prices(&[
            interval(1, 100_500, 100_000),
            interval(2, 104_500, 103_500),
            interval(3, 104_000, 103_000),
        ])
        .await
        .unwrap();

    let job = OutcomeEvaluationJob {
        as_of: fixed_time() + Duration::hours(3),
        horizon_secs: HorizonSecs(7_200),
        config: OutcomeEvaluationConfig::default(),
    };
    let first = evaluate_due_recommendations(&storage, job.clone())
        .await
        .unwrap();
    let second = evaluate_due_recommendations(&storage, job).await.unwrap();

    assert_eq!(first.inserted, 1);
    assert_eq!(second.inserted, 0);
    assert_eq!(
        storage
            .outcomes()
            .get_recommendation_outcome(recommendation.recommendation_id)
            .await
            .unwrap()
            .unwrap()
            .outcome_label,
        OutcomeLabel::Win
    );
}

async fn seed_item(storage: &Storage) {
    storage
        .items()
        .upsert_items(&[Item {
            item_id: ItemId(4151),
            name: "Abyssal whip".to_string(),
            examine: Some("A weapon from the abyss.".to_string()),
            members: true,
            buy_limit: Some(70),
            low_alch: None,
            high_alch: None,
            value: Some(Gp(120_001)),
            icon: None,
            updated_at: fixed_time(),
        }])
        .await
        .unwrap();
}

fn sample_feature_snapshot() -> FeatureSnapshot {
    FeatureSnapshot {
        feature_snapshot_id: Uuid::new_v4(),
        item_id: ItemId(4151),
        as_of: fixed_time(),
        feature_set_version: "features_v1".to_string(),
        graph_version: None,
        source_window_start: fixed_time() - Duration::hours(6),
        source_window_end: fixed_time(),
        features: serde_json::Map::new(),
        created_at: fixed_time(),
    }
}

fn sample_prediction(feature_snapshot_id: Uuid) -> Prediction {
    Prediction {
        prediction_id: PredictionId(Uuid::new_v4()),
        feature_snapshot_id,
        item_id: ItemId(4151),
        as_of: fixed_time(),
        horizon_secs: HorizonSecs(7_200),
        model_id: StrategyId::new("spread_edge").unwrap(),
        model_version: grand_edge_domain::ModelVersion::new("2026-06-16.1").unwrap(),
        predicted_direction: PredictionDirection::Up,
        predicted_return: Some(Rate::new(0.04).unwrap()),
        confidence: Probability::new(0.7).unwrap(),
        prediction_interval: None,
        explanation: serde_json::json!({}),
        created_at: fixed_time(),
    }
}

fn sample_recommendation(action: RecommendationAction, side: SignalSide) -> Recommendation {
    Recommendation {
        recommendation_id: RecommendationId(Uuid::new_v4()),
        user_id: Some(UserId(Uuid::new_v4())),
        item_id: ItemId(4151),
        as_of: fixed_time(),
        action,
        score: Rate::new(0.72).unwrap(),
        prediction_confidence: Some(Probability::new(0.67).unwrap()),
        execution_confidence: Some(Probability::new(0.61).unwrap()),
        recommendation_confidence: Probability::new(0.64).unwrap(),
        expected_net_gp: Some(Gp(1_000)),
        expected_roi: Some(Rate::new(0.05).unwrap()),
        risk_label: Some("moderate".to_string()),
        reasons: vec!["fixture".to_string()],
        explanation: RecommendationExplanation {
            feature_set_version: "features_v1".to_string(),
            market_rules_version: MarketRules::default().version,
            strategy_votes: vec![StrategySignal {
                item_id: ItemId(4151),
                strategy_id: StrategyId::new("spread_edge").unwrap(),
                model_version: grand_edge_domain::ModelVersion::new("2026-06-16.1").unwrap(),
                as_of: fixed_time(),
                side,
                horizon_secs: HorizonSecs(7_200),
                confidence: Probability::new(0.7).unwrap(),
                expected_return: Rate::new(0.04).unwrap(),
                expected_net_gp_per_unit: Gp(1_000),
                target_entry: Some(Gp(100_000)),
                target_exit: Some(Gp(103_000)),
                stop_loss: Some(Gp(99_000)),
                take_profit: Some(Gp(104_000)),
                max_quantity: Some(grand_edge_domain::Quantity(2)),
                execution_estimate: None,
                explanation: serde_json::json!({}),
            }],
            score_components: vec![ScoreComponent {
                key: "edge".to_string(),
                label: "Edge".to_string(),
                value: Rate::new(0.8).unwrap(),
                weight: Some(Rate::new(0.5).unwrap()),
            }],
            accuracy_snapshot: None,
            structured_explanation: StructuredRecommendationExplanation::default(),
        },
    }
}

fn interval(hours_after_as_of: i64, high: i64, low: i64) -> IntervalPrice {
    IntervalPrice {
        item_id: ItemId(4151),
        bucket_start: fixed_time() + Duration::hours(hours_after_as_of),
        interval: grand_edge_domain::PriceInterval::OneHour,
        avg_high_price: Some(Gp(high)),
        high_price_volume: 100,
        avg_low_price: Some(Gp(low)),
        low_price_volume: 100,
    }
}

fn fixed_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 16, 10, 0, 0).unwrap()
}
