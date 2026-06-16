use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KalmanState {
    pub fair_value: f64,
    pub variance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KalmanConfig {
    pub process_variance: f64,
    pub observation_variance: f64,
    pub buy_mispricing_threshold: f64,
    pub cashout_mispricing_threshold: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KalmanUpdate {
    pub kalman_gain: f64,
    pub posterior: KalmanState,
    pub mispricing: f64,
}

pub fn kalman_update(prior: KalmanState, observed_mid: f64, config: KalmanConfig) -> KalmanUpdate {
    let predicted_variance = (prior.variance + config.process_variance).max(0.0);
    let innovation_variance = predicted_variance + config.observation_variance.max(0.0);
    let kalman_gain = if innovation_variance > 0.0 {
        predicted_variance / innovation_variance
    } else {
        0.0
    };
    let posterior_fair_value = prior.fair_value + kalman_gain * (observed_mid - prior.fair_value);
    let posterior_variance = (1.0 - kalman_gain) * predicted_variance;
    let mispricing = if observed_mid.abs() > f64::EPSILON {
        (posterior_fair_value - observed_mid) / observed_mid
    } else {
        0.0
    };

    KalmanUpdate {
        kalman_gain,
        posterior: KalmanState {
            fair_value: posterior_fair_value,
            variance: posterior_variance.max(0.0),
        },
        mispricing,
    }
}

#[cfg(test)]
mod tests {
    use super::{KalmanConfig, KalmanState, kalman_update};

    #[test]
    fn kalman_update_matches_goal_fixture() {
        let update = kalman_update(
            KalmanState {
                fair_value: 100.0,
                variance: 4.0,
            },
            103.0,
            KalmanConfig {
                process_variance: 0.0,
                observation_variance: 1.0,
                buy_mispricing_threshold: 0.015,
                cashout_mispricing_threshold: -0.01,
            },
        );

        assert!((update.kalman_gain - 0.8).abs() < 1e-9);
        assert!((update.posterior.fair_value - 102.4).abs() < 1e-9);
        assert!((update.posterior.variance - 0.8).abs() < 1e-9);
    }
}
