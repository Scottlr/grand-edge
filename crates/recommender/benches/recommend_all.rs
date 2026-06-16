use chrono::{TimeZone, Utc};
use criterion::{Criterion, criterion_group, criterion_main};
use grand_edge_domain::{
    ExecutionEstimate, FeatureVector, Gp, HorizonSecs, ItemId, ModelVersion,
    ObservedLiquidityProxy, Probability, Quantity, Rate, StrategyId, StrategySignal,
};
use grand_edge_recommender::{RecommendationConfig, RecommendationEngine, RecommendationInput};
use sqlx::postgres::PgPoolOptions;

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

fn input() -> RecommendationInput {
    let as_of = Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();
    let signal = StrategySignal {
        item_id: ItemId(4151),
        strategy_id: StrategyId("spread_edge_v1".to_string()),
        model_version: ModelVersion("v1".to_string()),
        as_of,
        side: grand_edge_domain::SignalSide::Buy,
        horizon_secs: HorizonSecs(3_600),
        confidence: Probability::new(0.8).unwrap(),
        expected_return: Rate::new(0.03).unwrap(),
        expected_net_gp_per_unit: Gp(1_400),
        target_entry: Some(Gp(100_000)),
        target_exit: Some(Gp(104_000)),
        stop_loss: Some(Gp(99_000)),
        take_profit: Some(Gp(104_000)),
        max_quantity: Some(Quantity(8)),
        execution_estimate: Some(ExecutionEstimate {
            observed_liquidity: ObservedLiquidityProxy {
                observed_volume: Quantity(400),
                observed_high_side_volume: Quantity(220),
                observed_low_side_volume: Quantity(180),
                observed_volume_z: None,
                observed_volume_reliability: None,
                high_low_volume_ratio: None,
                note: "proxy".to_string(),
            },
            estimated_fill_probability: Some(Probability::new(0.6).unwrap()),
            liquidity_confidence: Some(Probability::new(0.7).unwrap()),
            estimated_capacity: Some(Quantity(8)),
            participation_rate: Some(Probability::new(0.05).unwrap()),
            confidence_haircut: Some(Probability::new(0.5).unwrap()),
            spread_pct: Some(Rate::new(0.02).unwrap()),
            price_staleness_seconds: Some(HorizonSecs(120)),
            volatility: Some(Rate::new(0.03).unwrap()),
        }),
        explanation: serde_json::json!({"reason": "fixture"}),
    };
    let mut values = serde_json::Map::new();
    values.insert("ewma_volatility_24h".to_string(), serde_json::json!(0.03));
    values.insert("spread_pct".to_string(), serde_json::json!(0.02));
    values.insert("price_staleness_secs".to_string(), serde_json::json!(120));

    RecommendationInput {
        user_id: Some(grand_edge_domain::UserId(uuid::Uuid::nil())),
        as_of,
        latest: grand_edge_domain::LatestPrice {
            item_id: ItemId(4151),
            high: Some(Gp(103_000)),
            high_time: Some(as_of),
            low: Some(Gp(101_000)),
            low_time: Some(as_of),
            observed_at: as_of,
        },
        feature_vector: FeatureVector {
            item_id: ItemId(4151),
            as_of,
            feature_set_version: "features_v1".to_string(),
            values,
        },
        primary_signal: signal.clone(),
        strategy_votes: vec![signal],
        accuracy_snapshot: None,
        existing_position: None,
    }
}

fn recommend_all(c: &mut Criterion) {
    let engine = engine();
    let input = input();

    c.bench_function("recommend_all_fixture", |b| {
        b.iter(|| engine.build_recommendation(input.clone()).unwrap())
    });
}

criterion_group!(benches, recommend_all);
criterion_main!(benches);
