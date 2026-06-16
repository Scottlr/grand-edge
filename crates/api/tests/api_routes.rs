use std::sync::Arc;

use async_trait::async_trait;
use axum::{Router, body::Body, http::Request};
use chrono::{TimeZone, Utc};
use grand_edge_api::{
    app::build_router,
    routes::{items::ItemDto, live::LiveEvent, recommendations::RecommendationActionDto},
    state::{
        ApiServices, AppState, LiveEventBus, PositionUpsert, SimulationRunDraft,
        StrategyStatusRecord,
    },
};
use grand_edge_domain::{
    Gp, Item, ItemIcon, ItemId, MarketRules, ModelAccuracySnapshot, ModelVersion, PositionId,
    PriceInterval, Probability, Quantity, Rate, Recommendation, RecommendationAction,
    RecommendationExplanation, RecommendationId, SignalSide, StrategyId, StrategySignal,
    StructuredRecommendationExplanation, UserId, UserPosition, WikiImageSource,
};
use grand_edge_storage::StoredSimulationRun;
use http_body_util::BodyExt;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Default)]
struct TestServices {
    items: Vec<Item>,
    history: Vec<grand_edge_domain::IntervalPrice>,
    recommendations: Vec<Recommendation>,
    strategies: Vec<StrategyStatusRecord>,
    positions: Vec<UserPosition>,
    runs: Vec<StoredSimulationRun>,
}

