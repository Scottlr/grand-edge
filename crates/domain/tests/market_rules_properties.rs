use grand_edge_domain::{Gp, ItemId, MarketRules, Rate};
use proptest::prelude::*;

proptest! {
    #[test]
    fn tax_is_never_negative(sell_price in 0_i64..10_000_000_000_i64) {
        let rules = MarketRules::default();
        let tax = rules.tax_for_sale(ItemId(4151), Gp(sell_price));
        prop_assert!(tax.as_i64() >= 0);
    }

    #[test]
    fn net_profit_decreases_when_tax_rate_increases(
        buy_price in 1_i64..5_000_000_i64,
        sell_price in 100_i64..10_000_000_i64,
        lower_rate in 0.0_f64..0.03_f64,
        higher_rate_delta in 0.0_f64..0.03_f64
    ) {
        let higher_rate = (lower_rate + higher_rate_delta).min(0.10);
        let mut lower_rules = MarketRules::default();
        lower_rules.tax_rate = Rate::new(lower_rate).unwrap();

        let mut higher_rules = MarketRules::default();
        higher_rules.tax_rate = Rate::new(higher_rate).unwrap();

        let lower_profit = lower_rules.net_profit_per_unit(ItemId(4151), Gp(buy_price), Gp(sell_price));
        let higher_profit = higher_rules.net_profit_per_unit(ItemId(4151), Gp(buy_price), Gp(sell_price));

        prop_assert!(higher_profit.as_i64() <= lower_profit.as_i64());
    }
}
