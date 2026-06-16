use chrono::{Duration, TimeZone, Utc};
use grand_edge_domain::{
    FeatureVector, Gp, GraphEdgeDirection, GraphEdgeSourceType, GraphEdgeType, IntervalPrice, Item,
    ItemGraphEdge, ItemId, LatestPrice, PriceInterval,
};
use uuid::Uuid;

use crate::{FEATURE_SET_VERSION, GraphFeatureContext, ItemFeatureInput, NeighborPriceHistory};

pub fn feature_fixture_input() -> ItemFeatureInput {
    let as_of = Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();
    let item = Item {
        item_id: ItemId(4151),
        name: "Abyssal whip".to_string(),
        examine: Some("A weapon from the abyss.".to_string()),
        members: true,
        buy_limit: Some(70),
        low_alch: Some(Gp(48_000)),
        high_alch: Some(Gp(72_000)),
        value: Some(Gp(120_001)),
        icon: None,
        updated_at: as_of,
    };
    let latest = LatestPrice {
        item_id: item.item_id,
        high: Some(Gp(100)),
        high_time: Some(as_of - Duration::minutes(1)),
        low: Some(Gp(80)),
        low_time: Some(as_of - Duration::minutes(2)),
        observed_at: as_of,
    };

    let interval_5m = (0..12)
        .map(|index| {
            interval_price_row(
                PriceInterval::FiveMinute,
                10 + index,
                8 + index,
                70 + index as i64,
                60 + index as i64,
                i64::from(11 - index),
            )
        })
        .collect();
    let interval_1h = (0..24)
        .map(|index| {
            interval_price_row(
                PriceInterval::OneHour,
                200 + index * 10,
                170 + index * 8,
                90 + index as i64,
                80 + index as i64,
                i64::from(23 - index),
            )
        })
        .collect();

    ItemFeatureInput {
        item,
        latest,
        interval_5m,
        interval_1h,
        as_of,
        graph_context: None,
    }
}

pub fn graph_feature_fixture_input() -> ItemFeatureInput {
    let mut input = feature_fixture_input();
    let graph_version = "graph_v1".to_string();
    input.graph_context = Some(GraphFeatureContext {
        graph_version: graph_version.clone(),
        incoming_neighbors: vec![neighbor_history_fixture(
            graph_version.clone(),
            GraphEdgeType::IngredientOf,
            ItemId(11818),
            ItemId(12873),
            &[40, 42, 43, 44, 45, 46, 47, 48],
        )],
        outgoing_neighbors: vec![neighbor_history_fixture(
            graph_version.clone(),
            GraphEdgeType::ComponentOfSet,
            ItemId(12873),
            ItemId(11820),
            &[80, 81, 82, 83, 84, 85, 86, 87],
        )],
        sector_neighbors: vec![neighbor_history_fixture(
            graph_version,
            GraphEdgeType::SameCategory,
            ItemId(4151),
            ItemId(13265),
            &[88, 89, 90, 91, 92, 93, 94, 95],
        )],
    });
    input
}

pub fn interval_price_row(
    interval: PriceInterval,
    high_volume: i64,
    low_volume: i64,
    avg_high_price: i64,
    avg_low_price: i64,
    hours_ago: i64,
) -> IntervalPrice {
    IntervalPrice {
        item_id: ItemId(4151),
        bucket_start: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap()
            - Duration::hours(hours_ago),
        interval,
        avg_high_price: Some(Gp(avg_high_price)),
        high_price_volume: high_volume,
        avg_low_price: Some(Gp(avg_low_price)),
        low_price_volume: low_volume,
    }
}

pub fn empty_feature_vector() -> FeatureVector {
    FeatureVector {
        item_id: ItemId(4151),
        as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        feature_set_version: FEATURE_SET_VERSION.to_string(),
        values: serde_json::Map::new(),
    }
}

fn neighbor_history_fixture(
    graph_version: String,
    edge_type: GraphEdgeType,
    from_item_id: ItemId,
    to_item_id: ItemId,
    mids: &[i64],
) -> NeighborPriceHistory {
    let base_time = Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap();
    NeighborPriceHistory {
        edge: ItemGraphEdge {
            edge_id: Uuid::new_v4(),
            graph_version,
            from_item_id,
            to_item_id,
            edge_type,
            direction: GraphEdgeDirection::Upstream,
            sign: 1.0,
            weight: 0.8,
            lag_seconds: None,
            confidence: 0.9,
            source_type: GraphEdgeSourceType::Mechanical,
            source_ref: Some("fixture".to_string()),
            observations: Vec::new(),
            formula: serde_json::json!({}),
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
                avg_high_price: Some(Gp(*mid + 1)),
                high_price_volume: 120,
                avg_low_price: Some(Gp(*mid - 1)),
                low_price_volume: 100,
            })
            .collect(),
    }
}
