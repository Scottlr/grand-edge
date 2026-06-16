use chrono::{Duration, TimeZone, Utc};
use grand_edge_domain::{
    ConfidenceBreakdown, FeatureSnapshot, Gp, HorizonSecs, Item, ItemId, OutcomeLabel, Prediction,
    PredictionDirection, PredictionId, Probability, Rate, ReasonAtom, ReasonDirection, ReasonType,
    Recommendation, RecommendationAction, RecommendationExplanation, RecommendationId,
    RecommendationOutcome, RecommendationPredictionLink, ScoreComponent, SignalSide, StrategyId,
    StrategySignal, StructuredRecommendationExplanation, UserId,
};
use grand_edge_metrics::{ReasonMetricsWindow, refresh_reason_outcomes};
use grand_edge_storage::Storage;
use uuid::Uuid;

#[tokio::test]
#[ignore]
async fn refresh_reason_outcomes_persists_grouped_summaries() {
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
    let recommendation = sample_recommendation();
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
        .outcomes()
        .upsert_recommendation_outcome(&sample_outcome(recommendation.recommendation_id))
        .await
        .unwrap();

    let summaries = refresh_reason_outcomes(
        &storage,
        ReasonMetricsWindow {
            window_start: fixed_time(),
            window_end: fixed_time() + Duration::hours(8),
            min_sample_size: 1,
        },
    )
    .await
    .unwrap();

    assert_eq!(summaries.len(), 2);
    let stored = storage
        .reason_outcomes()
        .list_reason_outcomes(
            ReasonType::ModelSignal,
            "model_signal:spread_edge:21600",
            "2026-06-16.1",
        )
        .await
        .unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].recommendation_action, RecommendationAction::Buy);
    assert_eq!(stored[0].confidence_bucket.as_deref(), Some("0.6-0.7"));
    assert!(stored[0].publishable);
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
        horizon_secs: HorizonSecs(21_600),
        model_id: StrategyId::new("spread_edge").unwrap(),
        model_version: grand_edge_domain::ModelVersion::new("2026-06-16.1").unwrap(),
        predicted_direction: PredictionDirection::Up,
        predicted_return: Some(Rate::new(0.04).unwrap()),
        confidence: Probability::new(0.67).unwrap(),
        prediction_interval: None,
        explanation: serde_json::json!({}),
        created_at: fixed_time(),
    }
}

fn sample_recommendation() -> Recommendation {
    Recommendation {
        recommendation_id: RecommendationId(Uuid::new_v4()),
        user_id: Some(UserId(Uuid::new_v4())),
        item_id: ItemId(4151),
        as_of: fixed_time(),
        action: RecommendationAction::Buy,
        score: Rate::new(0.72).unwrap(),
        prediction_confidence: Some(Probability::new(0.67).unwrap()),
        execution_confidence: Some(Probability::new(0.61).unwrap()),
        recommendation_confidence: Probability::new(0.64).unwrap(),
        expected_net_gp: Some(Gp(125_000)),
        expected_roi: Some(Rate::new(0.05).unwrap()),
        risk_label: Some("moderate".to_string()),
        reasons: vec!["Spread supports entry".to_string()],
        explanation: RecommendationExplanation {
            feature_set_version: "features_v1".to_string(),
            market_rules_version: "osrs_rules_v1_review_required".to_string(),
            strategy_votes: vec![StrategySignal {
                item_id: ItemId(4151),
                strategy_id: StrategyId::new("spread_edge").unwrap(),
                model_version: grand_edge_domain::ModelVersion::new("2026-06-16.1").unwrap(),
                as_of: fixed_time(),
                side: SignalSide::Buy,
                horizon_secs: HorizonSecs(21_600),
                confidence: Probability::new(0.67).unwrap(),
                expected_return: Rate::new(0.04).unwrap(),
                expected_net_gp_per_unit: Gp(1_200),
                target_entry: Some(Gp(2_000_000)),
                target_exit: Some(Gp(2_050_000)),
                stop_loss: Some(Gp(1_980_000)),
                take_profit: Some(Gp(2_070_000)),
                max_quantity: Some(grand_edge_domain::Quantity(8)),
                execution_estimate: None,
                explanation: serde_json::json!({"execution_mode": "conservative_instant"}),
            }],
            score_components: vec![ScoreComponent {
                key: "edge".to_string(),
                label: "Edge".to_string(),
                value: Rate::new(0.8).unwrap(),
                weight: Some(Rate::new(0.5).unwrap()),
            }],
            accuracy_snapshot: None,
            structured_explanation: StructuredRecommendationExplanation {
                summary: "Positive signal with manageable liquidity".to_string(),
                reason_atoms: vec![
                    ReasonAtom {
                        reason_type: ReasonType::ModelSignal,
                        reason_key: "model_signal:spread_edge:21600".to_string(),
                        label: "Model signal".to_string(),
                        direction: ReasonDirection::Positive,
                        weight: 0.7,
                        evidence: serde_json::json!({}),
                    },
                    ReasonAtom {
                        reason_type: ReasonType::LiquidityCheck,
                        reason_key: "liquidity:volume_capacity".to_string(),
                        label: "Liquidity".to_string(),
                        direction: ReasonDirection::Positive,
                        weight: 0.4,
                        evidence: serde_json::json!({}),
                    },
                ],
                invalidation_rules: Vec::new(),
                confidence: ConfidenceBreakdown {
                    prediction_confidence: Probability::new(0.67).unwrap(),
                    recommendation_confidence: Probability::new(0.64).unwrap(),
                    data_quality_confidence: Probability::new(0.9).unwrap(),
                    model_calibration_confidence: Probability::new(0.74).unwrap(),
                    liquidity_confidence: Probability::new(0.7).unwrap(),
                    explanation_confidence: Probability::new(0.68).unwrap(),
                },
                graph_version: None,
                graph_reason_path_count: None,
            },
        },
    }
}

fn sample_outcome(recommendation_id: RecommendationId) -> RecommendationOutcome {
    RecommendationOutcome {
        recommendation_id,
        evaluated_at: fixed_time() + Duration::hours(6),
        horizon_secs: HorizonSecs(21_600),
        actual_return: Some(Rate::new(0.03).unwrap()),
        actual_net_gp: Some(Gp(98_000)),
        direction_correct: Some(true),
        hit_take_profit: false,
        hit_stop_loss: false,
        max_favourable_excursion: Some(Rate::new(0.05).unwrap()),
        max_adverse_excursion: Some(Rate::new(-0.01).unwrap()),
        outcome_label: OutcomeLabel::Win,
    }
}

fn fixed_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap()
}
