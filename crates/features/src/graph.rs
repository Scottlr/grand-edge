use grand_edge_domain::{Gp, GraphEdgeType};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::snapshot::{GraphFeatureContext, NeighborPriceHistory};

pub const GRAPH_FEATURE_SET_VERSION: &str = "graph_features_v1";

pub const GRAPH_FEATURE_KEYS: &[&str] = &[
    "graph_version",
    "upstream_pressure_1h",
    "downstream_pressure_1h",
    "sector_momentum_6h",
    "neighbor_momentum_6h",
    "relative_value_residual",
    "conversion_gap_pct",
    "item_set_pack_gap_gp",
    "item_set_unpack_gap_gp",
    "basket_residual",
    "graph_adjusted_momentum_6h",
    "graph_adjusted_volatility_24h",
    "link_disagreement_6h",
    "strongest_graph_path_confidence",
    "graph_neighbor_count",
    "graph_missing_neighbor_data_count",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphFeatureConfig {
    pub max_graph_depth: usize,
    pub min_edge_confidence: f64,
    pub upstream_lambda: f64,
    pub downstream_lambda: f64,
    pub sector_lambda: f64,
    pub stale_after_secs: i64,
}

impl Default for GraphFeatureConfig {
    fn default() -> Self {
        Self {
            max_graph_depth: 2,
            min_edge_confidence: 0.55,
            upstream_lambda: 0.35,
            downstream_lambda: 0.25,
            sector_lambda: 0.15,
            stale_after_secs: 900,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedNeighborReturn {
    pub item_id: i64,
    pub return_value: f64,
    pub edge_weight: f64,
    pub edge_confidence: f64,
    pub lag_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedNeighborPrice {
    pub item_id: i64,
    pub log_price: f64,
    pub edge_weight: f64,
    pub edge_confidence: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphFeatureSnapshot {
    pub graph_version: String,
    pub values: Map<String, Value>,
}

pub fn upstream_pressure(neighbor_returns: &[WeightedNeighborReturn]) -> Option<f64> {
    weighted_average_return(neighbor_returns)
}

pub fn downstream_pressure(neighbor_returns: &[WeightedNeighborReturn]) -> Option<f64> {
    weighted_average_return(neighbor_returns)
}

pub fn relative_value_residual(
    own_log_price: f64,
    neighbor_log_prices: &[WeightedNeighborPrice],
) -> Option<f64> {
    let weighted_neighbor_price = weighted_average_price(neighbor_log_prices)?;
    Some(own_log_price - weighted_neighbor_price)
}

pub fn graph_adjusted_momentum(
    own_return: f64,
    upstream: f64,
    downstream: f64,
    sector: f64,
    config: &GraphFeatureConfig,
) -> f64 {
    own_return
        + upstream * config.upstream_lambda
        + downstream * config.downstream_lambda
        + sector * config.sector_lambda
}

pub fn link_disagreement(own_return: f64, expected_neighbor_return: f64) -> f64 {
    (own_return - expected_neighbor_return).abs()
}

pub fn conversion_gap_pct(
    input_cost: f64,
    output_sell_after_tax: f64,
    conversion_cost: f64,
) -> Option<f64> {
    let denominator = input_cost + conversion_cost;
    if denominator <= f64::EPSILON {
        return None;
    }

    Some((output_sell_after_tax - denominator) / denominator)
}

pub fn build_graph_feature_snapshot(
    own_return_6h: Option<f64>,
    own_volatility_24h: Option<f64>,
    own_mid: Option<f64>,
    context: &GraphFeatureContext,
    config: &GraphFeatureConfig,
) -> GraphFeatureSnapshot {
    let incoming_returns = collect_weighted_returns(&context.incoming_neighbors, config);
    let outgoing_returns = collect_weighted_returns(&context.outgoing_neighbors, config);
    let sector_returns = collect_weighted_returns(&context.sector_neighbors, config);
    let upstream = upstream_pressure(&incoming_returns);
    let downstream = downstream_pressure(&outgoing_returns);
    let sector = weighted_average_return(&sector_returns);

    let all_neighbor_returns = incoming_returns
        .iter()
        .chain(outgoing_returns.iter())
        .chain(sector_returns.iter())
        .cloned()
        .collect::<Vec<_>>();
    let neighbor_momentum = weighted_average_return(&all_neighbor_returns);

    let incoming_prices = collect_weighted_prices(&context.incoming_neighbors, config);
    let outgoing_prices = collect_weighted_prices(&context.outgoing_neighbors, config);
    let sector_prices = collect_weighted_prices(&context.sector_neighbors, config);
    let all_neighbor_prices = incoming_prices
        .iter()
        .chain(outgoing_prices.iter())
        .chain(sector_prices.iter())
        .cloned()
        .collect::<Vec<_>>();
    let own_log_price = own_mid.filter(|value| *value > 0.0).map(f64::ln);
    let relative_residual =
        own_log_price.and_then(|value| relative_value_residual(value, &all_neighbor_prices));
    let basket_residual =
        own_log_price.and_then(|value| relative_value_residual(value, &sector_prices));

    let strongest_confidence = context
        .incoming_neighbors
        .iter()
        .chain(context.outgoing_neighbors.iter())
        .chain(context.sector_neighbors.iter())
        .map(|neighbor| neighbor.edge.confidence)
        .max_by(f64::total_cmp);

    let missing_neighbor_data_count = context
        .incoming_neighbors
        .iter()
        .chain(context.outgoing_neighbors.iter())
        .chain(context.sector_neighbors.iter())
        .filter(|neighbor| neighbor_return(neighbor).is_none() || latest_mid(neighbor).is_none())
        .count();
    let graph_neighbor_count = context.incoming_neighbors.len()
        + context.outgoing_neighbors.len()
        + context.sector_neighbors.len();

    let expected_neighbor_return = neighbor_momentum.unwrap_or(0.0);
    let link_disagreement_6h =
        own_return_6h.map(|own_return| link_disagreement(own_return, expected_neighbor_return));
    let graph_adjusted_momentum_6h = own_return_6h.map(|own_return| {
        graph_adjusted_momentum(
            own_return,
            upstream.unwrap_or(0.0),
            downstream.unwrap_or(0.0),
            sector.unwrap_or(0.0),
            config,
        )
    });

    let graph_adjusted_volatility_24h = own_volatility_24h.map(|own_volatility| {
        let neighbor_vols = neighbor_volatilities(&context.incoming_neighbors, config)
            .into_iter()
            .chain(neighbor_volatilities(&context.outgoing_neighbors, config))
            .chain(neighbor_volatilities(&context.sector_neighbors, config))
            .collect::<Vec<_>>();
        if neighbor_vols.is_empty() {
            own_volatility
        } else {
            (own_volatility + neighbor_vols.iter().sum::<f64>() / neighbor_vols.len() as f64) / 2.0
        }
    });

    let conversion_metrics =
        conversion_metrics(&context.incoming_neighbors, &context.outgoing_neighbors);

    let mut values = Map::new();
    values.insert(
        "graph_version".to_string(),
        Value::String(context.graph_version.clone()),
    );
    insert_option_f64(&mut values, "upstream_pressure_1h", upstream);
    insert_option_f64(&mut values, "downstream_pressure_1h", downstream);
    insert_option_f64(&mut values, "sector_momentum_6h", sector);
    insert_option_f64(&mut values, "neighbor_momentum_6h", neighbor_momentum);
    insert_option_f64(&mut values, "relative_value_residual", relative_residual);
    insert_option_f64(
        &mut values,
        "conversion_gap_pct",
        conversion_metrics.conversion_gap_pct,
    );
    insert_option_f64(
        &mut values,
        "item_set_pack_gap_gp",
        conversion_metrics.item_set_pack_gap_gp,
    );
    insert_option_f64(
        &mut values,
        "item_set_unpack_gap_gp",
        conversion_metrics.item_set_unpack_gap_gp,
    );
    insert_option_f64(&mut values, "basket_residual", basket_residual);
    insert_option_f64(
        &mut values,
        "graph_adjusted_momentum_6h",
        graph_adjusted_momentum_6h,
    );
    insert_option_f64(
        &mut values,
        "graph_adjusted_volatility_24h",
        graph_adjusted_volatility_24h,
    );
    insert_option_f64(&mut values, "link_disagreement_6h", link_disagreement_6h);
    insert_option_f64(
        &mut values,
        "strongest_graph_path_confidence",
        strongest_confidence,
    );
    values.insert(
        "graph_neighbor_count".to_string(),
        Value::from(i64::try_from(graph_neighbor_count).unwrap_or(i64::MAX)),
    );
    values.insert(
        "graph_missing_neighbor_data_count".to_string(),
        Value::from(i64::try_from(missing_neighbor_data_count).unwrap_or(i64::MAX)),
    );
    values.insert(
        "graph_path_evidence".to_string(),
        json!({
            "incoming_edges": context.incoming_neighbors.iter().map(|neighbor| neighbor.edge.edge_id).collect::<Vec<_>>(),
            "outgoing_edges": context.outgoing_neighbors.iter().map(|neighbor| neighbor.edge.edge_id).collect::<Vec<_>>(),
            "sector_edges": context.sector_neighbors.iter().map(|neighbor| neighbor.edge.edge_id).collect::<Vec<_>>()
        }),
    );

    GraphFeatureSnapshot {
        graph_version: context.graph_version.clone(),
        values,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ConversionMetrics {
    conversion_gap_pct: Option<f64>,
    item_set_pack_gap_gp: Option<f64>,
    item_set_unpack_gap_gp: Option<f64>,
}

fn conversion_metrics(
    incoming_neighbors: &[NeighborPriceHistory],
    outgoing_neighbors: &[NeighborPriceHistory],
) -> ConversionMetrics {
    let incoming_cost = incoming_neighbors
        .iter()
        .filter(|neighbor| {
            matches!(
                neighbor.edge.edge_type,
                GraphEdgeType::ComponentOfSet | GraphEdgeType::IngredientOf
            )
        })
        .filter_map(latest_mid)
        .sum::<f64>();
    let outgoing_value = outgoing_neighbors
        .iter()
        .filter(|neighbor| {
            matches!(
                neighbor.edge.edge_type,
                GraphEdgeType::ComponentOfSet | GraphEdgeType::ChargeConversion
            )
        })
        .filter_map(latest_mid)
        .sum::<f64>();

    let gap = if incoming_cost > 0.0 && outgoing_value > 0.0 {
        conversion_gap_pct(incoming_cost, outgoing_value, 0.0)
    } else {
        None
    };

    ConversionMetrics {
        conversion_gap_pct: gap,
        item_set_pack_gap_gp: (incoming_cost > 0.0 && outgoing_value > 0.0)
            .then_some(outgoing_value - incoming_cost),
        item_set_unpack_gap_gp: (incoming_cost > 0.0 && outgoing_value > 0.0)
            .then_some(incoming_cost - outgoing_value),
    }
}

fn collect_weighted_returns(
    neighbors: &[NeighborPriceHistory],
    config: &GraphFeatureConfig,
) -> Vec<WeightedNeighborReturn> {
    neighbors
        .iter()
        .filter(|neighbor| {
            neighbor.edge.active && neighbor.edge.confidence >= config.min_edge_confidence
        })
        .filter_map(|neighbor| {
            let return_value = neighbor_return(neighbor)?;
            Some(WeightedNeighborReturn {
                item_id: neighbor.edge.to_item_id.0,
                return_value,
                edge_weight: neighbor.edge.weight,
                edge_confidence: neighbor.edge.confidence,
                lag_seconds: neighbor.edge.lag_seconds,
            })
        })
        .collect()
}

fn collect_weighted_prices(
    neighbors: &[NeighborPriceHistory],
    config: &GraphFeatureConfig,
) -> Vec<WeightedNeighborPrice> {
    neighbors
        .iter()
        .filter(|neighbor| {
            neighbor.edge.active && neighbor.edge.confidence >= config.min_edge_confidence
        })
        .filter_map(|neighbor| {
            let mid = latest_mid(neighbor)?;
            if mid <= 0.0 {
                return None;
            }
            Some(WeightedNeighborPrice {
                item_id: neighbor.edge.to_item_id.0,
                log_price: mid.ln(),
                edge_weight: neighbor.edge.weight,
                edge_confidence: neighbor.edge.confidence,
            })
        })
        .collect()
}

fn neighbor_volatilities(
    neighbors: &[NeighborPriceHistory],
    config: &GraphFeatureConfig,
) -> Vec<f64> {
    neighbors
        .iter()
        .filter(|neighbor| {
            neighbor.edge.active && neighbor.edge.confidence >= config.min_edge_confidence
        })
        .filter_map(|neighbor| {
            let mids = neighbor_mids(neighbor);
            if mids.len() < 2 {
                return None;
            }
            let returns = mids
                .windows(2)
                .filter_map(|window| {
                    if window[0] > 0.0 && window[1] > 0.0 {
                        Some((window[1] / window[0]).ln())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if returns.is_empty() {
                None
            } else {
                let mean = returns.iter().sum::<f64>() / returns.len() as f64;
                let variance = returns
                    .iter()
                    .map(|value| {
                        let delta = value - mean;
                        delta * delta
                    })
                    .sum::<f64>()
                    / returns.len() as f64;
                Some(variance.sqrt())
            }
        })
        .collect()
}

fn neighbor_return(neighbor: &NeighborPriceHistory) -> Option<f64> {
    let mids = neighbor_mids(neighbor);
    if mids.len() < 7 {
        return None;
    }
    let current = *mids.last()?;
    let previous = mids.get(mids.len() - 7).copied()?;
    if current <= 0.0 || previous <= 0.0 {
        return None;
    }
    Some((current / previous).ln())
}

fn neighbor_mids(neighbor: &NeighborPriceHistory) -> Vec<f64> {
    neighbor
        .interval_1h
        .iter()
        .filter_map(|row| {
            match (
                row.avg_high_price.map(Gp::as_i64),
                row.avg_low_price.map(Gp::as_i64),
            ) {
                (Some(high), Some(low)) => Some((high as f64 + low as f64) / 2.0),
                _ => None,
            }
        })
        .collect()
}

fn latest_mid(neighbor: &NeighborPriceHistory) -> Option<f64> {
    neighbor_mids(neighbor).last().copied()
}

fn weighted_average_return(neighbors: &[WeightedNeighborReturn]) -> Option<f64> {
    let total_weight = neighbors
        .iter()
        .map(|neighbor| neighbor.edge_weight * neighbor.edge_confidence)
        .sum::<f64>();
    if total_weight <= f64::EPSILON {
        return None;
    }
    Some(
        neighbors
            .iter()
            .map(|neighbor| neighbor.return_value * neighbor.edge_weight * neighbor.edge_confidence)
            .sum::<f64>()
            / total_weight,
    )
}

fn weighted_average_price(neighbors: &[WeightedNeighborPrice]) -> Option<f64> {
    let total_weight = neighbors
        .iter()
        .map(|neighbor| neighbor.edge_weight * neighbor.edge_confidence)
        .sum::<f64>();
    if total_weight <= f64::EPSILON {
        return None;
    }
    Some(
        neighbors
            .iter()
            .map(|neighbor| neighbor.log_price * neighbor.edge_weight * neighbor.edge_confidence)
            .sum::<f64>()
            / total_weight,
    )
}

fn insert_option_f64(values: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    values.insert(key.to_string(), value.map_or(Value::Null, Value::from));
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};
    use grand_edge_domain::{
        Gp, GraphEdgeDirection, GraphEdgeSourceType, GraphEdgeType, IntervalPrice, ItemGraphEdge,
        ItemId, PriceInterval,
    };
    use uuid::Uuid;

    use super::*;

    #[test]
    fn upstream_pressure_matches_weighted_fixture() {
        let rows = vec![
            WeightedNeighborReturn {
                item_id: 1,
                return_value: 0.10,
                edge_weight: 0.5,
                edge_confidence: 1.0,
                lag_seconds: None,
            },
            WeightedNeighborReturn {
                item_id: 2,
                return_value: 0.04,
                edge_weight: 0.5,
                edge_confidence: 1.0,
                lag_seconds: None,
            },
        ];
        assert_eq!(upstream_pressure(&rows), Some(0.07));
    }

    #[test]
    fn conversion_gap_pct_matches_fixture() {
        let gap = conversion_gap_pct(250.0, 270.0, 0.0).unwrap();
        assert!((gap - 0.08).abs() < 1e-9);
    }

    #[test]
    fn link_disagreement_matches_fixture() {
        assert!((link_disagreement(0.12, 0.07) - 0.05).abs() < 1e-12);
    }

    #[test]
    fn graph_feature_keys_are_stable_for_v1() {
        assert_eq!(GRAPH_FEATURE_SET_VERSION, "graph_features_v1");
        assert_eq!(GRAPH_FEATURE_KEYS[0], "graph_version");
        assert_eq!(
            GRAPH_FEATURE_KEYS.last().copied(),
            Some("graph_missing_neighbor_data_count")
        );
    }

    #[test]
    fn graph_snapshot_includes_graph_version_and_missing_count() {
        let context = fixture_graph_context();
        let snapshot = build_graph_feature_snapshot(
            Some(0.06),
            Some(0.03),
            Some(270.0),
            &context,
            &GraphFeatureConfig::default(),
        );
        assert_eq!(
            snapshot.values.get("graph_version").and_then(Value::as_str),
            Some("graph_v1")
        );
        assert!(
            snapshot
                .values
                .contains_key("graph_missing_neighbor_data_count")
        );
    }

    fn fixture_graph_context() -> GraphFeatureContext {
        GraphFeatureContext {
            graph_version: "graph_v1".to_string(),
            incoming_neighbors: vec![neighbor_fixture(
                GraphEdgeType::IngredientOf,
                ItemId(100),
                ItemId(200),
                &[100.0, 110.0, 115.0, 120.0, 130.0, 135.0, 140.0],
                0.9,
                0.8,
            )],
            outgoing_neighbors: vec![neighbor_fixture(
                GraphEdgeType::ComponentOfSet,
                ItemId(200),
                ItemId(300),
                &[200.0, 202.0, 210.0, 215.0, 220.0, 225.0, 230.0],
                0.8,
                0.9,
            )],
            sector_neighbors: vec![neighbor_fixture(
                GraphEdgeType::SameCategory,
                ItemId(200),
                ItemId(400),
                &[250.0, 252.0, 255.0, 258.0, 261.0, 264.0, 267.0],
                0.7,
                0.95,
            )],
        }
    }

    fn neighbor_fixture(
        edge_type: GraphEdgeType,
        from_item_id: ItemId,
        to_item_id: ItemId,
        mids: &[f64],
        weight: f64,
        confidence: f64,
    ) -> NeighborPriceHistory {
        let base_time = Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();
        NeighborPriceHistory {
            edge: ItemGraphEdge {
                edge_id: Uuid::new_v4(),
                graph_version: "graph_v1".to_string(),
                from_item_id,
                to_item_id,
                edge_type,
                direction: GraphEdgeDirection::Upstream,
                sign: 1.0,
                weight,
                lag_seconds: None,
                confidence,
                source_type: GraphEdgeSourceType::Mechanical,
                source_ref: Some("fixture".to_string()),
                observations: Vec::new(),
                formula: json!({}),
                requires_review: false,
                active: true,
                created_at: base_time,
                updated_at: base_time,
            },
            interval_1h: mids
                .iter()
                .enumerate()
                .map(|(index, mid)| IntervalPrice {
                    item_id: to_item_id,
                    bucket_start: base_time - Duration::hours((mids.len() - index) as i64),
                    interval: PriceInterval::OneHour,
                    avg_high_price: Some(Gp(mid.ceil() as i64)),
                    high_price_volume: 100,
                    avg_low_price: Some(Gp(mid.floor() as i64)),
                    low_price_volume: 90,
                })
                .collect(),
        }
    }
}
