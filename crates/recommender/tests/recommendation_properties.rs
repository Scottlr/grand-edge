use chrono::{TimeZone, Utc};
use grand_edge_domain::{
    ExecutionEstimate, FeatureVector, Gp, HorizonSecs, ItemId, ModelVersion,
    ObservedLiquidityProxy, Probability, Quantity, Rate, RecommendationAction, SignalSide,
    StrategyId, StrategySignal,
};
use grand_edge_recommender::{
    RecommendationConfig, RecommendationEngine, RecommendationInput, quantity::size_quantity,
};
use proptest::prelude::*;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

fn engine() -> RecommendationEngine {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let _guard = runtime.enter();
    let storage = grand_edge_storage::Storage::new(
        PgPoolOptions::new().connect_lazy_with(
            PgConnectOptions::new()
                .host("localhost")
                .username("grandedge")
                .password("grandedge")
                .database("grandedge"),
        ),
    );
    drop(_guard);
    std::mem::forget(runtime);
    let metrics = grand_edge_metrics::MetricsEngine::new(storage.clone());
    let simulator = grand_edge_simulator::SimulationEngine::new(
        storage.clone(),
        grand_edge_simulator::SimulatorConfig::default(),
    );
    RecommendationEngine::new(storage, metrics, simulator, RecommendationConfig::default())
}

fn signal(expected_net_gp_per_unit: i64, requested: i64, capacity: i64) -> StrategySignal {
    StrategySignal {
        item_id: ItemId(4151),
        strategy_id: StrategyId("spread_edge_v1".to_string()),
        model_version: ModelVersion("v1".to_string()),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        side: SignalSide::Buy,
        horizon_secs: HorizonSecs(3_600),
        confidence: Probability::new(0.8).unwrap(),
        expected_return: Rate::new(0.03).unwrap(),
        expected_net_gp_per_unit: Gp(expected_net_gp_per_unit),
        target_entry: Some(Gp(100_000)),
        target_exit: Some(Gp(104_000)),
        stop_loss: Some(Gp(99_000)),
        take_profit: Some(Gp(104_000)),
        max_quantity: Some(Quantity(requested)),
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
            estimated_fill_probability: Some(Probability::new(0.7).unwrap()),
            liquidity_confidence: Some(Probability::new(0.7).unwrap()),
            estimated_capacity: Some(Quantity(capacity)),
            participation_rate: None,
            confidence_haircut: None,
            spread_pct: Some(Rate::new(0.02).unwrap()),
            price_staleness_seconds: Some(HorizonSecs(120)),
            volatility: Some(Rate::new(0.03).unwrap()),
        }),
        explanation: serde_json::json!({}),
    }
}

fn feature_vector() -> FeatureVector {
    let mut values = serde_json::Map::new();
    values.insert("ewma_volatility_24h".to_string(), serde_json::json!(0.03));
    values.insert("spread_pct".to_string(), serde_json::json!(0.02));
    values.insert("price_staleness_secs".to_string(), serde_json::json!(120));
    FeatureVector {
        item_id: ItemId(4151),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        feature_set_version: "features_v1".to_string(),
        values,
    }
}

proptest! {
    #[test]
    fn recommended_quantity_never_exceeds_buy_limit(
        requested in 1_i64..10_000_i64,
        capacity in 1_i64..10_000_i64
    ) {
        let sized = size_quantity(&signal(1_400, requested, capacity)).unwrap();
        prop_assert!(sized.as_i64() <= requested);
        prop_assert!(sized.as_i64() <= capacity);
    }

    #[test]
    fn strategy_must_not_buy_when_expected_net_profit_non_positive(expected_net_gp_per_unit in -10_000_i64..=0_i64) {
        let latest = grand_edge_domain::LatestPrice {
            item_id: ItemId(4151),
            high: Some(Gp(103_000)),
            high_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap()),
            low: Some(Gp(101_000)),
            low_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap()),
            observed_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        };
        let input = RecommendationInput {
            user_id: None,
            as_of: latest.observed_at,
            latest,
            feature_vector: feature_vector(),
            primary_signal: signal(expected_net_gp_per_unit, 10, 10),
            strategy_votes: vec![signal(expected_net_gp_per_unit, 10, 10)],
            accuracy_snapshot: None,
            existing_position: None,
            graph_input: None,
        };

        let recommendation = engine().build_recommendation(input).unwrap();
        prop_assert_ne!(recommendation.action, RecommendationAction::Buy);
    }
}
