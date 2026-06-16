use grand_edge_domain::{
    GraphPath, GraphPathStep, GraphRecommendationAction, ItemGraphEdge, ItemId, ReasonAtom,
    ReasonDirection, ReasonType, UserPosition,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::RecommendationScore;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphActionConfig {
    pub min_path_confidence: f64,
    pub min_conversion_gap_pct: f64,
    pub min_second_order_expected_return: f64,
    pub max_blast_radius_risk: f64,
}

impl Default for GraphActionConfig {
    fn default() -> Self {
        Self {
            min_path_confidence: 0.60,
            min_conversion_gap_pct: 0.01,
            min_second_order_expected_return: 0.005,
            max_blast_radius_risk: 0.03,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphRecommendationInput {
    pub item_id: ItemId,
    pub graph_version: String,
    pub graph_paths: Vec<GraphPath>,
    pub graph_edges: Vec<ItemGraphEdge>,
    pub graph_adjusted_expected_return: Option<f64>,
    pub conversion_gap_pct: Option<f64>,
    pub link_disagreement: Option<f64>,
    pub blast_radius_risk: Option<f64>,
    pub execution_confidence: Option<f64>,
    pub historical_path_performance: Option<Value>,
    pub existing_position: Option<UserPosition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphActionDecision {
    pub action: GraphRecommendationAction,
    pub path_count: usize,
    pub edge_confidence: Option<f64>,
}

pub fn map_graph_action(
    input: &GraphRecommendationInput,
    score: &RecommendationScore,
    config: &GraphActionConfig,
) -> Option<GraphActionDecision> {
    let strongest_confidence = strongest_path_confidence(&input.graph_paths);
    let positive_graph_edge = input
        .graph_adjusted_expected_return
        .is_some_and(|value| value >= config.min_second_order_expected_return);
    let weak_execution = input.execution_confidence.is_some_and(|value| value < 0.45)
        || strongest_confidence.is_some_and(|value| value < config.min_path_confidence);

    if input.existing_position.is_some()
        && input
            .blast_radius_risk
            .is_some_and(|value| value <= -config.max_blast_radius_risk)
    {
        return Some(GraphActionDecision {
            action: GraphRecommendationAction::CashoutBeforeContagion,
            path_count: input.graph_paths.len(),
            edge_confidence: strongest_confidence,
        });
    }

    if input
        .blast_radius_risk
        .is_some_and(|value| value >= config.max_blast_radius_risk)
    {
        return Some(GraphActionDecision {
            action: GraphRecommendationAction::AvoidBlastRadius,
            path_count: input.graph_paths.len(),
            edge_confidence: strongest_confidence,
        });
    }

    if input
        .conversion_gap_pct
        .is_some_and(|value| value >= config.min_conversion_gap_pct)
        && input.execution_confidence.unwrap_or(0.0) >= 0.50
    {
        return Some(GraphActionDecision {
            action: GraphRecommendationAction::ExploitConversion,
            path_count: input.graph_paths.len(),
            edge_confidence: strongest_confidence,
        });
    }

    if positive_graph_edge && weak_execution {
        return Some(GraphActionDecision {
            action: GraphRecommendationAction::WatchSecondOrder,
            path_count: input.graph_paths.len(),
            edge_confidence: strongest_confidence,
        });
    }

    if positive_graph_edge
        && strongest_confidence.is_some_and(|value| value >= config.min_path_confidence)
    {
        let action = if input
            .graph_edges
            .iter()
            .any(|edge| edge.from_item_id == input.item_id && edge.to_item_id == input.item_id)
        {
            GraphRecommendationAction::PairTrade
        } else if input.graph_edges.iter().any(|edge| {
            matches!(
                edge.edge_type,
                grand_edge_domain::GraphEdgeType::Substitute
                    | grand_edge_domain::GraphEdgeType::SameCategory
            )
        }) {
            GraphRecommendationAction::Rotate
        } else if score.risk_penalty >= 0.10 {
            GraphRecommendationAction::Hedge
        } else {
            GraphRecommendationAction::BuyLinked
        };
        return Some(GraphActionDecision {
            action,
            path_count: input.graph_paths.len(),
            edge_confidence: strongest_confidence,
        });
    }

    None
}

pub fn build_graph_reason_atoms(
    input: &GraphRecommendationInput,
    decision: Option<&GraphActionDecision>,
) -> Vec<ReasonAtom> {
    let strongest_confidence = strongest_path_confidence(&input.graph_paths).unwrap_or(0.0);
    let mut atoms = Vec::new();

    if let Some(gap) = input.conversion_gap_pct.filter(|value| *value > 0.0) {
        atoms.push(graph_atom(
            "conversion_gap_positive_after_tax",
            "Conversion remains profitable after tax",
            ReasonDirection::Positive,
            gap.abs(),
            input,
            serde_json::json!({
                "conversionGapPct": gap,
                "graphAction": decision.map(|value| value.action),
            }),
        ));
    }

    if input
        .graph_adjusted_expected_return
        .is_some_and(|value| value > 0.0)
    {
        atoms.push(graph_atom(
            "linked_input_shock_not_priced_in",
            "Linked items moved faster than this item",
            ReasonDirection::Positive,
            input
                .graph_adjusted_expected_return
                .unwrap_or_default()
                .abs(),
            input,
            serde_json::json!({
                "graphAdjustedExpectedReturn": input.graph_adjusted_expected_return,
                "linkDisagreement": input.link_disagreement,
                "graphAction": decision.map(|value| value.action),
            }),
        ));
    }

    if input.link_disagreement.is_some_and(|value| value >= 0.03) {
        atoms.push(graph_atom(
            "substitute_rotation_candidate",
            "Neighbor repricing suggests a rotation setup",
            ReasonDirection::Positive,
            input.link_disagreement.unwrap_or_default(),
            input,
            serde_json::json!({
                "linkDisagreement": input.link_disagreement,
                "pathCount": input.graph_paths.len(),
            }),
        ));
    }

    if input
        .blast_radius_risk
        .is_some_and(|value| value >= 0.03 || value <= -0.03)
    {
        let risk = input.blast_radius_risk.unwrap_or_default();
        let direction = if risk < 0.0 {
            ReasonDirection::Negative
        } else {
            ReasonDirection::Negative
        };
        atoms.push(graph_atom(
            "portfolio_contagion_risk",
            "Graph paths imply contagion risk across linked items",
            direction,
            risk.abs(),
            input,
            serde_json::json!({
                "blastRadiusRisk": risk,
                "existingPosition": input.existing_position.is_some(),
                "graphAction": decision.map(|value| value.action),
            }),
        ));
    }

    if strongest_confidence < 0.60 && !input.graph_paths.is_empty() {
        atoms.push(graph_atom(
            "learned_edge_low_confidence",
            "Relationship evidence is present but path confidence is still weak",
            ReasonDirection::Negative,
            1.0 - strongest_confidence,
            input,
            serde_json::json!({
                "pathConfidence": strongest_confidence,
                "graphAction": decision.map(|value| value.action),
            }),
        ));
    }

    if input.graph_paths.is_empty() && !input.graph_edges.is_empty() {
        atoms.push(graph_atom(
            "graph_relationship_stale_or_broken",
            "Graph edges exist but no actionable path could be reconstructed",
            ReasonDirection::Negative,
            0.25,
            input,
            serde_json::json!({
                "edgeCount": input.graph_edges.len(),
                "pathCount": input.graph_paths.len(),
            }),
        ));
    }

    atoms
}

pub fn build_graph_paths(item_id: ItemId, edges: &[ItemGraphEdge]) -> Vec<GraphPath> {
    edges
        .iter()
        .map(|edge| GraphPath {
            source_item_id: edge.from_item_id,
            target_item_id: edge.to_item_id,
            steps: vec![GraphPathStep {
                from_item_id: edge.from_item_id,
                to_item_id: edge.to_item_id,
                edge_id: edge.edge_id,
                edge_type: edge.edge_type,
                confidence: edge.confidence,
                weight: edge.weight,
            }],
            path_confidence: edge.confidence,
            expected_impact: Some(edge.sign * edge.weight),
        })
        .filter(|path| path.source_item_id == item_id || path.target_item_id == item_id)
        .collect()
}

fn strongest_path_confidence(paths: &[GraphPath]) -> Option<f64> {
    paths
        .iter()
        .map(|path| path.path_confidence)
        .max_by(f64::total_cmp)
}

fn graph_atom(
    reason_key: &str,
    label: &str,
    direction: ReasonDirection,
    weight: f64,
    input: &GraphRecommendationInput,
    extra_evidence: Value,
) -> ReasonAtom {
    let evidence = serde_json::json!({
        "graphVersion": input.graph_version,
        "pathConfidence": strongest_path_confidence(&input.graph_paths),
        "historicalPathPerformance": input.historical_path_performance,
        "paths": input.graph_paths,
        "extra": extra_evidence,
    });
    ReasonAtom {
        reason_type: ReasonType::GraphRelationship,
        reason_key: format!("graph:{reason_key}"),
        label: label.to_string(),
        direction,
        weight,
        evidence,
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        Gp, GraphEdgeDirection, GraphEdgeSourceType, GraphEdgeType, ItemGraphEdge, ItemId,
        PositionId, UserId, UserPosition,
    };
    use serde_json::Value;
    use uuid::Uuid;

    use crate::RecommendationScore;

    use super::{
        GraphActionConfig, GraphRecommendationInput, build_graph_paths, build_graph_reason_atoms,
        map_graph_action,
    };

    #[test]
    fn exploit_conversion_requires_after_tax_gap() {
        let score = score(0.08, Some(0.7));
        let mut input = input();
        input.conversion_gap_pct = Some(0.02);
        let decision = map_graph_action(&input, &score, &GraphActionConfig::default()).unwrap();
        assert_eq!(
            decision.action,
            grand_edge_domain::GraphRecommendationAction::ExploitConversion
        );

        input.conversion_gap_pct = Some(0.0);
        assert_ne!(
            map_graph_action(&input, &score, &GraphActionConfig::default())
                .map(|value| value.action),
            Some(grand_edge_domain::GraphRecommendationAction::ExploitConversion)
        );
    }

    #[test]
    fn watch_second_order_when_execution_confidence_weak() {
        let score = score(0.08, Some(0.2));
        let mut input = input();
        input.graph_adjusted_expected_return = Some(0.02);
        input.execution_confidence = Some(0.2);
        let decision = map_graph_action(&input, &score, &GraphActionConfig::default()).unwrap();
        assert_eq!(
            decision.action,
            grand_edge_domain::GraphRecommendationAction::WatchSecondOrder
        );
    }

    #[test]
    fn cashout_before_contagion_uses_position_and_blast_risk() {
        let score = score(0.04, Some(0.8));
        let mut input = input();
        input.existing_position = Some(UserPosition {
            position_id: PositionId(Uuid::new_v4()),
            user_id: UserId(Uuid::new_v4()),
            item_id: ItemId(4151),
            quantity: grand_edge_domain::Quantity(2),
            avg_buy_price: Gp(100_000),
            bought_at: None,
            notes: None,
        });
        input.blast_radius_risk = Some(-0.05);
        let decision = map_graph_action(&input, &score, &GraphActionConfig::default()).unwrap();
        assert_eq!(
            decision.action,
            grand_edge_domain::GraphRecommendationAction::CashoutBeforeContagion
        );
    }

    #[test]
    fn graph_reason_atom_contains_path_evidence() {
        let input = input();
        let atoms = build_graph_reason_atoms(&input, None);
        let atom = atoms
            .iter()
            .find(|value| value.reason_key == "graph:linked_input_shock_not_priced_in")
            .unwrap();
        assert_eq!(
            atom.evidence.get("graphVersion").and_then(Value::as_str),
            Some("graph_v1")
        );
        assert!(atom.evidence.get("pathConfidence").is_some());
        assert!(atom.evidence.get("paths").is_some());
    }

    fn input() -> GraphRecommendationInput {
        let edge = ItemGraphEdge {
            edge_id: Uuid::new_v4(),
            graph_version: "graph_v1".to_string(),
            from_item_id: ItemId(11840),
            to_item_id: ItemId(4151),
            edge_type: GraphEdgeType::IngredientOf,
            direction: GraphEdgeDirection::Upstream,
            sign: 1.0,
            weight: 0.8,
            lag_seconds: Some(300),
            confidence: 0.9,
            source_type: GraphEdgeSourceType::Mechanical,
            source_ref: Some("fixture".to_string()),
            observations: Vec::new(),
            formula: serde_json::json!({}),
            requires_review: false,
            active: true,
            created_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        };
        let paths = build_graph_paths(ItemId(4151), std::slice::from_ref(&edge));
        GraphRecommendationInput {
            item_id: ItemId(4151),
            graph_version: "graph_v1".to_string(),
            graph_paths: paths,
            graph_edges: vec![edge],
            graph_adjusted_expected_return: Some(0.015),
            conversion_gap_pct: None,
            link_disagreement: Some(0.05),
            blast_radius_risk: None,
            execution_confidence: Some(0.7),
            historical_path_performance: Some(serde_json::json!({
                "historicalCatchupRate": 0.58,
                "medianCatchupLagSeconds": 7200
            })),
            existing_position: None,
        }
    }

    fn score(final_score: f64, execution_confidence: Option<f64>) -> RecommendationScore {
        RecommendationScore {
            raw_edge: 0.05,
            liquidity_adjusted_edge: 0.04,
            graph_adjusted_edge: Some(0.06),
            prediction_confidence: Some(0.7),
            execution_confidence,
            recommendation_confidence: 0.6,
            risk_penalty: 0.05,
            liquidity_penalty: 0.01,
            model_confidence_bonus: 0.0,
            user_fit_bonus: 0.0,
            final_score,
            components: Vec::new(),
        }
    }
}
