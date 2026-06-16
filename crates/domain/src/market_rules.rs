use serde::{Deserialize, Serialize};

use crate::{Gp, ItemId, Rate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketRules {
    pub version: String,
    pub tax_rate: Rate,
    pub tax_cap_gp: Gp,
    pub tax_min_price_gp: Gp,
    pub slot_limit: usize,
    pub buy_limit_window_secs: crate::HorizonSecs,
    #[serde(default)]
    pub tax_exempt_item_ids: Vec<ItemId>,
}

impl Default for MarketRules {
    fn default() -> Self {
        Self {
            version: "osrs_rules_v1_review_required".to_string(),
            tax_rate: Rate::new(0.02).expect("default tax rate must be valid"),
            tax_cap_gp: Gp(5_000_000),
            tax_min_price_gp: Gp(100),
            slot_limit: 8,
            buy_limit_window_secs: crate::HorizonSecs(14_400),
            tax_exempt_item_ids: Vec::new(),
        }
    }
}

impl MarketRules {
    pub fn tax_for_sale(&self, item_id: ItemId, sell_price_gp: Gp) -> Gp {
        if sell_price_gp < self.tax_min_price_gp || self.tax_exempt_item_ids.contains(&item_id) {
            return Gp::ZERO;
        }

        let raw_tax = (sell_price_gp.as_i64() as f64 * self.tax_rate.get()).floor() as i64;
        Gp(raw_tax.min(self.tax_cap_gp.as_i64()))
    }

    pub fn net_profit_per_unit(&self, item_id: ItemId, buy_price_gp: Gp, sell_price_gp: Gp) -> Gp {
        Gp(sell_price_gp.as_i64()
            - buy_price_gp.as_i64()
            - self.tax_for_sale(item_id, sell_price_gp).as_i64())
    }
}

#[cfg(test)]
mod tests {
    use super::MarketRules;

    #[test]
    fn market_rules_tax_fixture_matches_goal() {
        let rules = MarketRules::default();
        assert_eq!(
            rules.tax_for_sale(crate::ItemId(4151), crate::Gp(103_000)),
            crate::Gp(2_060)
        );
        assert_eq!(
            rules.net_profit_per_unit(crate::ItemId(4151), crate::Gp(100_000), crate::Gp(103_000)),
            crate::Gp(940)
        );
    }

    #[test]
    fn market_rules_caps_tax() {
        let rules = MarketRules::default();
        assert_eq!(
            rules.tax_for_sale(crate::ItemId(4151), crate::Gp(500_000_000)),
            crate::Gp(5_000_000)
        );
    }

    #[test]
    fn market_rules_respects_tax_minimum() {
        let rules = MarketRules::default();
        assert_eq!(
            rules.tax_for_sale(crate::ItemId(4151), crate::Gp(99)),
            crate::Gp::ZERO
        );
    }

    #[test]
    fn market_rules_respects_tax_exemption() {
        let mut rules = MarketRules::default();
        rules.tax_exempt_item_ids.push(crate::ItemId(4151));
        assert_eq!(
            rules.tax_for_sale(crate::ItemId(4151), crate::Gp(103_000)),
            crate::Gp::ZERO
        );
    }
}
