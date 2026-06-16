use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArBaselineConfig {
    pub intercept: f64,
    pub phi: f64,
    pub min_expected_return: f64,
    pub confidence_floor: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArForecast {
    pub forecast_delta: f64,
    pub forecast_price: f64,
    pub expected_return: f64,
}

pub fn forecast_next_price(
    current_price: f64,
    last_delta: f64,
    config: ArBaselineConfig,
) -> ArForecast {
    let forecast_delta = config.intercept + config.phi * last_delta;
    let forecast_price = current_price + forecast_delta;
    let expected_return = if current_price.abs() > f64::EPSILON {
        forecast_delta / current_price
    } else {
        0.0
    };

    ArForecast {
        forecast_delta,
        forecast_price,
        expected_return,
    }
}

#[cfg(test)]
mod tests {
    use super::{ArBaselineConfig, forecast_next_price};

    #[test]
    fn ar_forecast_matches_goal_fixture() {
        let forecast = forecast_next_price(
            103.0,
            2.0,
            ArBaselineConfig {
                intercept: 0.1,
                phi: 0.5,
                min_expected_return: 0.01,
                confidence_floor: 0.35,
            },
        );

        assert!((forecast.forecast_delta - 1.1).abs() < 1e-9);
        assert!((forecast.forecast_price - 104.1).abs() < 1e-9);
        assert!((forecast.expected_return - 0.010679611650485437).abs() < 1e-12);
    }
}
