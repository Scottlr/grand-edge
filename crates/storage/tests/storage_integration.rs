use chrono::{TimeDelta, TimeZone, Utc};
use grand_edge_domain::{
    ConfidenceBreakdown, FeatureSnapshot, Gp, HorizonSecs, InvalidationRule, Item, ItemId,
    OutcomeLabel, Prediction, PredictionDirection, PredictionId, PredictionInterval, Probability,
    Rate, ReasonAtom, ReasonDirection, ReasonOutcomeSummary, ReasonType, Recommendation,
    RecommendationAction, RecommendationExplanation, RecommendationId, RecommendationOutcome,
    RecommendationPredictionLink, ScoreComponent, SignalSide, StrategyId, StrategySignal,
    StructuredRecommendationExplanation, UserId,
};
use grand_edge_storage::Storage;
use uuid::Uuid;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

#[tokio::test]
#[ignore]
async fn insert_prediction_requires_feature_snapshot() {
    let Some(database_url) = database_url() else {
        return;
    };
    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    seed_item(&storage).await;

    let result = storage
        .predictions()
        .insert_predictions(&[sample_prediction(Uuid::new_v4(), Uuid::new_v4())])
        .await;

    assert!(result.is_err());
}

#[tokio::test]
#[ignore]
async fn recommendation_links_require_existing_prediction() {
    let Some(database_url) = database_url() else {
        return;
    };
    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    seed_item(&storage).await;

    let recommendation = sample_recommendation();
    let result = storage
        .recommendations()
        .insert_recommendation_with_links(
            &recommendation,
            &[RecommendationPredictionLink::new(
                recommendation.recommendation_id,
                PredictionId(Uuid::new_v4()),
                0.75,
            )
            .unwrap()],
        )
        .await;

    assert!(result.is_err());
    assert!(
        storage
            .recommendations()
            .get_recommendation(recommendation.recommendation_id)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
#[ignore]
async fn insert_recommendation_with_links_is_atomic() {
    let Some(database_url) = database_url() else {
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

    let valid_prediction = sample_prediction(snapshot.feature_snapshot_id, Uuid::new_v4());
    storage
        .predictions()
        .insert_predictions(std::slice::from_ref(&valid_prediction))
        .await
        .unwrap();

    let recommendation = sample_recommendation();
    let result = storage
        .recommendations()
        .insert_recommendation_with_links(
            &recommendation,
            &[
                RecommendationPredictionLink::new(
                    recommendation.recommendation_id,
                    valid_prediction.prediction_id,
                    0.6,
                )
                .unwrap(),
                RecommendationPredictionLink::new(
                    recommendation.recommendation_id,
                    PredictionId(Uuid::new_v4()),
                    0.4,
                )
                .unwrap(),
            ],
        )
        .await;

    assert!(result.is_err());
    assert!(
        storage
            .recommendations()
            .get_recommendation(recommendation.recommendation_id)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
#[ignore]
async fn evidence_for_recommendation_reconstructs_chain() {
    let Some(database_url) = database_url() else {
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
    let prediction = sample_prediction(snapshot.feature_snapshot_id, Uuid::new_v4());
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
    let outcome = sample_outcome(recommendation.recommendation_id);
    storage
        .outcomes()
        .upsert_recommendation_outcome(&outcome)
        .await
        .unwrap();

    let evidence = storage
        .evidence()
        .evidence_for_recommendation(recommendation.recommendation_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        evidence.recommendation.recommendation_id,
        recommendation.recommendation_id
    );
    assert_eq!(evidence.linked_predictions.len(), 1);
    assert_eq!(
        evidence.linked_predictions[0]
            .feature_snapshot
            .feature_snapshot_id,
        snapshot.feature_snapshot_id
    );
    assert_eq!(evidence.outcome.unwrap().outcome_label, OutcomeLabel::Win);
}

#[tokio::test]
#[ignore]
async fn evidence_for_recommendation_includes_graph_version_when_present() {
    let Some(database_url) = database_url() else {
        return;
    };
    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    seed_item(&storage).await;

    let mut snapshot = sample_feature_snapshot();
    snapshot.graph_version = Some("graph_v2".to_string());
    storage
        .evidence()
        .insert_feature_snapshot(&snapshot)
        .await
        .unwrap();
    let prediction = sample_prediction(snapshot.feature_snapshot_id, Uuid::new_v4());
    storage
        .predictions()
        .insert_predictions(std::slice::from_ref(&prediction))
        .await
        .unwrap();
    let recommendation = sample_structured_recommendation();
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

    let evidence = storage
        .evidence()
        .evidence_for_recommendation(recommendation.recommendation_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(evidence.graph.unwrap().graph_version, "graph_v2");
}

#[tokio::test]
#[ignore]
async fn reason_outcomes_upsert_by_primary_key() {
    let Some(database_url) = database_url() else {
        return;
    };
    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();

    let mut summary = sample_reason_outcome();
    storage
        .reason_outcomes()
        .upsert_reason_outcome(&summary)
        .await
        .unwrap();
    summary.sample_size = 9;
    storage
        .reason_outcomes()
        .upsert_reason_outcome(&summary)
        .await
        .unwrap();

    let rows = storage
        .reason_outcomes()
        .list_reason_outcomes(ReasonType::ModelSignal, "spread_edge", "2026-06-16.1")
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].sample_size, 9);
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
        source_window_start: fixed_time() - TimeDelta::hours(6),
        source_window_end: fixed_time(),
        features: serde_json::Map::from_iter([("spread_bps".to_string(), serde_json::json!(18))]),
        created_at: fixed_time(),
    }
}

fn sample_prediction(feature_snapshot_id: Uuid, prediction_id: Uuid) -> Prediction {
    Prediction {
        prediction_id: PredictionId(prediction_id),
        feature_snapshot_id,
        item_id: ItemId(4151),
        as_of: fixed_time(),
        horizon_secs: HorizonSecs(21_600),
        model_id: StrategyId::new("gbm_ranker_v1").unwrap(),
        model_version: grand_edge_domain::ModelVersion::new("2026-06-16.1").unwrap(),
        predicted_direction: PredictionDirection::Up,
        predicted_return: Some(Rate::new(0.04).unwrap()),
        confidence: Probability::new(0.67).unwrap(),
        prediction_interval: Some(PredictionInterval {
            low: Some(Rate::new(0.01).unwrap()),
            high: Some(Rate::new(0.08).unwrap()),
        }),
        explanation: serde_json::json!({"kind": "fixture"}),
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
            market_rules_version: "rules_v1".to_string(),
            graph_version: None,
            graph_context: None,
            strategy_votes: vec![StrategySignal {
                strategy_id: StrategyId::new("spread_edge").unwrap(),
                model_version: grand_edge_domain::ModelVersion::new("2026-06-16.1").unwrap(),
                item_id: ItemId(4151),
                as_of: fixed_time(),
                horizon_secs: HorizonSecs(21_600),
                side: SignalSide::Buy,
                expected_return: Rate::new(0.04).unwrap(),
                confidence: Probability::new(0.67).unwrap(),
                expected_net_gp_per_unit: Gp(1_200),
                target_entry: Some(Gp(2_000_000)),
                target_exit: Some(Gp(2_050_000)),
                stop_loss: Some(Gp(1_980_000)),
                take_profit: Some(Gp(2_070_000)),
                max_quantity: Some(grand_edge_domain::Quantity(8)),
                execution_estimate: None,
                explanation: serde_json::json!({"vote": "positive"}),
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

fn sample_structured_recommendation() -> Recommendation {
    let base = sample_recommendation();
    let explanation = StructuredRecommendationExplanation {
        summary: "Graph-aware evidence is present".to_string(),
        reason_atoms: vec![ReasonAtom {
            reason_type: ReasonType::ModelSignal,
            reason_key: "graph_signal".to_string(),
            label: "Graph signal".to_string(),
            direction: ReasonDirection::Positive,
            weight: 0.7,
            evidence: serde_json::json!({"path_count": 2}),
        }],
        invalidation_rules: vec![InvalidationRule {
            rule_key: "spread_break".to_string(),
            label: "Spread break".to_string(),
            metric: "spread_bps".to_string(),
            operator: ">".to_string(),
            threshold: "50".to_string(),
            current_value: Some("18".to_string()),
        }],
        confidence: ConfidenceBreakdown {
            prediction_confidence: Probability::new(0.67).unwrap(),
            recommendation_confidence: Probability::new(0.64).unwrap(),
            data_quality_confidence: Probability::new(0.9).unwrap(),
            model_calibration_confidence: Probability::new(0.74).unwrap(),
            liquidity_confidence: Probability::new(0.7).unwrap(),
            explanation_confidence: Probability::new(0.68).unwrap(),
        },
        graph_version: Some("graph_v2".to_string()),
        graph_reason_path_count: Some(2),
        graph_context: None,
    };

    Recommendation {
        explanation: RecommendationExplanation {
            feature_set_version: base.explanation.feature_set_version.clone(),
            market_rules_version: base.explanation.market_rules_version.clone(),
            graph_version: Some("graph_v2".to_string()),
            graph_context: None,
            strategy_votes: base.explanation.strategy_votes.clone(),
            score_components: base.explanation.score_components.clone(),
            accuracy_snapshot: base.explanation.accuracy_snapshot.clone(),
            structured_explanation: explanation,
        },
        ..base
    }
}

fn sample_outcome(recommendation_id: RecommendationId) -> RecommendationOutcome {
    RecommendationOutcome {
        recommendation_id,
        evaluated_at: fixed_time() + TimeDelta::hours(6),
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

fn sample_reason_outcome() -> ReasonOutcomeSummary {
    ReasonOutcomeSummary {
        reason_type: ReasonType::ModelSignal,
        reason_key: "spread_edge".to_string(),
        model_version: grand_edge_domain::ModelVersion::new("2026-06-16.1").unwrap(),
        recommendation_action: RecommendationAction::Buy,
        execution_mode: Some(grand_edge_domain::ExecutionMode::ConservativeInstant),
        confidence_bucket: Some("0.6-0.7".to_string()),
        window_start: fixed_time() - TimeDelta::days(7),
        window_end: fixed_time(),
        sample_size: 4,
        publishable: true,
        win_rate: Some(Probability::new(0.75).unwrap()),
        avg_actual_return: Some(Rate::new(0.023).unwrap()),
        avg_net_gp: Some(Gp(87_000)),
        calibration_error: Some(0.08),
    }
}

fn fixed_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap()
}
