use grand_edge_domain::EdgeObservation;

pub fn edge_stability_score(observations: &[EdgeObservation]) -> Option<f64> {
    if observations.is_empty() {
        return None;
    }

    let latest_sign = observations
        .iter()
        .find_map(|observation| observation.estimated_effect.map(signum_unit))?;

    let mut weighted_matches = 0.0;
    let mut total_weight = 0.0;

    for (index, observation) in observations.iter().enumerate() {
        let Some(effect) = observation.estimated_effect else {
            continue;
        };
        let recency_weight = 1.0 / (index as f64 + 1.0);
        let confidence_weight = observation.confidence.clamp(0.0, 1.0);
        let weight = recency_weight * confidence_weight;
        total_weight += weight;
        if signum_unit(effect) == latest_sign {
            weighted_matches += weight;
        }
    }

    (total_weight > 0.0).then_some((weighted_matches / total_weight).clamp(0.0, 1.0))
}

fn signum_unit(value: f64) -> i8 {
    if value > 0.0 {
        1
    } else if value < 0.0 {
        -1
    } else {
        0
    }
}
