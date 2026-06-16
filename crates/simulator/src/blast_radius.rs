use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Utc};
use grand_edge_domain::{GraphPath, GraphPathStep, ItemGraphEdge, ItemId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusConfig {
    pub alpha: f64,
    pub propagation_depth: usize,
    pub edge_confidence_threshold: f64,
    pub min_reported_impact_abs: f64,
    pub scenario_mode: BlastScenarioMode,
}

impl Default for BlastRadiusConfig {
    fn default() -> Self {
        Self {
            alpha: 0.65,
            propagation_depth: 3,
            edge_confidence_threshold: 0.55,
            min_reported_impact_abs: 0.005,
            scenario_mode: BlastScenarioMode::Balanced,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlastScenarioMode {
    Conservative,
    Balanced,
    Optimistic,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BlastScenarioMultipliers {
    pub edge_weight_multiplier: f64,
    pub uncertainty_multiplier: f64,
    pub min_confidence_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shock {
    pub item_id: i64,
    pub return_shock: f64,
    pub shock_type: ShockType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShockType {
    PriceUp,
    PriceDown,
    VolumeSpike,
    EventShock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusResult {
    pub run_id: Uuid,
    pub graph_version: String,
    pub source_shocks: Vec<Shock>,
    pub impacts: Vec<BlastRadiusImpact>,
    pub scenario_mode: BlastScenarioMode,
    pub config: BlastRadiusConfig,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusImpact {
    pub item_id: i64,
    pub expected_impact: f64,
    pub impact_low: Option<f64>,
    pub impact_high: Option<f64>,
    pub path: GraphPath,
    pub order: usize,
    pub confidence: f64,
    pub recommendation_change: Option<String>,
}

#[derive(Debug, Clone)]
struct PendingImpact {
    source_item_id: ItemId,
    current_item_id: ItemId,
    current_impact: f64,
    current_confidence: f64,
    order: usize,
    steps: Vec<GraphPathStep>,
}

#[derive(Debug, Clone)]
struct AccumulatedImpact {
    expected_impact: f64,
    path: GraphPath,
    confidence: f64,
    order: usize,
}

pub fn scenario_multipliers(mode: BlastScenarioMode) -> BlastScenarioMultipliers {
    match mode {
        BlastScenarioMode::Conservative => BlastScenarioMultipliers {
            edge_weight_multiplier: 0.55,
            uncertainty_multiplier: 1.60,
            min_confidence_multiplier: 1.10,
        },
        BlastScenarioMode::Balanced => BlastScenarioMultipliers {
            edge_weight_multiplier: 1.00,
            uncertainty_multiplier: 1.00,
            min_confidence_multiplier: 1.00,
        },
        BlastScenarioMode::Optimistic => BlastScenarioMultipliers {
            edge_weight_multiplier: 1.15,
            uncertainty_multiplier: 0.80,
            min_confidence_multiplier: 0.90,
        },
    }
}

pub fn propagate_shock(
    shocks: &[Shock],
    edges: &[ItemGraphEdge],
    config: &BlastRadiusConfig,
    regime_multiplier: impl Fn(&ItemGraphEdge) -> f64,
    liquidity_reliability: impl Fn(i64) -> f64,
) -> HashMap<i64, f64> {
    let state = propagate_internal(
        shocks,
        edges,
        config,
        regime_multiplier,
        liquidity_reliability,
    );
    state
        .into_iter()
        .map(|(item_id, accumulated)| (item_id, accumulated.expected_impact))
        .collect()
}

pub fn simulate_blast_radius(
    graph_version: &str,
    shocks: &[Shock],
    edges: &[ItemGraphEdge],
    config: BlastRadiusConfig,
    regime_multiplier: impl Fn(&ItemGraphEdge) -> f64,
    liquidity_reliability: impl Fn(i64) -> f64,
) -> BlastRadiusResult {
    let multipliers = scenario_multipliers(config.scenario_mode);
    let impacts = propagate_internal(
        shocks,
        edges,
        &config,
        regime_multiplier,
        liquidity_reliability,
    )
    .into_iter()
    .map(|(item_id, accumulated)| {
        let uncertainty =
            accumulated.expected_impact.abs() * 0.25 * multipliers.uncertainty_multiplier;
        BlastRadiusImpact {
            item_id,
            expected_impact: accumulated.expected_impact,
            impact_low: Some(accumulated.expected_impact - uncertainty),
            impact_high: Some(accumulated.expected_impact + uncertainty),
            path: accumulated.path,
            order: accumulated.order,
            confidence: accumulated.confidence,
            recommendation_change: None,
        }
    })
    .collect::<Vec<_>>();

    BlastRadiusResult {
        run_id: Uuid::new_v4(),
        graph_version: graph_version.to_string(),
        source_shocks: shocks.to_vec(),
        impacts,
        scenario_mode: config.scenario_mode,
        config,
        created_at: Utc::now(),
    }
}

fn propagate_internal(
    shocks: &[Shock],
    edges: &[ItemGraphEdge],
    config: &BlastRadiusConfig,
    regime_multiplier: impl Fn(&ItemGraphEdge) -> f64,
    liquidity_reliability: impl Fn(i64) -> f64,
) -> HashMap<i64, AccumulatedImpact> {
    let multipliers = scenario_multipliers(config.scenario_mode);
    let min_confidence =
        (config.edge_confidence_threshold * multipliers.min_confidence_multiplier).clamp(0.0, 1.0);
    let mut queue = VecDeque::new();
    let mut impacts = HashMap::<i64, AccumulatedImpact>::new();

    for shock in shocks {
        queue.push_back(PendingImpact {
            source_item_id: ItemId(shock.item_id),
            current_item_id: ItemId(shock.item_id),
            current_impact: shock.return_shock,
            current_confidence: 1.0,
            order: 0,
            steps: Vec::new(),
        });
    }

    while let Some(state) = queue.pop_front() {
        if state.order >= config.propagation_depth {
            continue;
        }

        for edge in edges.iter().filter(|edge| {
            edge.active
                && edge.from_item_id == state.current_item_id
                && edge.confidence >= min_confidence
        }) {
            let effective_weight = edge.weight
                * edge.confidence
                * edge.sign
                * regime_multiplier(edge)
                * liquidity_reliability(edge.to_item_id.0)
                * multipliers.edge_weight_multiplier;
            let next_impact = state.current_impact * config.alpha * effective_weight;
            if next_impact.abs() < config.min_reported_impact_abs {
                continue;
            }

            let next_order = state.order + 1;
            let depth_decay = 0.85_f64.powi(next_order as i32);
            let next_confidence =
                (state.current_confidence * edge.confidence * depth_decay).clamp(0.0, 1.0);
            let mut steps = state.steps.clone();
            steps.push(GraphPathStep {
                from_item_id: edge.from_item_id,
                to_item_id: edge.to_item_id,
                edge_id: edge.edge_id,
                edge_type: edge.edge_type,
                confidence: edge.confidence,
                weight: edge.weight,
            });

            let path = GraphPath {
                source_item_id: state.source_item_id,
                target_item_id: edge.to_item_id,
                steps: steps.clone(),
                path_confidence: next_confidence,
                expected_impact: Some(next_impact),
            };

            impacts
                .entry(edge.to_item_id.0)
                .and_modify(|current| {
                    current.expected_impact += next_impact;
                    if next_impact.abs() > current.path.expected_impact.unwrap_or_default().abs() {
                        current.path = path.clone();
                        current.confidence = next_confidence;
                        current.order = next_order;
                    }
                })
                .or_insert_with(|| AccumulatedImpact {
                    expected_impact: next_impact,
                    path: path.clone(),
                    confidence: next_confidence,
                    order: next_order,
                });

            queue.push_back(PendingImpact {
                source_item_id: state.source_item_id,
                current_item_id: edge.to_item_id,
                current_impact: next_impact,
                current_confidence: next_confidence,
                order: next_order,
                steps,
            });
        }
    }

    impacts
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        GraphEdgeDirection, GraphEdgeSourceType, GraphEdgeType, ItemGraphEdge, ItemId,
    };
    use uuid::Uuid;

    use super::{
        BlastRadiusConfig, BlastScenarioMode, Shock, ShockType, propagate_shock,
        scenario_multipliers, simulate_blast_radius,
    };

    #[test]
    fn single_edge_propagates_expected_impact() {
        let impacts = propagate_shock(
            &[Shock {
                item_id: 4151,
                return_shock: 0.10,
                shock_type: ShockType::PriceUp,
            }],
            &[edge(4151, 11840, 0.6, 0.8)],
            &BlastRadiusConfig::default(),
            |_| 1.0,
            |_| 1.0,
        );

        let expected = 0.10 * 0.65 * 0.6 * 0.8 * 1.0;
        assert!((impacts.get(&11840).copied().unwrap() - expected).abs() < 1e-9);
    }

    #[test]
    fn two_hop_path_confidence_decays() {
        let result = simulate_blast_radius(
            "graph_v1",
            &[Shock {
                item_id: 4151,
                return_shock: 0.10,
                shock_type: ShockType::PriceUp,
            }],
            &[edge(4151, 11840, 0.6, 0.9), edge(11840, 2366, 0.5, 0.8)],
            BlastRadiusConfig::default(),
            |_| 1.0,
            |_| 1.0,
        );

        let first_hop = result
            .impacts
            .iter()
            .find(|impact| impact.item_id == 11840)
            .unwrap();
        let second_hop = result
            .impacts
            .iter()
            .find(|impact| impact.item_id == 2366)
            .unwrap();
        assert!(second_hop.confidence < first_hop.confidence);
    }

    #[test]
    fn conservative_mode_reduces_impact() {
        let balanced = propagate_shock(
            &[Shock {
                item_id: 4151,
                return_shock: 0.10,
                shock_type: ShockType::PriceUp,
            }],
            &[edge(4151, 11840, 0.6, 0.8)],
            &BlastRadiusConfig::default(),
            |_| 1.0,
            |_| 1.0,
        );
        let conservative = propagate_shock(
            &[Shock {
                item_id: 4151,
                return_shock: 0.10,
                shock_type: ShockType::PriceUp,
            }],
            &[edge(4151, 11840, 0.6, 0.8)],
            &BlastRadiusConfig {
                scenario_mode: BlastScenarioMode::Conservative,
                ..BlastRadiusConfig::default()
            },
            |_| 1.0,
            |_| 1.0,
        );

        assert!(conservative.get(&11840).unwrap().abs() <= balanced.get(&11840).unwrap().abs());
    }

    #[test]
    fn edges_below_threshold_are_excluded() {
        let impacts = propagate_shock(
            &[Shock {
                item_id: 4151,
                return_shock: 0.10,
                shock_type: ShockType::PriceUp,
            }],
            &[edge(4151, 11840, 0.6, 0.50)],
            &BlastRadiusConfig::default(),
            |_| 1.0,
            |_| 1.0,
        );

        assert!(!impacts.contains_key(&11840));
    }

    #[test]
    fn blast_result_includes_graph_version() {
        let result = simulate_blast_radius(
            "graph_v1",
            &[Shock {
                item_id: 4151,
                return_shock: 0.10,
                shock_type: ShockType::PriceUp,
            }],
            &[edge(4151, 11840, 0.6, 0.8)],
            BlastRadiusConfig::default(),
            |_| 1.0,
            |_| 1.0,
        );

        assert_eq!(result.graph_version, "graph_v1");
        assert_eq!(result.scenario_mode, BlastScenarioMode::Balanced);
    }

    #[test]
    fn scenario_multipliers_are_stable() {
        assert_eq!(
            scenario_multipliers(BlastScenarioMode::Conservative).edge_weight_multiplier,
            0.55
        );
        assert_eq!(
            scenario_multipliers(BlastScenarioMode::Balanced).uncertainty_multiplier,
            1.0
        );
        assert_eq!(
            scenario_multipliers(BlastScenarioMode::Optimistic).min_confidence_multiplier,
            0.90
        );
    }

    #[test]
    fn impact_order_matches_path_length() {
        let result = simulate_blast_radius(
            "graph_v1",
            &[Shock {
                item_id: 4151,
                return_shock: 0.10,
                shock_type: ShockType::PriceUp,
            }],
            &[edge(4151, 11840, 0.6, 0.9), edge(11840, 2366, 0.5, 0.8)],
            BlastRadiusConfig::default(),
            |_| 1.0,
            |_| 1.0,
        );

        for impact in result.impacts {
            assert_eq!(impact.order, impact.path.steps.len());
            assert_eq!(impact.recommendation_change, None);
        }
    }

    fn edge(from_item_id: i64, to_item_id: i64, weight: f64, confidence: f64) -> ItemGraphEdge {
        ItemGraphEdge {
            edge_id: Uuid::new_v4(),
            graph_version: "graph_v1".to_string(),
            from_item_id: ItemId(from_item_id),
            to_item_id: ItemId(to_item_id),
            edge_type: GraphEdgeType::ShockTransmitsTo,
            direction: GraphEdgeDirection::Downstream,
            sign: 1.0,
            weight,
            lag_seconds: Some(300),
            confidence,
            source_type: GraphEdgeSourceType::Learned,
            source_ref: Some("fixture".to_string()),
            observations: Vec::new(),
            formula: serde_json::json!({"fixture": true}),
            requires_review: false,
            active: true,
            created_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }
}
