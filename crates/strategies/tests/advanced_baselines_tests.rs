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
    let mut values = serde_json::Map::new();
    values.insert("return_1h".to_string(), serde_json::json!(0.02));
    values.insert("return_6h".to_string(), serde_json::json!(0.04));
    values.insert("observed_volume_1h".to_string(), serde_json::json!(400));

    FeatureVector {
        item_id: ItemId(4151),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        feature_set_version: "features_v1".to_string(),
        values,
    }
}

#[test]
fn advanced_baselines_disabled_by_default() {
    let mut registry = StrategyRegistry::new();
    register_baseline_strategies(&mut registry).unwrap();

    let config = StrategyConfig::default();
    let enabled = registry.enabled(&config);
    assert!(registry.get("kalman_fair_value_v1").is_some());
    assert!(registry.get("arima_baseline_v1").is_some());
    assert!(
        !enabled
            .iter()
            .any(|strategy| strategy.id() == "kalman_fair_value_v1")
    );
    assert!(
        !enabled
            .iter()
            .any(|strategy| strategy.id() == "arima_baseline_v1")
    );
}

#[test]
fn advanced_baselines_emit_stable_explanation_keys() {
    let mut registry = StrategyRegistry::new();
    register_baseline_strategies(&mut registry).unwrap();

    let config = StrategyConfig {
        enabled_strategies: vec![
            "kalman_fair_value_v1".to_string(),
            "arima_baseline_v1".to_string(),
        ],
        ..StrategyConfig::default()
    };
    let results = registry.generate_all(
        &config,
        &grand_edge_strategies::builtin::test_context(),
        &item(),
        &latest(),
        &features(),
    );

    assert_eq!(results.len(), 2);
    for result in results {
        let signal = result
            .signal
            .expect("advanced baseline should emit a signal");
        let explanation = signal.explanation.as_object().unwrap();
        assert!(explanation.contains_key("method"));
        assert!(explanation.contains_key("assumptions"));
        assert!(explanation.contains_key("inputs"));
    }

    let ar_signal = registry
        .get("arima_baseline_v1")
        .unwrap()
        .generate(
            &grand_edge_strategies::builtin::test_context(),
            &item(),
            &latest(),
            &features(),
        )
        .unwrap();
    assert!(
        ar_signal
            .explanation
            .to_string()
            .contains("simplified AR(1) on price differences")
    );
}

#[test]
fn risk_overlay_reason_keys_survive_serialization() {
    let mut registry = StrategyRegistry::new();
    register_baseline_strategies(&mut registry).unwrap();

    let signal = registry
        .get("advanced_risk_overlay_v1")
        .unwrap()
        .generate(
            &grand_edge_strategies::builtin::test_context(),
            &item(),
            &latest(),
            &features(),
        )
        .unwrap();
    let overlay = signal.explanation.get("risk_overlay").unwrap().clone();
    let round_trip: grand_edge_strategies::RiskOverlay = serde_json::from_value(overlay).unwrap();

    assert!(round_trip.reasons.iter().all(|reason| {
        reason
            .key
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch == '_')
    }));
}
