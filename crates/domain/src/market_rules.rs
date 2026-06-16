use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketRules {
    pub version: String,
    pub tax_rate: f64,
    pub tax_cap_gp: i64,
    pub tax_min_price_gp: i64,
    pub slot_limit: usize,
    pub buy_limit_window_secs: i64,
    #[serde(default)]
    pub tax_exempt_item_ids: Vec<i64>,
}

impl Default for MarketRules {
    fn default() -> Self {
        Self {
            version: "osrs_rules_v1_review_required".to_string(),
            tax_rate: 0.02,
            tax_cap_gp: 5_000_000,
            tax_min_price_gp: 100,
            slot_limit: 8,
            buy_limit_window_secs: 14_400,
            tax_exempt_item_ids: Vec::new(),
        }
    }
}

impl MarketRules {
    pub fn tax_for_sale(&self, item_id: i64, sell_price_gp: i64) -> i64 {
        if sell_price_gp < self.tax_min_price_gp || self.tax_exempt_item_ids.contains(&item_id) {
            return 0;
        }

        let raw_tax = (sell_price_gp as f64 * self.tax_rate).floor() as i64;
        raw_tax.min(self.tax_cap_gp)
    }

    pub fn net_profit_per_unit(&self, item_id: i64, buy_price_gp: i64, sell_price_gp: i64) -> i64 {
        sell_price_gp - buy_price_gp - self.tax_for_sale(item_id, sell_price_gp)
    }
}

#[cfg(test)]
mod tests {
    use super::MarketRules;

    #[test]
    fn market_rules_tax_fixture_matches_goal() {
        let rules = MarketRules::default();
        assert_eq!(rules.tax_for_sale(4151, 103_000), 2_060);
        assert_eq!(rules.net_profit_per_unit(4151, 100_000, 103_000), 940);
    }

    #[test]
    fn market_rules_caps_tax() {
        let rules = MarketRules::default();
        assert_eq!(rules.tax_for_sale(4151, 500_000_000), 5_000_000);
    }

    #[test]
    fn market_rules_respects_tax_minimum() {
        let rules = MarketRules::default();
        assert_eq!(rules.tax_for_sale(4151, 99), 0);
    }

    #[test]
    fn market_rules_respects_tax_exemption() {
        let mut rules = MarketRules::default();
        rules.tax_exempt_item_ids.push(4151);
        assert_eq!(rules.tax_for_sale(4151, 103_000), 0);
    }
}
