use chrono::{TimeZone, Utc};
use grand_edge_api::recommendations::view::RecommendationDto;
use grand_edge_domain::{
    ConfidenceBreakdown, ExecutionEstimate, Gp, HorizonSecs, InvalidationRule, Item, ItemIcon,
    ItemId, ModelVersion, ObservedLiquidityProxy, Probability, Rate, ReasonAtom, ReasonDirection,
    ReasonType, Recommendation, RecommendationAction, RecommendationExplanation, RecommendationId,
    SignalSide, StrategyId, StrategySignal, StructuredRecommendationExplanation, UserId,
    WikiImageSource,
};
use insta::assert_json_snapshot;

fn base_item(cdn_url: &str) -> Item {
    Item {
        item_id: ItemId(4151),
        name: "Abyssal whip".to_string(),
        examine: None,
        members: true,
        buy_limit: Some(70),
        low_alch: None,
        high_alch: None,
        value: None,
        icon: Some(ItemIcon {
            source_file_name: "Abyssal whip.png".to_string(),
            canonical_file_name: "Abyssal_whip.png".to_string(),
            cdn_url: cdn_url.to_string(),
            source: WikiImageSource::MappingIcon,
        }),
        updated_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
    }
}

fn data_quality_atom(stale: bool) -> ReasonAtom {
    ReasonAtom {
        reason_type: ReasonType::DataQualityCheck,
        reason_key: "data_quality:freshness_completeness".to_string(),
        label: "Data quality".to_string(),
        direction: if stale {
            ReasonDirection::Negative
        } else {
            ReasonDirection::Positive
        },
        weight: if stale { 0.25 } else { 0.95 },
        evidence: serde_json::json!({
            "freshness_confidence": if stale { 0.25 } else { 0.9 },
            "completeness_confidence": 1.0,
            "stale": stale,
            "missing_inputs": if stale {
                vec!["price_staleness_secs"]
            } else {
                Vec::<&str>::new()
            },
        }),
    }
}

fn strategy_signal() -> StrategySignal {
    StrategySignal {
        item_id: ItemId(4151),
        strategy_id: StrategyId("spread_edge_v1".to_string()),
        model_version: ModelVersion("v1".to_string()),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        side: SignalSide::Buy,
        horizon_secs: HorizonSecs(3600),
        confidence: Probability::new(0.8).unwrap(),
        expected_return: Rate::new(0.03).unwrap(),
        expected_net_gp_per_unit: Gp(1_400),
        target_entry: Some(Gp(100_000)),
        target_exit: Some(Gp(104_000)),
        stop_loss: Some(Gp(99_000)),
        take_profit: Some(Gp(104_000)),
        max_quantity: Some(grand_edge_domain::Quantity(8)),
        execution_estimate: Some(ExecutionEstimate {
            observed_liquidity: ObservedLiquidityProxy {
                observed_volume: grand_edge_domain::Quantity(400),
                observed_high_side_volume: grand_edge_domain::Quantity(220),
                observed_low_side_volume: grand_edge_domain::Quantity(180),
                observed_volume_z: Some(Rate::new(1.2).unwrap()),
                observed_volume_reliability: Some(Probability::new(0.7).unwrap()),
                high_low_volume_ratio: Some(Rate::new(1.22).unwrap()),
                note: "Observed volume is a proxy, not true GE depth.".to_string(),
            },
            estimated_fill_probability: Some(Probability::new(0.6).unwrap()),
            liquidity_confidence: Some(Probability::new(0.7).unwrap()),
            estimated_capacity: Some(grand_edge_domain::Quantity(8)),
            participation_rate: Some(Probability::new(0.05).unwrap()),
            confidence_haircut: Some(Probability::new(0.5).unwrap()),
            spread_pct: Some(Rate::new(0.02).unwrap()),
            price_staleness_seconds: Some(HorizonSecs(120)),
            volatility: Some(Rate::new(0.03).unwrap()),
        }),
        explanation: serde_json::json!({"reason": "fixture"}),
    }
}

fn recommendation_fixture(stale: bool, missing_accuracy: bool, degraded: bool) -> Recommendation {
    Recommendation {
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
        reasons: if degraded {
            Vec::new()
        } else {
            vec!["Tax-adjusted edge clears threshold".to_string()]
        },
        explanation: RecommendationExplanation {
            feature_set_version: "features_v1".to_string(),
            market_rules_version: "osrs_rules_v1_review_required".to_string(),
            graph_version: None,
            graph_context: None,
            strategy_votes: if degraded {
                Vec::new()
            } else {
                vec![strategy_signal()]
            },
            score_components: vec![
                grand_edge_domain::ScoreComponent {
                    key: "edge".to_string(),
                    label: "Edge".to_string(),
                    value: Rate::new(0.7).unwrap(),
                    weight: Some(Rate::new(0.5).unwrap()),
                },
                grand_edge_domain::ScoreComponent {
                    key: "liquidity_penalty".to_string(),
                    label: "Liquidity penalty".to_string(),
                    value: Rate::new(-0.05).unwrap(),
                    weight: None,
                },
            ],
            accuracy_snapshot: if missing_accuracy {
                None
            } else {
                Some(grand_edge_domain::ModelAccuracySnapshot {
                    strategy_id: StrategyId("spread_edge_v1".to_string()),
                    model_version: ModelVersion("v1".to_string()),
                    lookback_window: "seven_days".to_string(),
                    sample_size: 12,
                    directional_accuracy: Some(Rate::new(0.66).unwrap()),
                    brier_score: Some(Rate::new(0.18).unwrap()),
                    avg_realized_roi: Some(Rate::new(0.02).unwrap()),
                    max_drawdown: Some(Rate::new(0.1).unwrap()),
                    calibration: serde_json::json!({}),
                })
            },
            structured_explanation: StructuredRecommendationExplanation {
                summary: "Buy because tax-adjusted edge clears threshold.".to_string(),
                reason_atoms: if degraded {
                    Vec::new()
                } else {
                    vec![data_quality_atom(stale)]
                },
                invalidation_rules: vec![InvalidationRule {
                    rule_key: "score_threshold".to_string(),
                    label: "Score threshold".to_string(),
                    metric: "final_score".to_string(),
                    operator: "<".to_string(),
                    threshold: "0.05".to_string(),
                    current_value: Some("0.9".to_string()),
                }],
                confidence: ConfidenceBreakdown {
                    prediction_confidence: Probability::new(0.8).unwrap(),
                    recommendation_confidence: Probability::new(0.75).unwrap(),
                    data_quality_confidence: Probability::new(if stale { 0.25 } else { 0.9 })
                        .unwrap(),
                    model_calibration_confidence: Probability::new(0.8).unwrap(),
                    liquidity_confidence: Probability::new(0.7).unwrap(),
                    explanation_confidence: Probability::new(0.72).unwrap(),
                },
                graph_version: None,
                graph_reason_path_count: None,
                graph_context: None,
            },
        },
    }
}

