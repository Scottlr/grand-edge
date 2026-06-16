use chrono::{DateTime, Utc};
use grand_edge_domain::{IntervalPrice, LatestPrice};

pub fn mid_price(high: Option<i64>, low: Option<i64>) -> Option<f64> {
    match (high, low) {
        (Some(high), Some(low)) => Some((high as f64 + low as f64) / 2.0),
        _ => None,
    }
}

pub fn spread_abs(high: Option<i64>, low: Option<i64>) -> Option<i64> {
    match (high, low) {
        (Some(high), Some(low)) => Some(high - low),
        _ => None,
    }
}

pub fn spread_pct(high: Option<i64>, low: Option<i64>) -> Option<f64> {
    let spread = spread_abs(high, low)? as f64;
    let mid = mid_price(high, low)?;
    if mid <= 0.0 {
        return None;
    }

    Some(spread / mid)
}

pub fn log_return(current_mid: f64, previous_mid: f64) -> Option<f64> {
    if current_mid <= 0.0 || previous_mid <= 0.0 {
        return None;
    }

    Some((current_mid / previous_mid).ln())
}

pub fn rolling_mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    Some(values.iter().sum::<f64>() / values.len() as f64)
}

pub fn rolling_std(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }

    let mean = rolling_mean(values)?;
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;

    Some(variance.sqrt())
}

pub fn z_score(value: f64, mean: f64, std: f64) -> Option<f64> {
    if std <= f64::EPSILON {
        return None;
    }

    Some((value - mean) / std)
}

pub fn ewma_variance(returns: &[f64], lambda: f64) -> Option<f64> {
    if returns.is_empty() || !(0.0..=1.0).contains(&lambda) {
        return None;
    }

    let mut variance = returns[0];
    for squared_return in returns.iter().skip(1) {
        variance = lambda * variance + (1.0 - lambda) * squared_return;
    }

    Some(variance)
}

pub fn observed_volume(row: &IntervalPrice) -> i64 {
    observed_high_side_volume(row) + observed_low_side_volume(row)
}

pub fn observed_high_side_volume(row: &IntervalPrice) -> i64 {
    row.high_price_volume
}

pub fn observed_low_side_volume(row: &IntervalPrice) -> i64 {
    row.low_price_volume
}

pub fn high_low_volume_ratio(row: &IntervalPrice) -> Option<f64> {
    if row.low_price_volume <= 0 {
        return None;
    }

    Some(row.high_price_volume as f64 / row.low_price_volume as f64)
}

pub fn observed_volume_reliability(history: &[IntervalPrice]) -> Option<f64> {
    if history.is_empty() {
        return None;
    }

    let coverage = history
        .iter()
        .filter(|row| row.high_price_volume > 0 || row.low_price_volume > 0)
        .count() as f64
        / history.len() as f64;
    let volumes: Vec<f64> = history
        .iter()
        .map(|row| observed_volume(row) as f64)
        .collect();
    let mean = rolling_mean(&volumes)?;
    let std = rolling_std(&volumes).unwrap_or(0.0);
    let stability = if mean <= f64::EPSILON {
        0.0
    } else {
        (1.0 - (std / mean)).clamp(0.0, 1.0)
    };
    let sample_score = (history.len() as f64 / 24.0).clamp(0.0, 1.0);

    Some(((coverage + stability + sample_score) / 3.0).clamp(0.0, 1.0))
}

pub fn spread_stability(spread_pct_values: &[f64]) -> Option<f64> {
    let std = rolling_std(spread_pct_values)?;
    Some((1.0 / (1.0 + std)).clamp(0.0, 1.0))
}

pub fn price_staleness_secs(latest: &LatestPrice, as_of: DateTime<Utc>) -> Option<i64> {
    let newest_market_time = [latest.high_time, latest.low_time]
        .into_iter()
        .flatten()
        .max()?;
    Some((as_of - newest_market_time).num_seconds())
}

pub fn alch_floor_distance(mid: Option<f64>, high_alch: Option<i64>) -> Option<f64> {
    let mid = mid?;
    let high_alch = high_alch?;
    if mid <= 0.0 {
        return None;
    }

    Some((mid - high_alch as f64) / mid)
}

pub fn buy_limit_utilization(quantity_hint: Option<i64>, buy_limit: Option<i32>) -> Option<f64> {
    let quantity_hint = quantity_hint?;
    let buy_limit = i64::from(buy_limit?);
    if buy_limit <= 0 {
        return None;
    }

    Some((quantity_hint as f64 / buy_limit as f64).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::{
        buy_limit_utilization, ewma_variance, log_return, mid_price, observed_volume, rolling_mean,
        rolling_std, spread_abs, spread_pct, z_score,
    };
    use crate::fixtures::interval_price_row;
    use grand_edge_domain::PriceInterval;

    #[test]
    fn mid_spread_handles_missing_side() {
        assert_eq!(mid_price(Some(100), None), None);
        assert_eq!(spread_abs(Some(100), None), None);
        assert_eq!(spread_pct(Some(100), None), None);
    }

    #[test]
    fn z_score_fixture_matches_goal() {
        assert_eq!(z_score(90.0, 100.0, 5.0).unwrap(), -2.0);
    }

    #[test]
    fn ewma_fixture_matches_goal() {
        let variance = ewma_variance(&[0.0004, 0.0009], 0.94).unwrap();
        assert!((variance - 0.00043).abs() < 0.000_001);
    }

    #[test]
    fn observed_volume_sums_high_and_low_side_volume() {
        let row = interval_price_row(PriceInterval::OneHour, 250, 170, 100, 90, 0);
        assert_eq!(observed_volume(&row), 420);
    }

    #[test]
    fn helper_math_stays_deterministic() {
        assert!(log_return(105.0, 100.0).unwrap() > 0.0);
        assert_eq!(rolling_mean(&[1.0, 2.0, 3.0]).unwrap(), 2.0);
        assert!(rolling_std(&[1.0, 2.0, 3.0]).unwrap() > 0.0);
        assert_eq!(buy_limit_utilization(Some(50), Some(100)).unwrap(), 0.5);
    }
}
