use chrono::{TimeZone, Utc};
use grand_edge_api::routes::recommendations::RecommendationDto;
use grand_edge_domain::{
    Gp, ItemId, ModelVersion, Probability, Rate, Recommendation, RecommendationAction,
    RecommendationExplanation, RecommendationId, StrategyId, StrategySignal,
    StructuredRecommendationExplanation, UserId,
};
use insta::assert_json_snapshot;

#[test]
fn api_recommendations_snapshot_is_stable() {
    let recommendation = Recommendation {
        recommendation_id: RecommendationId(uuid::Uuid::nil()),
        user_id: Some(UserId(uuid::Uuid::nil())),
        item_id: ItemId(4151),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        action: RecommendationAction::Buy,
        score: Rate::new(0.9).unwrap(),
        prediction_confidence: Some(Probability::new(0.8).unwrap()),
        execution_confidence: Some(Probability::new(0.7).unwrap()),
        recommendation_confidence: Probability::new(0.75).unwrap(),
        expected_net_gp: Some(Gp(1_400)),
        expected_roi: Some(Rate::new(0.03).unwrap()),
        risk_label: Some("low".to_string()),
        reasons: vec!["Tax-adjusted edge clears threshold".to_string()],
        explanation: RecommendationExplanation {
            feature_set_version: "features_v1".to_string(),
            market_rules_version: "osrs_rules_v1_review_required".to_string(),
            graph_version: None,
            graph_context: None,
            strategy_votes: vec![StrategySignal {
                item_id: ItemId(4151),
                strategy_id: StrategyId("spread_edge_v1".to_string()),
                model_version: ModelVersion("v1".to_string()),
                as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
                side: grand_edge_domain::SignalSide::Buy,
                horizon_secs: grand_edge_domain::HorizonSecs(3600),
                confidence: Probability::new(0.8).unwrap(),
                expected_return: Rate::new(0.03).unwrap(),
                expected_net_gp_per_unit: Gp(1_400),
                target_entry: Some(Gp(100_000)),
                target_exit: Some(Gp(104_000)),
                stop_loss: Some(Gp(99_000)),
                take_profit: Some(Gp(104_000)),
                max_quantity: Some(grand_edge_domain::Quantity(8)),
                execution_estimate: None,
                explanation: serde_json::json!({"reason": "fixture"}),
            }],
            score_components: vec![grand_edge_domain::ScoreComponent {
                key: "edge".to_string(),
                label: "Edge".to_string(),
                value: Rate::new(0.7).unwrap(),
                weight: Some(Rate::new(0.5).unwrap()),
            }],
            accuracy_snapshot: None,
            structured_explanation: StructuredRecommendationExplanation::default(),
        },
    };

    assert_json_snapshot!(
        "api_recommendation_dto",
        RecommendationDto::from(recommendation)
    );
}