#[test]
fn api_recommendations_snapshot_is_stable() {
    assert_json_snapshot!(
        "api_recommendation_dto",
        RecommendationDto::from_parts(
            recommendation_fixture(false, true, false),
            Some(base_item(
                "https://oldschool.runescape.wiki/images/Abyssal_whip.png"
            )),
        )
    );
}

#[test]
fn recommendation_dto_preserves_missing_accuracy_as_none() {
    let dto = RecommendationDto::from_parts(
        recommendation_fixture(false, true, false),
        Some(base_item(
            "https://oldschool.runescape.wiki/images/Abyssal_whip.png",
        )),
    );

    assert_eq!(dto.accuracy, None);
    assert_eq!(
        dto.data_state,
        grand_edge_api::market::status_view::DataStateDto::Live
    );
}

#[test]
fn recommendation_dto_includes_sanitized_item_icon() {
    let dto = RecommendationDto::from_parts(
        recommendation_fixture(false, false, false),
        Some(base_item(
            "https://oldschool.runescape.wiki/images/Abyssal_whip.png",
        )),
    );

    assert_eq!(
        dto.item_icon.as_ref().map(|icon| icon.cdn_url.as_str()),
        Some("https://oldschool.runescape.wiki/images/Abyssal_whip.png")
    );
}

#[test]
fn recommendation_dto_preserves_percent_27_icon_url() {
    let dto = RecommendationDto::from_parts(
        recommendation_fixture(false, false, false),
        Some(base_item(
            "https://oldschool.runescape.wiki/images/Chef%27s_hat.png",
        )),
    );

    assert_eq!(
        dto.item_icon.unwrap().cdn_url,
        "https://oldschool.runescape.wiki/images/Chef%27s_hat.png"
    );
}

#[test]
fn recommendation_dto_marks_stale_data() {
    let dto = RecommendationDto::from_parts(
        recommendation_fixture(true, false, false),
        Some(base_item(
            "https://oldschool.runescape.wiki/images/Abyssal_whip.png",
        )),
    );

    assert_eq!(
        dto.data_state,
        grand_edge_api::market::status_view::DataStateDto::Stale
    );
    assert!(dto.market_status.stale_reason.is_some());
}

#[test]
fn recommendation_dto_keeps_prediction_execution_and_recommendation_confidence_separate() {
    let dto = RecommendationDto::from_parts(
        recommendation_fixture(false, false, false),
        Some(base_item(
            "https://oldschool.runescape.wiki/images/Abyssal_whip.png",
        )),
    );

    assert_eq!(dto.prediction_confidence, Some(0.8));
    assert_eq!(dto.execution_confidence, Some(0.7));
    assert_eq!(dto.recommendation_confidence, 0.75);
}

#[test]
fn execution_confidence_dto_uses_estimated_fill_probability_name() {
    let dto = RecommendationDto::from_parts(
        recommendation_fixture(false, false, false),
        Some(base_item(
            "https://oldschool.runescape.wiki/images/Abyssal_whip.png",
        )),
    );
    let serialized = serde_json::to_value(&dto).unwrap();

    assert_eq!(
        serialized["execution"]["estimatedFillProbability"],
        serde_json::json!(0.6)
    );
    assert!(serialized["execution"].get("fillProbability").is_none());
}

#[test]
fn invalidation_rules_are_structured() {
    let dto = RecommendationDto::from_parts(
        recommendation_fixture(false, false, false),
        Some(base_item(
            "https://oldschool.runescape.wiki/images/Abyssal_whip.png",
        )),
    );
    let rule = &dto.invalidation_rules[0];

    assert_eq!(rule.metric, "final_score");
    assert_eq!(rule.operator, "<");
    assert_eq!(rule.threshold, "0.05");
    assert_eq!(rule.current_value.as_deref(), Some("0.9"));
}

#[test]
fn recommendation_dto_marks_degraded_data_when_evidence_is_incomplete() {
    let dto = RecommendationDto::from_parts(
        recommendation_fixture(false, false, true),
        Some(base_item(
            "https://oldschool.runescape.wiki/images/Abyssal_whip.png",
        )),
    );

    assert_eq!(
        dto.data_state,
        grand_edge_api::market::status_view::DataStateDto::Degraded
    );
    assert_eq!(
        dto.market_status.degraded_reason.as_deref(),
        Some("Recommendation evidence is incomplete.")
    );
}
