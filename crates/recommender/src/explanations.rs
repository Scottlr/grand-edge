use grand_edge_domain::{
    FeatureVector, LatestPrice, MarketRules, RecommendationAction, RecommendationExplanation,
    ScoreComponent as DomainScoreComponent, StrategySignal, UserPosition,
};

use crate::{RecommendationScore, scoring::ScoreComponent};

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

pub fn build_explanation(
    feature_vector: &FeatureVector,
    market_rules: &MarketRules,
    strategy_votes: Vec<StrategySignal>,
    score_components: &[ScoreComponent],
    accuracy_snapshot: Option<grand_edge_domain::ModelAccuracySnapshot>,
) -> RecommendationExplanation {
    RecommendationExplanation {
        feature_set_version: feature_vector.feature_set_version.clone(),
        market_rules_version: market_rules.version.clone(),
        strategy_votes,
        score_components: score_components
            .iter()
            .map(to_domain_score_component)
            .collect(),
        accuracy_snapshot,
    }
}

fn to_domain_score_component(component: &ScoreComponent) -> DomainScoreComponent {
    DomainScoreComponent {
        key: component.name.clone(),
        label: component.name.replace('_', " "),
        value: grand_edge_domain::Rate::new(component.value)
            .unwrap_or_else(|_| grand_edge_domain::Rate(0.0)),
        weight: None,
    }
}
