use chrono::{DateTime, Duration, Utc};
use grand_edge_domain::{
    EdgeObservation, EdgeObservationMethod, GraphEdgeDirection, GraphEdgeSourceType, GraphEdgeType,
    GraphVersion, ItemGraphEdge, ItemId,
};
use grand_edge_storage::Storage;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::AnalyticsError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedEdgeDiscoveryConfig {
    pub min_samples: usize,
    pub max_lag_buckets: usize,
    pub min_abs_correlation: f64,
    pub max_p_value: f64,
    pub min_out_of_sample_hit_rate: f64,
    pub default_confidence_cap: f64,
    pub cluster_size_limit: usize,
}

impl Default for LearnedEdgeDiscoveryConfig {
    fn default() -> Self {
        Self {
            min_samples: 120,
            max_lag_buckets: 12,
            min_abs_correlation: 0.35,
            max_p_value: 0.05,
            min_out_of_sample_hit_rate: 0.55,
            default_confidence_cap: 0.65,
            cluster_size_limit: 30,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearnedEdgeStatistic {
    pub lag: usize,
    pub statistic: f64,
    pub p_value: f64,
    pub estimated_effect: f64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedEdgeCandidate {
    pub from_item_id: i64,
    pub to_item_id: i64,
    pub edge_type: GraphEdgeType,
    pub method: EdgeObservationMethod,
    pub estimated_lag_seconds: Option<i64>,
    pub statistic: Option<f64>,
    pub p_value: Option<f64>,
    pub estimated_effect: Option<f64>,
    pub confidence: f64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedEdgePair {
    pub from_item_id: i64,
    pub to_item_id: i64,
    pub bucket_seconds: i64,
    pub source: Vec<f64>,
    pub target: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedEdgeDiscoveryRequest {
    pub graph_version: GraphVersion,
    pub observed_at: DateTime<Utc>,
    pub config: LearnedEdgeDiscoveryConfig,
    pub pairs: Vec<LearnedEdgePair>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedEdgeDiscoveryReport {
    pub graph_version: String,
    pub candidates: Vec<LearnedEdgeCandidate>,
    pub persisted_edges: u64,
    pub persisted_observations: u64,
    pub note: String,
}

pub fn rolling_correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() != y.len() || x.len() < 2 {
        return None;
    }

    let mean_x = x.iter().sum::<f64>() / x.len() as f64;
    let mean_y = y.iter().sum::<f64>() / y.len() as f64;

    let mut numerator = 0.0;
    let mut denom_x = 0.0;
    let mut denom_y = 0.0;
    for (lhs, rhs) in x.iter().zip(y.iter()) {
        let dx = lhs - mean_x;
        let dy = rhs - mean_y;
        numerator += dx * dy;
        denom_x += dx * dx;
        denom_y += dy * dy;
    }

    let denominator = (denom_x * denom_y).sqrt();
    (denominator > 0.0).then_some((numerator / denominator).clamp(-1.0, 1.0))
}

pub fn lead_lag_regression_score(
    source: &[f64],
    target: &[f64],
    lag: usize,
) -> Option<LearnedEdgeStatistic> {
    if source.len() != target.len() || source.len() <= lag + 1 {
        return None;
    }

    let shifted_source = &source[..source.len() - lag];
    let shifted_target = &target[lag..];
    let statistic = rolling_correlation(shifted_source, shifted_target)?;
    let sample_size = shifted_source.len() as f64;
    let p_value = (1.0 - statistic.abs()).powi(2) / sample_size.sqrt().max(1.0);

    Some(LearnedEdgeStatistic {
        lag,
        statistic,
        p_value,
        estimated_effect: statistic,
        summary: format!("predictive lead-lag evidence at lag {lag}"),
    })
}

pub fn granger_style_predictive_test(
    source: &[f64],
    target: &[f64],
    max_lag: usize,
) -> Option<LearnedEdgeStatistic> {
    let mut best: Option<LearnedEdgeStatistic> = None;
    for lag in 1..=max_lag {
        let Some(candidate) = lead_lag_regression_score(source, target, lag) else {
            continue;
        };
        let replace = best
            .as_ref()
            .is_none_or(|current| candidate.statistic.abs() > current.statistic.abs());
        if replace {
            best = Some(LearnedEdgeStatistic {
                summary: format!(
                    "predictive lead-lag evidence suggests source may help forecast target at lag {}",
                    candidate.lag
                ),
                ..candidate
            });
        }
    }
    best
}

pub fn discover_learned_edges(
    request: LearnedEdgeDiscoveryRequest,
) -> Result<LearnedEdgeDiscoveryReport, AnalyticsError> {
    let mut candidates = Vec::new();
    for pair in &request.pairs {
        if pair.source.len() < request.config.min_samples
            || pair.target.len() < request.config.min_samples
        {
            continue;
        }

        let Some(correlation) = rolling_correlation(&pair.source, &pair.target) else {
            continue;
        };
        if correlation.abs() < request.config.min_abs_correlation {
            continue;
        }

        let Some(statistic) = granger_style_predictive_test(
            &pair.source,
            &pair.target,
            request.config.max_lag_buckets,
        ) else {
            continue;
        };
        if statistic.p_value > request.config.max_p_value {
            continue;
        }

        let confidence = confidence_without_oos(
            request.config.default_confidence_cap,
            statistic.statistic.abs(),
        );
        candidates.push(LearnedEdgeCandidate {
            from_item_id: pair.from_item_id,
            to_item_id: pair.to_item_id,
            edge_type: if statistic.lag > 0 {
                GraphEdgeType::Leads
            } else {
                GraphEdgeType::CorrelatedWith
            },
            method: EdgeObservationMethod::GrangerStyle,
            estimated_lag_seconds: Some(pair.bucket_seconds * statistic.lag as i64),
            statistic: Some(statistic.statistic),
            p_value: Some(statistic.p_value),
            estimated_effect: Some(statistic.estimated_effect),
            confidence,
            metadata: serde_json::json!({
                "summary": statistic.summary,
                "sample_size": pair.source.len(),
                "predictive_evidence_only": true,
            }),
        });
    }

    Ok(LearnedEdgeDiscoveryReport {
        graph_version: request.graph_version.graph_version,
        persisted_edges: 0,
        persisted_observations: 0,
        note: if request.dry_run {
            "dry run only; candidates were not persisted".to_string()
        } else {
            "candidate generation complete".to_string()
        },
        candidates,
    })
}

pub async fn persist_learned_edge_candidates(
    storage: &Storage,
    graph_version: &GraphVersion,
    observed_at: DateTime<Utc>,
    candidates: &[LearnedEdgeCandidate],
) -> Result<(u64, u64), AnalyticsError> {
    storage.graph().insert_graph_version(graph_version).await?;

    let edges = candidates
        .iter()
        .map(|candidate| candidate_to_edge(graph_version, observed_at, candidate))
        .collect::<Vec<_>>();
    let observations = candidates
        .iter()
        .zip(edges.iter())
        .map(|(candidate, edge)| candidate_to_observation(observed_at, candidate, edge.edge_id))
        .collect::<Vec<_>>();

    let inserted_edges = storage.graph().upsert_edges(&edges).await?;
    let inserted_observations = storage
        .graph()
        .insert_edge_observations(&observations)
        .await?;
    Ok((inserted_edges, inserted_observations))
}

pub fn discover_fixture_edges(dry_run: bool) -> Result<LearnedEdgeDiscoveryReport, AnalyticsError> {
    let observed_at = Utc::now();
    let request = LearnedEdgeDiscoveryRequest {
        graph_version: GraphVersion {
            graph_version: "learned_fixture_v1".to_string(),
            source_hash: "fixture".to_string(),
            created_at: observed_at,
            description: "fixture learned-edge discovery run".to_string(),
        },
        observed_at,
        config: LearnedEdgeDiscoveryConfig {
            min_samples: 6,
            max_lag_buckets: 3,
            min_abs_correlation: 0.35,
            max_p_value: 0.05,
            min_out_of_sample_hit_rate: 0.55,
            default_confidence_cap: 0.65,
            cluster_size_limit: 4,
        },
        pairs: vec![fixture_pair()],
        dry_run,
    };
    discover_learned_edges(request)
}

fn candidate_to_edge(
    graph_version: &GraphVersion,
    observed_at: DateTime<Utc>,
    candidate: &LearnedEdgeCandidate,
) -> ItemGraphEdge {
    ItemGraphEdge {
        edge_id: stable_edge_id(graph_version, candidate),
        graph_version: graph_version.graph_version.clone(),
        from_item_id: ItemId(candidate.from_item_id),
        to_item_id: ItemId(candidate.to_item_id),
        edge_type: candidate.edge_type,
        direction: GraphEdgeDirection::Downstream,
        sign: candidate.estimated_effect.unwrap_or(0.0).signum(),
        weight: candidate
            .estimated_effect
            .unwrap_or(candidate.statistic.unwrap_or(0.0))
            .abs()
            .clamp(0.0, 1.0),
        lag_seconds: candidate.estimated_lag_seconds,
        confidence: candidate.confidence,
        source_type: GraphEdgeSourceType::Learned,
        source_ref: Some("analytics.learned_edges".to_string()),
        observations: Vec::new(),
        formula: serde_json::json!({
            "method": candidate.method,
            "predictive_evidence_only": true,
        }),
        requires_review: false,
        active: candidate.confidence >= 0.5,
        created_at: observed_at,
        updated_at: observed_at,
    }
}

fn candidate_to_observation(
    observed_at: DateTime<Utc>,
    candidate: &LearnedEdgeCandidate,
    edge_id: Uuid,
) -> EdgeObservation {
    EdgeObservation {
        edge_id,
        observed_at,
        method: candidate.method,
        window_start: observed_at - Duration::days(30),
        window_end: observed_at,
        statistic: candidate.statistic,
        p_value: candidate.p_value,
        estimated_lag_seconds: candidate.estimated_lag_seconds,
        estimated_effect: candidate.estimated_effect,
        confidence: candidate.confidence,
        metadata: candidate.metadata.clone(),
    }
}

fn stable_edge_id(graph_version: &GraphVersion, candidate: &LearnedEdgeCandidate) -> Uuid {
    let digest = Sha256::digest(
        format!(
            "{}:{}:{}:{:?}",
            graph_version.graph_version,
            candidate.from_item_id,
            candidate.to_item_id,
            candidate.method
        )
        .as_bytes(),
    );
    Uuid::from_slice(&digest[..16]).expect("sha256 digest chunk is a valid UUID byte slice")
}

fn confidence_without_oos(cap: f64, signal_strength: f64) -> f64 {
    (signal_strength * 0.9).clamp(0.0, cap)
}

fn fixture_pair() -> LearnedEdgePair {
    LearnedEdgePair {
        from_item_id: 4151,
        to_item_id: 11840,
        bucket_seconds: 300,
        source: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        target: vec![0.0, 0.9, 2.1, 2.9, 4.0, 5.2, 6.1, 7.1],
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};
    use grand_edge_domain::{EdgeObservation, EdgeObservationMethod, GraphVersion};

    use super::{
        LearnedEdgeDiscoveryConfig, LearnedEdgeDiscoveryRequest, confidence_without_oos,
        discover_learned_edges, granger_style_predictive_test, lead_lag_regression_score,
        rolling_correlation,
    };
    use crate::edge_stability::edge_stability_score;

    #[test]
    fn rolling_correlation_matches_fixture() {
        let score = rolling_correlation(&[1.0, 2.0, 3.0, 4.0], &[2.0, 4.0, 6.0, 8.0]).unwrap();
        assert!((score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn lead_lag_regression_identifies_known_lag() {
        let source = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let target = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let statistic = lead_lag_regression_score(&source, &target, 1).unwrap();
        assert!(statistic.statistic > 0.99);
    }

    #[test]
    fn granger_style_output_does_not_claim_causality() {
        let source = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let target = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let statistic = granger_style_predictive_test(&source, &target, 2).unwrap();
        assert!(statistic.summary.contains("predictive"));
        assert!(!statistic.summary.to_lowercase().contains("causal"));
    }

    #[test]
    fn edge_stability_drops_on_conflicting_observations() {
        let aligned = vec![
            fixture_observation(0.5, 0.8, 0),
            fixture_observation(0.4, 0.7, 1),
        ];
        let conflicting = vec![
            fixture_observation(0.5, 0.8, 0),
            fixture_observation(-0.4, 0.7, 1),
        ];

        assert!(
            edge_stability_score(&aligned).unwrap() > edge_stability_score(&conflicting).unwrap()
        );
    }

    #[test]
    fn learned_edge_confidence_is_capped_without_oos_results() {
        let confidence = confidence_without_oos(0.65, 0.98);
        assert!(confidence <= 0.65);
    }

    #[test]
    fn discover_learned_edges_builds_candidates_from_fixture_pairs() {
        let report = discover_learned_edges(LearnedEdgeDiscoveryRequest {
            graph_version: GraphVersion {
                graph_version: "graph_v1".to_string(),
                source_hash: "fixture".to_string(),
                created_at: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
                description: "fixture".to_string(),
            },
            observed_at: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            config: LearnedEdgeDiscoveryConfig {
                min_samples: 6,
                ..LearnedEdgeDiscoveryConfig::default()
            },
            pairs: vec![super::fixture_pair()],
            dry_run: true,
        })
        .unwrap();

        assert_eq!(report.candidates.len(), 1);
        assert!(report.note.contains("dry run"));
    }

    fn fixture_observation(effect: f64, confidence: f64, days_back: i64) -> EdgeObservation {
        let observed_at =
            Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap() - Duration::days(days_back);
        EdgeObservation {
            edge_id: uuid::Uuid::new_v4(),
            observed_at,
            method: EdgeObservationMethod::OutcomeBacktest,
            window_start: Utc.with_ymd_and_hms(2026, 5, 16, 0, 0, 0).unwrap(),
            window_end: Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap(),
            statistic: Some(effect),
            p_value: Some(0.02),
            estimated_lag_seconds: Some(300),
            estimated_effect: Some(effect),
            confidence,
            metadata: serde_json::json!({}),
        }
    }
}
