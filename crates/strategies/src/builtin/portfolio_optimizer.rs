use grand_edge_domain::{FeatureVector, Item, LatestPrice, SignalSide};

use crate::{LookbackSpec, Strategy, StrategyContext, StrategyError};

use super::{base_explanation, feature_i64, strategy_signal};

const STRATEGY_ID: &str = "portfolio_optimizer_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortfolioCandidate {
    pub item_id: i64,
    pub entry_price: i64,
    pub expected_net_gp_per_unit: i64,
    pub max_quantity: i64,
    pub risk_score: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortfolioOrderSuggestion {
    pub item_id: i64,
    pub quantity: i64,
    pub capital_used: i64,
    pub expected_net_gp: i64,
}

#[derive(Debug, Clone, Default)]
pub struct PortfolioOptimizerStrategy;

impl Strategy for PortfolioOptimizerStrategy {
    fn id(&self) -> &'static str {
        STRATEGY_ID
    }

    fn version(&self) -> &'static str {
        STRATEGY_VERSION
    }

    fn required_lookback(&self) -> LookbackSpec {
        LookbackSpec {
            min_5m_buckets: 0,
            min_1h_buckets: 1,
        }
    }

    fn generate(
        &self,
        ctx: &StrategyContext,
        item: &Item,
        _latest: &LatestPrice,
        features: &FeatureVector,
    ) -> Result<grand_edge_domain::StrategySignal, StrategyError> {
        let buy_limit = item.buy_limit.map(i64::from).unwrap_or(0);
        let observed_volume = feature_i64(features, "observed_volume_1h").unwrap_or(0);
        let max_quantity = buy_limit.min(observed_volume).max(1);
        let mut explanation =
            base_explanation(self.id(), self.version(), "portfolio_sizing_helper");
        explanation.insert("max_quantity".to_string(), serde_json::json!(max_quantity));
        explanation.insert(
            "capital_efficiency_note".to_string(),
            serde_json::json!(
                "portfolio optimizer is deterministic greedy sizing, not a final recommendation"
            ),
        );

        strategy_signal(
            self.id(),
            self.version(),
            ctx,
            item,
            SignalSide::Watch,
            0.5,
            0.0,
            0,
            None,
            None,
            None,
            None,
            Some(max_quantity),
            None,
            explanation,
        )
    }
}

pub fn optimize_portfolio(
    capital_gp: i64,
    slot_limit: usize,
    candidates: &[PortfolioCandidate],
) -> Vec<PortfolioOrderSuggestion> {
    let mut ordered = candidates
        .iter()
        .filter(|candidate| {
            candidate.entry_price > 0
                && candidate.expected_net_gp_per_unit > 0
                && candidate.max_quantity > 0
        })
        .cloned()
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        let left_efficiency = left.expected_net_gp_per_unit as f64 / left.entry_price as f64;
        let right_efficiency = right.expected_net_gp_per_unit as f64 / right.entry_price as f64;
        right_efficiency
            .partial_cmp(&left_efficiency)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut remaining_capital = capital_gp.max(0);
    let mut suggestions = Vec::new();
    for candidate in ordered.into_iter().take(slot_limit) {
        let affordable = remaining_capital / candidate.entry_price;
        let quantity = affordable.min(candidate.max_quantity);
        if quantity <= 0 {
            continue;
        }
        let capital_used = quantity * candidate.entry_price;
        remaining_capital -= capital_used;
        suggestions.push(PortfolioOrderSuggestion {
            item_id: candidate.item_id,
            quantity,
            capital_used,
            expected_net_gp: quantity * candidate.expected_net_gp_per_unit,
        });
    }

    suggestions
}
