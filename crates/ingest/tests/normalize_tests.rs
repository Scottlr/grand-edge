use chrono::{TimeZone, Utc};
use grand_edge_domain::PriceInterval;
use grand_edge_ingest::{
    IntervalBulkResponseRaw, LatestResponseRaw, MappingItemRaw, normalize_interval_bulk,
    normalize_latest, normalize_mapping,
};

#[test]
fn mapping_normalization_builds_domain_items_with_icon_metadata() {
    let rows: Vec<MappingItemRaw> =
        serde_json::from_str(include_str!("fixtures/mapping_sample.json")).unwrap();

    let items =
        normalize_mapping(rows, Utc.with_ymd_and_hms(2024, 5, 20, 12, 0, 0).unwrap()).unwrap();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].item_id.0, 4151);
    assert_eq!(
        items[1].icon.as_ref().unwrap().cdn_url,
        "https://oldschool.runescape.wiki/images/Chef%27s_hat.png"
    );
}

#[test]
fn latest_normalization_converts_unix_seconds_to_utc() {
    let response: LatestResponseRaw =
        serde_json::from_str(include_str!("fixtures/latest_4151.json")).unwrap();
    let observed_at = Utc.with_ymd_and_hms(2024, 5, 20, 12, 5, 0).unwrap();

    let prices = normalize_latest(response, observed_at).unwrap();

    assert_eq!(prices.len(), 1);
    assert_eq!(prices[0].item_id.0, 4151);
    assert_eq!(prices[0].high.unwrap().0, 2_910_000);
    assert_eq!(prices[0].observed_at, observed_at);
    assert_eq!(
        prices[0].high_time.unwrap(),
        Utc.timestamp_opt(1716181200, 0).unwrap()
    );
}

#[test]
fn interval_bulk_normalization_uses_response_timestamp_for_bucket_start() {
    let response: IntervalBulkResponseRaw =
        serde_json::from_str(include_str!("fixtures/interval_5m_sample.json")).unwrap();

    let rows = normalize_interval_bulk(response, PriceInterval::FiveMinute).unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].bucket_start,
        Utc.timestamp_opt(1716181200, 0).unwrap()
    );
    assert_eq!(rows[0].high_price_volume, 12);
    assert_eq!(rows[0].low_price_volume, 10);
}
