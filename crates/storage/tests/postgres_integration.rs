use chrono::Utc;
use grand_edge_domain::{
    Gp, Item, ItemIcon, ItemId, LatestPrice, PositionId, Probability, Quantity, Rate,
    Recommendation, RecommendationAction, RecommendationExplanation, RecommendationId, UserId,
    UserPosition, WikiImageSource,
};
use grand_edge_storage::Storage;
use serde_json::json;
use testcontainers::{GenericImage, ImageExt, core::WaitFor, runners::AsyncRunner};
use uuid::Uuid;

async fn storage_from_container() -> Option<Storage> {
    let image = GenericImage::new("postgres", "16-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_DB", "grand_edge")
        .with_env_var("POSTGRES_USER", "grand_edge")
        .with_env_var("POSTGRES_PASSWORD", "grand_edge");

    let container = match image.start().await {
        Ok(container) => container,
        Err(_) => return None,
    };

    let host_port = match container.get_host_port_ipv4(5432).await {
        Ok(port) => port,
        Err(_) => return None,
    };
    let database_url = format!("postgres://grand_edge:grand_edge@127.0.0.1:{host_port}/grand_edge");
    let storage = Storage::connect(&database_url).await.ok()?;
    storage.migrate().await.ok()?;
    Some(storage)
}

#[tokio::test]
#[ignore]
async fn migrations_and_key_repositories_round_trip_under_testcontainers() {
    let Some(storage) = storage_from_container().await else {
        return;
    };

    let item = Item {
        item_id: ItemId(4151),
        name: "Abyssal whip".to_string(),
        examine: Some("A weapon from the abyss.".to_string()),
        members: true,
        buy_limit: Some(70),
        low_alch: Some(Gp(72_000)),
        high_alch: Some(Gp(108_001)),
        value: Some(Gp(120_001)),
        icon: Some(ItemIcon {
            source_file_name: "Abyssal whip.png".to_string(),
            canonical_file_name: "Abyssal_whip.png".to_string(),
            cdn_url: "https://oldschool.runescape.wiki/images/Abyssal_whip.png".to_string(),
            source: WikiImageSource::MappingIcon,
        }),
        updated_at: Utc::now(),
    };
    storage
        .items()
        .upsert_items(std::slice::from_ref(&item))
        .await
        .unwrap();

    let latest = LatestPrice {
        item_id: item.item_id,
        high: Some(Gp(2_910_000)),
        high_time: Some(Utc::now()),
        low: Some(Gp(2_895_000)),
        low_time: Some(Utc::now()),
        observed_at: Utc::now(),
    };
    storage
        .prices()
        .insert_latest_prices(std::slice::from_ref(&latest))
        .await
        .unwrap();

    let position = UserPosition {
        position_id: PositionId(Uuid::new_v4()),
        user_id: UserId(Uuid::new_v4()),
        item_id: item.item_id,
        quantity: Quantity(5),
        avg_buy_price: Gp(2_800_000),
        bought_at: Some(Utc::now()),
        notes: Some("fixture".to_string()),
    };
    storage
        .positions()
        .upsert_positions(std::slice::from_ref(&position))
        .await
        .unwrap();

    let recommendation = Recommendation {
        recommendation_id: RecommendationId(Uuid::new_v4()),
        user_id: Some(position.user_id),
        item_id: item.item_id,
        as_of: Utc::now(),
        action: RecommendationAction::Buy,
        score: Rate::new(0.9).unwrap(),
        prediction_confidence: Some(Probability::new(0.8).unwrap()),
        execution_confidence: Some(Probability::new(0.7).unwrap()),
        recommendation_confidence: Probability::new(0.75).unwrap(),
        expected_net_gp: Some(Gp(1_400)),
        expected_roi: Some(Rate::new(0.03).unwrap()),
        risk_label: Some("low".to_string()),
        reasons: vec!["fixture".to_string()],
        explanation: RecommendationExplanation {
            feature_set_version: "features_v1".to_string(),
            market_rules_version: "osrs_rules_v1_review_required".to_string(),
            strategy_votes: Vec::new(),
            score_components: Vec::new(),
            accuracy_snapshot: None,
            structured_explanation: grand_edge_domain::StructuredRecommendationExplanation::default(
            ),
        },
    };
    storage
        .recommendations()
        .insert_recommendations(std::slice::from_ref(&recommendation))
        .await
        .unwrap();

    storage
        .simulations()
        .insert_simulation_run(
            Uuid::new_v4(),
            "fixture",
            json!({"strategy": "noop"}),
            "created",
        )
        .await
        .unwrap();

    let item_round_trip = storage.items().get_item(item.item_id).await.unwrap();
    let latest_round_trip = storage.prices().latest_snapshot().await.unwrap();
    let positions_round_trip = storage
        .positions()
        .active_positions_for_user(position.user_id)
        .await
        .unwrap();
    let recommendations_round_trip = storage
        .recommendations()
        .list_recent(Some(position.user_id), None, 10, 0)
        .await
        .unwrap();
    let runs_round_trip = storage.simulations().list_runs(10, 0).await.unwrap();

    assert!(item_round_trip.is_some());
    assert!(!latest_round_trip.is_empty());
    assert_eq!(positions_round_trip.len(), 1);
    assert_eq!(recommendations_round_trip.len(), 1);
    assert_eq!(runs_round_trip.len(), 1);
}
