use chrono::{TimeZone, Utc};
use grand_edge_domain::{
    ExecutionEstimate, FeatureVector, Gp, HorizonSecs, ItemId, ObservedLiquidityProxy, Probability,
    Quantity, Rate, SignalSide, StrategyId, StrategySignal,
};

#[test]
fn risk_overlay_regime_penalty_is_visible_in_score_components() {
    let score = grand_edge_recommender::scoring::score_candidate(
        &signal(),
        &features(),
        None,
        &grand_edge_recommender::RecommendationConfig::default(),
    );

    assert!(
        score
            .components
            .iter()
            .any(|component| component.name == "regime_penalty" && component.value < 0.0)
    );
}

fn signal() -> StrategySignal {
    StrategySignal {
        item_id: ItemId(4151),
        strategy_id: StrategyId("advanced_risk_overlay_v1".to_string()),
        model_version: grand_edge_domain::ModelVersion("v1".to_string()),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        side: SignalSide::Watch,
        horizon_secs: HorizonSecs(3_600),
        confidence: Probability::new(0.5).unwrap(),
        expected_return: Rate::new(0.01).unwrap(),
        expected_net_gp_per_unit: Gp(0),
        target_entry: None,
        target_exit: None,
        stop_loss: None,
        take_profit: None,
        max_quantity: None,
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
            estimated_fill_probability: Some(Probability::new(0.5).unwrap()),
            liquidity_confidence: Some(Probability::new(0.5).unwrap()),
            estimated_capacity: Some(Quantity(10)),
            participation_rate: None,
            confidence_haircut: None,
            spread_pct: Some(Rate::new(0.02).unwrap()),
            price_staleness_seconds: Some(HorizonSecs(120)),
            volatility: Some(Rate::new(0.03).unwrap()),
        }),
        explanation: serde_json::json!({
            "risk_overlay": {
                "volatility_penalty": 0.05,
                "spread_penalty": 0.03,
                "staleness_penalty": 0.02,
                "regime_penalty": 0.20,
                "final_multiplier": 0.70,
                "reasons": [
                    { "key": "high_volatility", "label": "High volatility", "penalty": 0.05 },
                    { "key": "wide_spread", "label": "Wide spread", "penalty": 0.03 },
                    { "key": "stale_price", "label": "Stale price", "penalty": 0.02 },
                    { "key": "illiquid_regime", "label": "Illiquid regime", "penalty": 0.20 }
                ]
            },
            "regime": {
                "regime": "illiquid",
                "probability": 0.88,
                "method": "heuristic_v1",
                "strategy_overrides": {}
            }
        }),
    }
}

fn features() -> FeatureVector {
    let mut values = serde_json::Map::new();
    values.insert("ewma_volatility_24h".to_string(), serde_json::json!(0.03));
    values.insert("spread_pct".to_string(), serde_json::json!(0.02));
    values.insert("price_staleness_secs".to_string(), serde_json::json!(120.0));
    FeatureVector {
        item_id: ItemId(4151),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        feature_set_version: "features_v1".to_string(),
        values,
    }
}
