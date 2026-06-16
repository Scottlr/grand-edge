use chrono::{TimeZone, Utc};
use criterion::{Criterion, criterion_group, criterion_main};
use grand_edge_domain::{FeatureVector, Gp, Item, ItemId, LatestPrice};
use grand_edge_strategies::{
    StrategyConfig, StrategyRegistry, builtin::register_baseline_strategies, builtin::test_context,
};

fn strategy_batch(c: &mut Criterion) {
    let mut registry = StrategyRegistry::new();
    register_baseline_strategies(&mut registry).unwrap();
    let item = Item {
        item_id: ItemId(4151),
        name: "Abyssal whip".to_string(),
        examine: None,
        members: true,
        buy_limit: Some(70),
        low_alch: Some(Gp(72_000)),
        high_alch: Some(Gp(108_001)),
        value: Some(Gp(120_001)),
        icon: None,
        updated_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
    };
    let latest = LatestPrice {
        item_id: ItemId(4151),
        high: Some(Gp(103_000)),
        high_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 59, 0).unwrap()),
        low: Some(Gp(101_000)),
        low_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 58, 0).unwrap()),
        observed_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
    };
    let mut values = serde_json::Map::new();
    values.insert("return_1h".to_string(), serde_json::json!(0.04));
    values.insert("return_6h".to_string(), serde_json::json!(0.06));
    values.insert("observed_volume_z_24h".to_string(), serde_json::json!(1.5));
    values.insert("spread_pct".to_string(), serde_json::json!(0.02));
    values.insert("ewma_volatility_24h".to_string(), serde_json::json!(0.01));
    values.insert("observed_volume_1h".to_string(), serde_json::json!(400));
    values.insert(
        "observed_high_side_volume_1h".to_string(),
        serde_json::json!(220),
    );
    values.insert(
        "observed_low_side_volume_1h".to_string(),
        serde_json::json!(180),
    );
    values.insert("price_staleness_secs".to_string(), serde_json::json!(60));
    values.insert("z_score_24h".to_string(), serde_json::json!(-2.0));
    let features = FeatureVector {
        item_id: ItemId(4151),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        feature_set_version: "features_v1".to_string(),
        values,
    };
    let config = StrategyConfig {
        enabled_strategies: registry.ids(),
        risk: grand_edge_strategies::RiskConfig::default(),
    };
    let ctx = test_context();

    c.bench_function("strategy_batch_fixture", |b| {
        b.iter(|| registry.generate_all(&config, &ctx, &item, &latest, &features))
    });
}

criterion_group!(benches, strategy_batch);
criterion_main!(benches);
