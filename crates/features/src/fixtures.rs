use chrono::{Duration, TimeZone, Utc};
use grand_edge_domain::{
    FeatureVector, Gp, IntervalPrice, Item, ItemId, LatestPrice, PriceInterval,
};

use crate::{FEATURE_SET_VERSION, ItemFeatureInput};

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
    }
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