#[async_trait]
impl ApiServices for TestServices {
    async fn list_items(
        &self,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<Item>, grand_edge_api::errors::ApiError> {
        Ok(self.items.clone())
    }

    async fn get_item(
        &self,
        item_id: ItemId,
    ) -> Result<Option<Item>, grand_edge_api::errors::ApiError> {
        Ok(self
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .cloned())
    }

    async fn item_history(
        &self,
        _item_id: ItemId,
        _interval: PriceInterval,
        _limit: i64,
        _before: Option<chrono::DateTime<Utc>>,
    ) -> Result<Vec<grand_edge_domain::IntervalPrice>, grand_edge_api::errors::ApiError> {
        Ok(self.history.clone())
    }

    async fn list_recommendations(
        &self,
        _user_id: Option<UserId>,
        _action: Option<RecommendationAction>,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<Recommendation>, grand_edge_api::errors::ApiError> {
        Ok(self.recommendations.clone())
    }

    async fn get_recommendation_explanation(
        &self,
        recommendation_id: RecommendationId,
    ) -> Result<Option<RecommendationExplanation>, grand_edge_api::errors::ApiError> {
        Ok(self
            .recommendations
            .iter()
            .find(|recommendation| recommendation.recommendation_id == recommendation_id)
            .map(|recommendation| recommendation.explanation.clone()))
    }

    async fn list_strategies(
        &self,
    ) -> Result<Vec<StrategyStatusRecord>, grand_edge_api::errors::ApiError> {
        Ok(self.strategies.clone())
    }

    async fn patch_strategy(
        &self,
        strategy_id: &str,
        enabled: bool,
    ) -> Result<Option<StrategyStatusRecord>, grand_edge_api::errors::ApiError> {
        Ok(self
            .strategies
            .iter()
            .find(|strategy| strategy.strategy_id == strategy_id)
            .map(|strategy| StrategyStatusRecord {
                strategy_id: strategy.strategy_id.clone(),
                enabled,
            }))
    }

    async fn list_simulation_runs(
        &self,
        _limit: i64,
        _offset: i64,
    ) -> Result<Vec<StoredSimulationRun>, grand_edge_api::errors::ApiError> {
        Ok(self.runs.clone())
    }

    async fn create_simulation_run(
        &self,
        draft: SimulationRunDraft,
    ) -> Result<StoredSimulationRun, grand_edge_api::errors::ApiError> {
        Ok(StoredSimulationRun {
            run_id: grand_edge_domain::RunId(Uuid::new_v4()),
            name: draft.name,
            strategy_config: draft.strategy_config,
            started_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            finished_at: None,
            status: "created".to_string(),
        })
    }

    async fn list_positions(&self) -> Result<Vec<UserPosition>, grand_edge_api::errors::ApiError> {
        Ok(self.positions.clone())
    }

    async fn create_position(
        &self,
        input: PositionUpsert,
    ) -> Result<UserPosition, grand_edge_api::errors::ApiError> {
        Ok(UserPosition {
            position_id: PositionId(Uuid::new_v4()),
            user_id: UserId(Uuid::new_v4()),
            item_id: ItemId(input.item_id),
            quantity: Quantity(input.quantity),
            avg_buy_price: Gp(input.avg_buy_price),
            bought_at: input.bought_at,
            notes: input.notes,
        })
    }

    async fn update_position(
        &self,
        position_id: PositionId,
        input: PositionUpsert,
    ) -> Result<Option<UserPosition>, grand_edge_api::errors::ApiError> {
        Ok(Some(UserPosition {
            position_id,
            user_id: UserId(Uuid::new_v4()),
            item_id: ItemId(input.item_id),
            quantity: Quantity(input.quantity),
            avg_buy_price: Gp(input.avg_buy_price),
            bought_at: input.bought_at,
            notes: input.notes,
        }))
    }
}

fn router(services: TestServices, live_events: LiveEventBus) -> Router {
    build_router(AppState::new(Arc::new(services), live_events), None)
}

fn recommendation_fixture() -> Recommendation {
    Recommendation {
        recommendation_id: RecommendationId(Uuid::new_v4()),
        user_id: Some(UserId(Uuid::new_v4())),
        item_id: ItemId(4151),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        action: RecommendationAction::Buy,
        score: Rate::new(0.9).unwrap(),
        prediction_confidence: Some(Probability::new(0.8).unwrap()),
        execution_confidence: Some(Probability::new(0.7).unwrap()),
        recommendation_confidence: Probability::new(0.75).unwrap(),
        expected_net_gp: Some(Gp(1400)),
        expected_roi: Some(Rate::new(0.03).unwrap()),
        risk_label: Some("low".to_string()),
        reasons: vec!["Tax-adjusted edge clears threshold".to_string()],
        explanation: RecommendationExplanation {
            feature_set_version: "features_v1".to_string(),
            market_rules_version: MarketRules::default().version,
            graph_version: None,
            graph_context: None,
            strategy_votes: vec![StrategySignal {
                item_id: ItemId(4151),
                strategy_id: StrategyId("spread_edge_v1".to_string()),
                model_version: ModelVersion("v1".to_string()),
                as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
                side: SignalSide::Buy,
                horizon_secs: grand_edge_domain::HorizonSecs(3600),
                confidence: Probability::new(0.8).unwrap(),
                expected_return: Rate::new(0.03).unwrap(),
                expected_net_gp_per_unit: Gp(1400),
                target_entry: Some(Gp(100_000)),
                target_exit: Some(Gp(104_000)),
                stop_loss: Some(Gp(99_000)),
                take_profit: Some(Gp(104_000)),
                max_quantity: Some(Quantity(8)),
                execution_estimate: None,
                explanation: serde_json::json!({"reason": "fixture"}),
            }],
            score_components: vec![grand_edge_domain::ScoreComponent {
                key: "edge".to_string(),
                label: "Edge".to_string(),
                value: Rate::new(0.7).unwrap(),
                weight: Some(Rate::new(0.5).unwrap()),
            }],
            accuracy_snapshot: Some(ModelAccuracySnapshot {
                strategy_id: StrategyId("spread_edge_v1".to_string()),
                model_version: ModelVersion("v1".to_string()),
                lookback_window: "seven_days".to_string(),
                sample_size: 12,
                directional_accuracy: Some(Rate::new(0.66).unwrap()),
                brier_score: Some(Rate::new(0.18).unwrap()),
                avg_realized_roi: Some(Rate::new(0.02).unwrap()),
                max_drawdown: Some(Rate::new(0.1).unwrap()),
                calibration: serde_json::json!({}),
            }),
            structured_explanation: StructuredRecommendationExplanation::default(),
        },
    }
}

fn item_fixture(file_name: &str, cdn_url: &str) -> Item {
    Item {
        item_id: ItemId(1949),
        name: "Chef's hat".to_string(),
        examine: Some("A tall hat for chefs.".to_string()),
        members: false,
        buy_limit: Some(100),
        low_alch: Some(Gp(1)),
        high_alch: Some(Gp(2)),
        value: Some(Gp(4)),
        icon: Some(ItemIcon {
            source_file_name: file_name.to_string(),
            canonical_file_name: "Chef's_hat.png".to_string(),
            cdn_url: cdn_url.to_string(),
            source: WikiImageSource::MappingIcon,
        }),
        updated_at: Utc::now(),
    }
}

#[tokio::test]
async fn health_route_returns_ok() {
    let response = router(TestServices::default(), LiveEventBus::default())
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn items_route_returns_sanitized_icon_cdn_url() {
    let response = router(
        TestServices {
            items: vec![item_fixture(
                "Bronze pickaxe.png",
                "https://oldschool.runescape.wiki/images/Bronze_pickaxe.png",
            )],
            ..Default::default()
        },
        LiveEventBus::default(),
    )
    .oneshot(
        Request::builder()
            .uri("/api/items")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let items: Vec<ItemDto> = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        items[0].icon.as_ref().unwrap().cdn_url,
        "https://oldschool.runescape.wiki/images/Bronze_pickaxe.png"
    );
}

#[tokio::test]
async fn items_route_preserves_apostrophe_icon_encoding() {
    let response = router(
        TestServices {
            items: vec![item_fixture(
                "Chef's hat.png",
                "https://oldschool.runescape.wiki/images/Chef%27s_hat.png",
            )],
            ..Default::default()
        },
        LiveEventBus::default(),
    )
    .oneshot(
        Request::builder()
            .uri("/api/items/1949")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let item: ItemDto = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        item.icon.unwrap().cdn_url,
        "https://oldschool.runescape.wiki/images/Chef%27s_hat.png"
    );
}

#[tokio::test]
async fn history_requires_limit() {
    let response = router(TestServices::default(), LiveEventBus::default())
        .oneshot(
            Request::builder()
                .uri("/api/items/1949/history?interval=1h")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn patch_strategy_rejects_unknown_id() {
    let response = router(TestServices::default(), LiveEventBus::default())
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/strategies/unknown")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"enabled":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn recommendations_route_returns_typed_payload() {
    let recommendation = recommendation_fixture();
    let response = router(
        TestServices {
            recommendations: vec![recommendation],
            ..Default::default()
        },
        LiveEventBus::default(),
    )
    .oneshot(
        Request::builder()
            .uri("/api/recommendations?action=buy")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload[0]["action"], "buy");
    assert_eq!(
        payload[0]["explanation"]["strategyVotes"][0]["strategyId"],
        "spread_edge_v1"
    );
}

#[tokio::test]
async fn live_stream_serializes_event() {
    let bus = LiveEventBus::default();
    let app = router(TestServices::default(), bus.clone());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/live/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    bus.publish(LiveEvent::RecommendationUpdated {
        recommendation_id: Uuid::nil(),
        item_id: 4151,
        action: RecommendationActionDto::Buy,
    });

    let mut body = response.into_body();
    let frame = tokio::time::timeout(std::time::Duration::from_secs(1), body.frame())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let bytes = frame.into_data().unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(text.contains("recommendation_updated"));
    assert!(text.contains("\"item_id\":4151"));
}
