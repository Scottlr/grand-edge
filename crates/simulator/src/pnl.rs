use grand_edge_domain::{Gp, ItemId, MarketRules};

pub fn tax_on_sale(market_rules: &MarketRules, item_id: i64, sell_price: i64) -> i64 {
    market_rules
        .tax_for_sale(ItemId(item_id), Gp(sell_price))
        .as_i64()
}

pub fn realized_profit_gp(
    market_rules: &MarketRules,
    item_id: i64,
    entry_price: i64,
    exit_price: i64,
    quantity: i64,
) -> i64 {
    let per_unit = market_rules
        .net_profit_per_unit(ItemId(item_id), Gp(entry_price), Gp(exit_price))
        .as_i64();
    per_unit * quantity
}

pub fn realized_roi(entry_price: i64, quantity: i64, realized_profit_gp: i64) -> Option<f64> {
    let capital = entry_price.checked_mul(quantity)?;
    if capital <= 0 {
        return None;
    }

    Some(realized_profit_gp as f64 / capital as f64)
}
