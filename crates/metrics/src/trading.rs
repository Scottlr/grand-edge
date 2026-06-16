use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TradingMetrics {
    pub gross_profit_gp: i64,
    pub net_profit_gp: i64,
    pub win_rate: Option<f64>,
    pub avg_win_gp: Option<f64>,
    pub avg_loss_gp: Option<f64>,
    pub profit_factor: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub capital_efficiency: Option<f64>,
}

pub fn profit_factor(profits: &[i64]) -> Option<f64> {
    if profits.is_empty() {
        return None;
    }

    let gross_profit = profits
        .iter()
        .copied()
        .filter(|value| *value > 0)
        .sum::<i64>() as f64;
    let gross_loss = profits
        .iter()
        .copied()
        .filter(|value| *value < 0)
        .map(|value| value.abs())
        .sum::<i64>() as f64;

    if gross_loss == 0.0 {
        return if gross_profit > 0.0 { None } else { Some(0.0) };
    }

    Some(gross_profit / gross_loss)
}

pub fn win_rate(profits: &[i64]) -> Option<f64> {
    if profits.is_empty() {
        return None;
    }

    Some(profits.iter().filter(|value| **value > 0).count() as f64 / profits.len() as f64)
}

pub fn average_win(profits: &[i64]) -> Option<f64> {
    average_subset(profits, |value| value > 0)
}

pub fn average_loss(profits: &[i64]) -> Option<f64> {
    average_subset(profits, |value| value < 0)
}

pub fn capital_efficiency(entry_notional: &[f64], net_profit: i64) -> Option<f64> {
    if entry_notional.is_empty() {
        return None;
    }

    let deployed = entry_notional.iter().sum::<f64>();
    if deployed <= 0.0 {
        return None;
    }

    Some(net_profit as f64 / deployed)
}

fn average_subset<F>(profits: &[i64], predicate: F) -> Option<f64>
where
    F: Fn(i64) -> bool,
{
    let values = profits
        .iter()
        .copied()
        .filter(|value| predicate(*value))
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }

    Some(values.iter().sum::<i64>() as f64 / values.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::profit_factor;

    #[test]
    fn profit_factor_handles_no_losses() {
        assert_eq!(profit_factor(&[100, 50]), None);
        assert_eq!(profit_factor(&[]), None);
        assert_eq!(profit_factor(&[100, -25, -25]), Some(2.0));
    }
}
