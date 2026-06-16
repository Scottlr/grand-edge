use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ForecastMetrics {
    pub mae: Option<f64>,
    pub rmse: Option<f64>,
    pub directional_accuracy: Option<f64>,
    pub brier_score: Option<f64>,
}

pub fn mean_absolute_error(actual: &[f64], predicted: &[f64]) -> Option<f64> {
    paired_values(actual, predicted).map(|pairs| {
        pairs
            .iter()
            .map(|(actual, predicted)| (actual - predicted).abs())
            .sum::<f64>()
            / pairs.len() as f64
    })
}

pub fn root_mean_squared_error(actual: &[f64], predicted: &[f64]) -> Option<f64> {
    paired_values(actual, predicted).map(|pairs| {
        (pairs
            .iter()
            .map(|(actual, predicted)| {
                let error = actual - predicted;
                error * error
            })
            .sum::<f64>()
            / pairs.len() as f64)
            .sqrt()
    })
}

pub fn directional_accuracy(actual_returns: &[f64], predicted_returns: &[f64]) -> Option<f64> {
    paired_values(actual_returns, predicted_returns).map(|pairs| {
        pairs
            .iter()
            .filter(|(actual, predicted)| sign(*actual) == sign(*predicted))
            .count() as f64
            / pairs.len() as f64
    })
}

pub fn brier_score(outcomes: &[bool], probabilities: &[f64]) -> Option<f64> {
    if outcomes.is_empty() || outcomes.len() != probabilities.len() {
        return None;
    }

    Some(
        outcomes
            .iter()
            .zip(probabilities.iter())
            .map(|(outcome, probability)| {
                let observed = if *outcome { 1.0 } else { 0.0 };
                let diff = observed - probability;
                diff * diff
            })
            .sum::<f64>()
            / outcomes.len() as f64,
    )
}

fn paired_values<'a>(left: &'a [f64], right: &'a [f64]) -> Option<Vec<(f64, f64)>> {
    if left.is_empty() || left.len() != right.len() {
        return None;
    }

    Some(
        left.iter()
            .copied()
            .zip(right.iter().copied())
            .collect::<Vec<_>>(),
    )
}

fn sign(value: f64) -> i8 {
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::{brier_score, directional_accuracy, mean_absolute_error, root_mean_squared_error};

    #[test]
    fn mae_and_rmse_match_small_fixture() {
        let actual = [0.01, -0.02, 0.03];
        let predicted = [0.02, -0.01, 0.01];
        assert!((mean_absolute_error(&actual, &predicted).unwrap() - 0.013_333_333).abs() < 1e-9);
        assert!(
            (root_mean_squared_error(&actual, &predicted).unwrap() - 0.014_142_135).abs() < 1e-9
        );
    }

    #[test]
    fn directional_accuracy_counts_sign_matches() {
        let actual = [0.01, -0.02, 0.0, 0.03];
        let predicted = [0.02, -0.01, 0.1, -0.02];
        assert_eq!(directional_accuracy(&actual, &predicted), Some(0.5));
    }

    #[test]
    fn brier_score_matches_fixture() {
        let outcomes = [true, false, true];
        let probabilities = [0.8, 0.3, 0.6];
        assert!((brier_score(&outcomes, &probabilities).unwrap() - 0.096_666_666).abs() < 1e-9);
    }

    #[test]
    fn empty_samples_return_none_metrics() {
        assert_eq!(mean_absolute_error(&[], &[]), None);
        assert_eq!(root_mean_squared_error(&[], &[]), None);
        assert_eq!(directional_accuracy(&[], &[]), None);
        assert_eq!(brier_score(&[], &[]), None);
    }
}
