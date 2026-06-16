use chrono::{TimeZone, Utc};
use grand_edge_domain::{FeatureVector, Gp, Item, ItemId, LatestPrice};
use grand_edge_strategies::{StrategyConfig, StrategyRegistry, register_baseline_strategies};

fn item() -> Item {
    Item {
        item_id: ItemId(4151),
        name: "Abyssal whip".to_string(),
        examine: None,
        members: true,
        buy_limit: Some(70),
        low_alch: None,
        high_alch: None,
        value: None,
        icon: None,
        updated_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
    }
}

fn latest() -> LatestPrice {
    LatestPrice {
        item_id: ItemId(4151),
        high: Some(Gp(103_000)),
        high_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 59, 0).unwrap()),
        low: Some(Gp(101_000)),
        low_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 58, 0).unwrap()),
        observed_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
    }
}

fn features() -> FeatureVector {
    FeatureVector {
        item_id: ItemId(4151),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        feature_set_version: "features_v1".to_string(),
        values: serde_json::Map::new(),
    }
}

#[test]
fn artifacts_disabled_by_default() {
    let mut registry = StrategyRegistry::new();
    register_baseline_strategies(&mut registry).unwrap();

    let config = StrategyConfig::default();
    let enabled = registry.enabled(&config);
    for strategy_id in [
        "gbm_ranker_v1",
        "contextual_bandit_v1",
        "online_ensemble_v1",
        "meta_label_v1",
    ] {
        assert!(registry.get(strategy_id).is_some(), "{strategy_id} missing");
        assert!(!enabled.iter().any(|strategy| strategy.id() == strategy_id));
    }
}

#[test]
fn artifacts_missing_gbdt_artifact_returns_error() {
    let mut registry = StrategyRegistry::new();
    register_baseline_strategies(&mut registry).unwrap();

    let config = StrategyConfig {
        enabled_strategies: vec!["gbm_ranker_v1".to_string()],
        ..StrategyConfig::default()
    };
    let results = registry.generate_all(
        &config,
        &grand_edge_strategies::builtin::test_context(),
        &item(),
        &latest(),
        &features(),
    );

    assert_eq!(results.len(), 1);
    assert!(results[0].signal.is_none());
    assert!(matches!(
        results[0].error.as_ref(),
        Some(grand_edge_strategies::StrategyError::MissingArtifact(strategy_id))
            if strategy_id == "gbm_ranker_v1"
    ));
}
