use grand_edge_domain::{
    FeatureVector, GraphRecommendationContext, LatestPrice, MarketRules, Prediction,
    Recommendation, RecommendationAction, RecommendationExplanation, StrategySignal,
    StructuredRecommendationExplanation, UserPosition,
};

use crate::{
    RecommendationConfig, RecommendationError, RecommendationScore,
    confidence::{
        CalibrationSnapshot, ConfidenceInputs, LiquiditySnapshot, build_confidence_breakdown,
    },
    graph_actions::{GraphActionDecision, GraphRecommendationInput, build_graph_reason_atoms},
    reason_atoms::{
        DataQualitySnapshot, ReasonAtomInputs, RiskProfile, build_invalidation_rules,
        build_reason_atoms,
    },
    scoring::ScoreComponent,
};

pub fn build_reasons(
    action: RecommendationAction,
    signal: &StrategySignal,
    latest: &LatestPrice,
    score: &RecommendationScore,
    existing_position: Option<&UserPosition>,
    market_rules: &MarketRules,
) -> Vec<String> {
    let mut reasons = Vec::new();
    reasons.push(format!(
        "Expected net edge is {} gp per unit with final score {:.4}.",
        signal.expected_net_gp_per_unit.as_i64(),
        score.final_score
    ));

    if let Some(execution_confidence) = score.execution_confidence {
        reasons.push(format!(
            "Execution confidence is {:.2}, separate from prediction confidence {:.2}.",
            execution_confidence,
            score.prediction_confidence.unwrap_or(0.0)
        ));
    }

    match action {
        RecommendationAction::Cashout => {
            if let (Some(position), Some(low_price)) = (existing_position, latest.low) {
                let after_tax_profit = market_rules.net_profit_per_unit(
                    position.item_id,
                    position.avg_buy_price,
                    low_price,
                );
                reasons.push(format!(
                    "Current conservative sell price implies after-tax profit of {} gp per unit.",
                    after_tax_profit.as_i64()
                ));
            }
            reasons
                .push("Forecast or confidence deteriorated below the hold threshold.".to_string());
        }
        RecommendationAction::Avoid => {
            if signal.expected_net_gp_per_unit.as_i64() <= 0 {
                reasons.push("Tax-adjusted expected net profit is not positive.".to_string());
            }
            if score.execution_confidence.unwrap_or(0.0) < 0.45 {
                reasons.push(
                    "Liquidity, spread, or staleness makes execution too uncertain.".to_string(),
                );
            }
        }
        RecommendationAction::Watch => {
            reasons.push("Price likely moves up, but execution quality is uncertain.".to_string());
        }
        RecommendationAction::Buy => {
            reasons.push(
                "Expected net edge, confidence, and execution quality clear buy thresholds."
                    .to_string(),
            );
        }
        RecommendationAction::Add => {
            reasons.push(
                "Existing position remains attractive and sizing stays within estimated capacity."
                    .to_string(),
            );
        }
        RecommendationAction::Hold => {
            reasons.push(
                "Existing position retains positive edge, but not enough to cash out or add."
                    .to_string(),
            );
        }
    }

    reasons
}

pub struct ExplanationInputs<'a> {
    pub recommendation: &'a Recommendation,
    pub feature_vector: &'a FeatureVector,
    pub market_rules: &'a MarketRules,
    pub strategy_votes: &'a [StrategySignal],
    pub predictions: &'a [Prediction],
    pub score_components: &'a [ScoreComponent],
    pub accuracy_snapshot: Option<&'a grand_edge_domain::ModelAccuracySnapshot>,
    pub config: &'a RecommendationConfig,
    pub graph_input: Option<&'a GraphRecommendationInput>,
    pub graph_decision: Option<&'a GraphActionDecision>,
}

