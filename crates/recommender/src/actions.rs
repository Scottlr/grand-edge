use grand_edge_domain::{
    Gp, LatestPrice, MarketRules, RecommendationAction, StrategySignal, UserPosition,
};

use crate::{RecommendationConfig, RecommendationScore};

pub fn map_action(
    signal: &StrategySignal,
    latest: &LatestPrice,
    score: &RecommendationScore,
    existing_position: Option<&UserPosition>,
    market_rules: &MarketRules,
    config: &RecommendationConfig,
) -> RecommendationAction {
    let expected_net_gp_per_unit = signal.expected_net_gp_per_unit.as_i64();
    let expected_roi = signal.expected_return.get();
    let execution_confidence = score.execution_confidence.unwrap_or(0.0);

    if let Some(position) = existing_position {
        let sell_price = latest.low.or(latest.high);
        let has_after_tax_profit = sell_price.is_some_and(|price| {
            market_rules.net_profit_per_unit(position.item_id, position.avg_buy_price, price)
                > Gp::ZERO
        });

        if has_after_tax_profit
            && (score.final_score < config.min_watch_score
                || matches!(
                    signal.side,
                    grand_edge_domain::SignalSide::Avoid
                        | grand_edge_domain::SignalSide::Sell
                        | grand_edge_domain::SignalSide::Cashout
                ))
        {
            return RecommendationAction::Cashout;
        }

        if score.final_score >= config.min_buy_score
            && expected_net_gp_per_unit > 0
            && execution_confidence >= config.min_execution_confidence
        {
            return RecommendationAction::Add;
        }

        return RecommendationAction::Hold;
    }

    if expected_net_gp_per_unit <= 0 || expected_roi <= 0.0 {
        return RecommendationAction::Avoid;
    }

    if expected_roi > 0.0 && execution_confidence < config.min_execution_confidence {
        return RecommendationAction::Watch;
    }

    if score.final_score >= config.min_buy_score
        && score.recommendation_confidence >= config.min_confidence
        && expected_roi >= config.min_expected_roi
    {
        return RecommendationAction::Buy;
    }

    if score.final_score >= config.min_watch_score {
        RecommendationAction::Watch
    } else {
        RecommendationAction::Avoid
    }
}
