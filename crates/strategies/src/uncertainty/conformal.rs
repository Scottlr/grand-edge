use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalInterval {
    pub lower_return: f64,
    pub predicted_return: f64,
    pub upper_return: f64,
    pub coverage: f64,
    pub residual_quantile: f64,
}

pub fn conformal_interval(
    predicted_return: f64,
    residual_quantile: f64,
    coverage: f64,
) -> ConformalInterval {
    let residual_quantile = residual_quantile.abs();
    let coverage = coverage.clamp(0.0, 1.0);
    ConformalInterval {
        lower_return: predicted_return - residual_quantile,
        predicted_return,
        upper_return: predicted_return + residual_quantile,
        coverage,
        residual_quantile,
    }
}

#[cfg(test)]
mod tests {
    use super::conformal_interval;

    #[test]
    fn conformal_interval_matches_goal_fixture() {
        let interval = conformal_interval(0.03, 0.018, 0.90);
        assert!((interval.lower_return - 0.012).abs() < 1e-12);
        assert!((interval.upper_return - 0.048).abs() < 1e-12);
    }
}