pub fn build_structured_explanation(
    inputs: ExplanationInputs<'_>,
) -> Result<StructuredRecommendationExplanation, RecommendationError> {
    let data_quality = data_quality_snapshot(inputs.feature_vector);
    let reason_atoms = build_reason_atoms(ReasonAtomInputs {
        recommendation: inputs.recommendation,
        predictions: inputs.predictions,
        score_components: inputs.score_components,
        market_rules: inputs.market_rules,
        risk_profile: &RiskProfile {
            min_buy_score: inputs.config.min_buy_score,
            min_watch_score: inputs.config.min_watch_score,
            min_execution_confidence: inputs.config.min_execution_confidence,
        },
        data_quality: &data_quality,
    })?;
    let graph_reason_atoms = inputs
        .graph_input
        .map(|graph_input| build_graph_reason_atoms(graph_input, inputs.graph_decision))
        .unwrap_or_default();
    let mut all_reason_atoms = reason_atoms;
    all_reason_atoms.extend(graph_reason_atoms);
    let invalidation_rules = build_invalidation_rules(
        inputs.recommendation,
        inputs.score_components,
        inputs.market_rules,
        &RiskProfile {
            min_buy_score: inputs.config.min_buy_score,
            min_watch_score: inputs.config.min_watch_score,
            min_execution_confidence: inputs.config.min_execution_confidence,
        },
    );
    let confidence = build_confidence_breakdown(ConfidenceInputs {
        predictions: inputs.predictions,
        score_components: inputs.score_components,
        calibration: Some(&CalibrationSnapshot {
            recent_directional_accuracy: inputs
                .accuracy_snapshot
                .and_then(|snapshot| snapshot.directional_accuracy)
                .map(|value| value.get()),
        }),
        liquidity: Some(&LiquiditySnapshot {
            liquidity_confidence: inputs
                .strategy_votes
                .iter()
                .filter_map(|vote| {
                    vote.execution_estimate
                        .as_ref()
                        .and_then(|estimate| estimate.liquidity_confidence)
                        .map(|value| value.get())
                })
                .reduce(f64::max),
        }),
        data_quality: &data_quality,
        explanation_atoms: &all_reason_atoms,
    })?;

    let graph_context = inputs
        .graph_input
        .map(|graph_input| GraphRecommendationContext {
            graph_version: graph_input.graph_version.clone(),
            graph_action: inputs.graph_decision.map(|decision| decision.action),
            paths: graph_input.graph_paths.clone(),
            edge_confidence: inputs
                .graph_decision
                .and_then(|decision| decision.edge_confidence),
            historical_path_performance: graph_input.historical_path_performance.clone(),
        });
    let summary = derive_summary(inputs.recommendation.action, &all_reason_atoms, &confidence);
    Ok(StructuredRecommendationExplanation {
        summary,
        reason_atoms: all_reason_atoms,
        invalidation_rules,
        confidence,
        graph_version: graph_context
            .as_ref()
            .map(|value| value.graph_version.clone()),
        graph_reason_path_count: graph_context.as_ref().map(|value| value.paths.len()),
        graph_context,
    })
}

pub fn build_explanation(
    feature_vector: &FeatureVector,
    market_rules: &MarketRules,
    strategy_votes: Vec<StrategySignal>,
    predictions: &[Prediction],
    score_components: &[ScoreComponent],
    accuracy_snapshot: Option<grand_edge_domain::ModelAccuracySnapshot>,
    recommendation: &Recommendation,
    config: &RecommendationConfig,
    graph_input: Option<&GraphRecommendationInput>,
    graph_decision: Option<&GraphActionDecision>,
) -> Result<RecommendationExplanation, RecommendationError> {
    let structured_explanation = build_structured_explanation(ExplanationInputs {
        recommendation,
        feature_vector,
        market_rules,
        strategy_votes: &strategy_votes,
        predictions,
        score_components,
        accuracy_snapshot: accuracy_snapshot.as_ref(),
        config,
        graph_input,
        graph_decision,
    })?;

    Ok(RecommendationExplanation {
        feature_set_version: feature_vector.feature_set_version.clone(),
        market_rules_version: market_rules.version.clone(),
        graph_version: structured_explanation.graph_version.clone(),
        graph_context: structured_explanation.graph_context.clone(),
        strategy_votes,
        score_components: score_components
            .iter()
            .map(to_domain_score_component)
            .collect(),
        accuracy_snapshot,
        structured_explanation,
    })
}

fn to_domain_score_component(component: &ScoreComponent) -> grand_edge_domain::ScoreComponent {
    grand_edge_domain::ScoreComponent {
        key: component.name.clone(),
        label: component.name.replace('_', " "),
        value: grand_edge_domain::Rate::new(component.value)
            .unwrap_or_else(|_| grand_edge_domain::Rate(0.0)),
        weight: None,
    }
}

fn data_quality_snapshot(feature_vector: &FeatureVector) -> DataQualitySnapshot {
    let stale = feature_vector
        .values
        .get("price_staleness_secs")
        .and_then(|value| value.as_f64())
        .map(|value| value > 300.0)
        .unwrap_or(true);
    let missing_inputs = ["ewma_volatility_24h", "spread_pct", "price_staleness_secs"]
        .into_iter()
        .filter(|key| !feature_vector.values.contains_key(*key))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let freshness_confidence = if stale { 0.25 } else { 0.9 };
    let completeness_confidence = if missing_inputs.is_empty() { 1.0 } else { 0.5 };
    DataQualitySnapshot {
        freshness_confidence,
        completeness_confidence,
        stale,
        missing_inputs,
    }
}

fn derive_summary(
    action: RecommendationAction,
    reason_atoms: &[grand_edge_domain::ReasonAtom],
    confidence: &grand_edge_domain::ConfidenceBreakdown,
) -> String {
    let dominant_reason = reason_atoms
        .iter()
        .max_by(|left, right| {
            left.weight
                .partial_cmp(&right.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|atom| atom.label.clone())
        .unwrap_or_else(|| "No dominant reason".to_string());
    format!(
        "{:?} because {dominant_reason}; recommendation confidence {:.2}",
        action,
        confidence.recommendation_confidence.get()
    )
}
