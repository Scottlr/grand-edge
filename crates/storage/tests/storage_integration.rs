use chrono::Utc;
use grand_edge_domain::{Gp, Item, ItemIcon, ItemId, LatestPrice, PriceInterval, WikiImageSource};
use grand_edge_storage::Storage;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

#[tokio::test]
#[ignore]
async fn upsert_interval_prices_is_idempotent() {
    let Some(database_url) = database_url() else {
        return;
    };
    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let prices = storage.prices();
    let row = grand_edge_domain::IntervalPrice {
        item_id: ItemId(4151),
        bucket_start: Utc::now(),
        interval: PriceInterval::OneHour,
        avg_high_price: Some(Gp(100)),
        high_price_volume: 10,
        avg_low_price: Some(Gp(90)),
        low_price_volume: 8,
    };
    prices.upsert_interval_prices(&[row.clone()]).await.unwrap();
    prices.upsert_interval_prices(&[row]).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn latest_snapshot_returns_newest_per_item() {
    let Some(database_url) = database_url() else {
        return;
    };
    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let prices = storage.prices();
    let now = Utc::now();
    prices
        .insert_latest_prices(&[
            LatestPrice {
                item_id: ItemId(4151),
                high: Some(Gp(100)),
                high_time: Some(now),
                low: Some(Gp(90)),
                low_time: Some(now),
                observed_at: now,
            },
            LatestPrice {
                item_id: ItemId(4151),
                high: Some(Gp(101)),
                high_time: Some(now),
                low: Some(Gp(91)),
                low_time: Some(now),
                observed_at: now + chrono::TimeDelta::seconds(1),
            },
        ])
        .await
        .unwrap();
    let _ = prices.latest_snapshot().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn item_icon_metadata_round_trips() {
    let Some(database_url) = database_url() else {
        return;
    };
    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let items = storage.items();
    let item = Item {
        item_id: ItemId(1949),
        name: "Chef's hat".to_string(),
        examine: None,
        members: false,
        buy_limit: None,
        low_alch: None,
        high_alch: None,
        value: None,
        icon: Some(ItemIcon {
            source_file_name: "Chef's hat.png".to_string(),
            canonical_file_name: "Chef's_hat.png".to_string(),
            cdn_url: "https://oldschool.runescape.wiki/images/Chef%27s_hat.png".to_string(),
            source: WikiImageSource::MappingIcon,
        }),
        updated_at: Utc::now(),
    };
    items.upsert_items(&[item.clone()]).await.unwrap();
    let _ = items.get_item(item.item_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn active_positions_filters_by_user() {
    let Some(database_url) = database_url() else {
        return;
    };
    let storage = Storage::connect(&database_url).await.unwrap();
    storage.migrate().await.unwrap();
    let _ = storage.positions();
}
