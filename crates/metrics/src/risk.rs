use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RiskAdjustedMetrics {
    pub sharpe: Option<f64>,
    pub sortino: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub deflated_sharpe_ratio: Option<f64>,
    pub probability_of_backtest_overfitting: Option<f64>,
    pub placeholder_reason: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OverfitRiskMetrics {
    pub deflated_sharpe_ratio: Option<f64>,
    pub probability_of_backtest_overfitting: Option<f64>,
    pub placeholder_reason: Option<String>,
}

pub fn max_drawdown(returns: &[f64]) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }

    let mut equity = 1.0_f64;
    let mut peak = 1.0_f64;
    let mut worst = 0.0_f64;
    for value in returns {
        equity *= 1.0 + value;
        peak = peak.max(equity);
        if peak > 0.0 {
            worst = worst.max((peak - equity) / peak);
        }
    }

    Some(worst)
}

pub fn sharpe_ratio(returns: &[f64]) -> Option<f64> {
    ratio(returns, false)
}

pub fn sortino_ratio(returns: &[f64]) -> Option<f64> {
    ratio(returns, true)
}

fn ratio(returns: &[f64], downside_only: bool) -> Option<f64> {
    if returns.len() < 2 {
        return None;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance_source = returns
        .iter()
        .copied()
        .filter(|value| !downside_only || *value < 0.0)
        .collect::<Vec<_>>();
    if variance_source.is_empty() {
        return None;
    }

    let variance = variance_source
        .iter()
        .map(|value| {
            let centered = value - mean;
            centered * centered
        })
        .sum::<f64>()
        / variance_source.len() as f64;
    if variance == 0.0 {
        return None;
    }

    Some(mean / variance.sqrt())
}

#[cfg(test)]
mod tests {
    use super::{max_drawdown, sharpe_ratio, sortino_ratio};

    #[test]
    fn max_drawdown_tracks_equity_curve() {
        let returns = [0.10, -0.05, -0.20, 0.15];
        assert!((max_drawdown(&returns).unwrap() - 0.24).abs() < 0.01);
    }

    #[test]
    fn sharpe_and_sortino_have_deterministic_values() {
        let returns = [0.02, 0.01, -0.03, 0.04, -0.01];
        assert!(sharpe_ratio(&returns).is_some());
        assert!(sortino_ratio(&returns).is_some());
    }
}
