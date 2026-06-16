use grand_edge_domain::{Quantity, StrategySignal};

pub fn size_quantity(signal: &StrategySignal) -> Option<Quantity> {
    let requested = signal.max_quantity?;
    let estimated_capacity = signal
        .execution_estimate
        .as_ref()
        .and_then(|estimate| estimate.estimated_capacity);

    match estimated_capacity {
        Some(capacity) => Quantity::try_from(requested.as_i64().min(capacity.as_i64())).ok(),
        None => Some(requested),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        ExecutionEstimate, Gp, HorizonSecs, ItemId, ModelVersion, ObservedLiquidityProxy,
        Probability, Quantity, Rate, SignalSide, StrategyId, StrategySignal,
    };

    use super::size_quantity;

    #[test]
    fn quantity_uses_estimated_capacity_not_raw_observed_volume() {
        let signal = StrategySignal {
            item_id: ItemId(4151),
            strategy_id: StrategyId("momentum_v1".to_string()),
            model_version: ModelVersion("v1".to_string()),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            side: SignalSide::Buy,
            horizon_secs: HorizonSecs(3_600),
            confidence: Probability::new(0.8).unwrap(),
            expected_return: Rate::new(0.04).unwrap(),
            expected_net_gp_per_unit: Gp(1_200),
            target_entry: Some(Gp(100_000)),
            target_exit: Some(Gp(104_000)),
            stop_loss: None,
            take_profit: None,
            max_quantity: Some(Quantity(40)),
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
                liquidity_confidence: Some(Probability::new(0.6).unwrap()),
                estimated_capacity: Some(Quantity(10)),
                participation_rate: Some(Probability::new(0.05).unwrap()),
                confidence_haircut: Some(Probability::new(0.5).unwrap()),
                spread_pct: None,
                price_staleness_seconds: None,
                volatility: None,
            }),
            explanation: serde_json::json!({}),
        };

        assert_eq!(size_quantity(&signal), Some(Quantity(10)));
    }
}
