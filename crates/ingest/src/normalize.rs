use chrono::{DateTime, Utc};
use grand_edge_domain::{Gp, IntervalPrice, Item, ItemId, LatestPrice, PriceInterval};

use crate::{
    IngestError, IntervalBulkResponseRaw, IntervalPriceRaw, LatestResponseRaw, MappingItemRaw,
    TimeseriesResponseRaw, item_icon_from_mapping_icon,
};

pub fn normalize_mapping(
    items: Vec<MappingItemRaw>,
    observed_at: DateTime<Utc>,
) -> Result<Vec<Item>, IngestError> {
    items
        .into_iter()
        .map(|item| normalize_item(item, observed_at))
        .collect()
}

pub fn normalize_latest(
    response: LatestResponseRaw,
    observed_at: DateTime<Utc>,
) -> Result<Vec<LatestPrice>, IngestError> {
    response
        .data
        .into_iter()
        .map(|(item_id, price)| {
            let item_id = parse_item_id_key(item_id)?;
            Ok(LatestPrice {
                item_id,
                high: optional_gp(price.high, "high")?,
                high_time: optional_timestamp(price.high_time)?,
                low: optional_gp(price.low, "low")?,
                low_time: optional_timestamp(price.low_time)?,
                observed_at,
            })
        })
        .collect()
}

pub fn normalize_interval_bulk(
    response: IntervalBulkResponseRaw,
    interval: PriceInterval,
) -> Result<Vec<IntervalPrice>, IngestError> {
    let bucket_start = unix_seconds_to_utc(response.timestamp)?;
    response
        .data
        .into_iter()
        .map(|(item_id, row)| {
            normalize_interval_row(parse_item_id_key(item_id)?, row, interval, bucket_start)
        })
        .collect()
}

pub fn normalize_timeseries(
    item_id: i64,
    response: TimeseriesResponseRaw,
    interval: PriceInterval,
) -> Result<Vec<IntervalPrice>, IngestError> {
    let item_id = ItemId::try_from(item_id).map_err(|source| IngestError::InvalidItemId {
        value: item_id,
        source,
    })?;

    response
        .data
        .into_iter()
        .map(|row| {
            let bucket_start = unix_seconds_to_utc(row.timestamp)?;
            normalize_interval_row(
                item_id,
                interval_raw_from_timeseries(row)?,
                interval,
                bucket_start,
            )
        })
        .collect()
}

pub fn unix_seconds_to_utc(seconds: i64) -> Result<DateTime<Utc>, IngestError> {
    DateTime::<Utc>::from_timestamp(seconds, 0).ok_or(IngestError::InvalidTimestamp(seconds))
}

fn normalize_item(item: MappingItemRaw, observed_at: DateTime<Utc>) -> Result<Item, IngestError> {
    Ok(Item {
        item_id: ItemId::try_from(item.id).map_err(|source| IngestError::InvalidItemId {
            value: item.id,
            source,
        })?,
        name: item.name,
        examine: item.examine,
        members: item.members,
        buy_limit: item.limit,
        low_alch: optional_gp(item.lowalch, "lowalch")?,
        high_alch: optional_gp(item.highalch, "highalch")?,
        value: optional_gp(item.value, "value")?,
        icon: item_icon_from_mapping_icon(item.icon.as_deref())?,
        updated_at: observed_at,
    })
}

fn parse_item_id_key(raw: String) -> Result<ItemId, IngestError> {
    let value = raw
        .parse::<i64>()
        .map_err(|_| IngestError::InvalidLatestKey(raw.clone()))?;
    ItemId::try_from(value).map_err(|source| IngestError::InvalidItemId { value, source })
}

fn normalize_interval_row(
    item_id: ItemId,
    row: IntervalPriceRaw,
    interval: PriceInterval,
    bucket_start: DateTime<Utc>,
) -> Result<IntervalPrice, IngestError> {
    Ok(IntervalPrice {
        item_id,
        bucket_start,
        interval,
        avg_high_price: optional_gp(row.avg_high_price, "avg_high_price")?,
        high_price_volume: row.high_price_volume.unwrap_or(0),
        avg_low_price: optional_gp(row.avg_low_price, "avg_low_price")?,
        low_price_volume: row.low_price_volume.unwrap_or(0),
    })
}

fn interval_raw_from_timeseries(
    row: crate::TimeseriesPriceRaw,
) -> Result<IntervalPriceRaw, IngestError> {
    Ok(IntervalPriceRaw {
        avg_high_price: row.avg_high_price,
        high_price_volume: non_negative_volume(row.high_price_volume, "high_price_volume")?,
        avg_low_price: row.avg_low_price,
        low_price_volume: non_negative_volume(row.low_price_volume, "low_price_volume")?,
    })
}

fn optional_gp(value: Option<i64>, field: &'static str) -> Result<Option<Gp>, IngestError> {
    value
        .map(|value| {
            Gp::try_from(value).map_err(|source| IngestError::InvalidGp {
                field,
                value,
                source,
            })
        })
        .transpose()
}

fn optional_timestamp(value: Option<i64>) -> Result<Option<DateTime<Utc>>, IngestError> {
    value.map(unix_seconds_to_utc).transpose()
}

fn non_negative_volume(
    value: Option<i64>,
    field: &'static str,
) -> Result<Option<i64>, IngestError> {
    match value {
        Some(value) if value < 0 => Err(IngestError::InvalidConfig(format!(
            "{field} must be non-negative"
        ))),
        Some(value) => Ok(Some(value)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{normalize_latest, unix_seconds_to_utc};
    use crate::{LatestPriceRaw, LatestResponseRaw};

    #[test]
    fn unix_seconds_to_utc_rejects_out_of_range_timestamps() {
        assert!(unix_seconds_to_utc(i64::MAX).is_err());
    }

    #[test]
    fn normalize_latest_keeps_missing_values_as_none() {
        let mut data = std::collections::HashMap::new();
        data.insert(
            "4151".to_string(),
            LatestPriceRaw {
                high: None,
                high_time: None,
                low: Some(3_000_000),
                low_time: Some(1_700_000_000),
            },
        );

        let prices = normalize_latest(LatestResponseRaw { data }, Utc::now()).unwrap();

        assert_eq!(prices[0].high, None);
        assert_eq!(prices[0].low.unwrap().0, 3_000_000);
    }
}
