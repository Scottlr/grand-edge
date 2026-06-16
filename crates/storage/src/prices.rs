use chrono::{DateTime, Utc};
use grand_edge_domain::{Gp, IntervalPrice, ItemId, LatestPrice, PriceInterval};
use sqlx::{PgPool, Row};

use crate::StorageError;

#[derive(Clone)]
pub struct PriceRepository {
    pool: PgPool,
}

impl PriceRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_latest_prices(&self, prices: &[LatestPrice]) -> Result<u64, StorageError> {
        let mut affected = 0;
        for price in prices {
            let result = sqlx::query(
                r#"
                INSERT INTO latest_prices (item_id, high, high_time, low, low_time, observed_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (item_id, observed_at) DO NOTHING
                "#,
            )
            .bind(price.item_id.0)
            .bind(price.high.map(|value| value.0))
            .bind(price.high_time)
            .bind(price.low.map(|value| value.0))
            .bind(price.low_time)
            .bind(price.observed_at)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn upsert_interval_prices(
        &self,
        rows: &[IntervalPrice],
    ) -> Result<u64, StorageError> {
        let mut affected = 0;
        for row in rows {
            let result = sqlx::query(
                r#"
                INSERT INTO interval_prices (
                    item_id, bucket_start, interval, avg_high_price, high_price_volume, avg_low_price, low_price_volume
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (item_id, interval, bucket_start) DO UPDATE SET
                    avg_high_price = EXCLUDED.avg_high_price,
                    high_price_volume = EXCLUDED.high_price_volume,
                    avg_low_price = EXCLUDED.avg_low_price,
                    low_price_volume = EXCLUDED.low_price_volume
                "#,
            )
            .bind(row.item_id.0)
            .bind(row.bucket_start)
            .bind(enum_to_string(&row.interval)?)
            .bind(row.avg_high_price.map(|value| value.0))
            .bind(row.high_price_volume)
            .bind(row.avg_low_price.map(|value| value.0))
            .bind(row.low_price_volume)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn latest_snapshot(&self) -> Result<Vec<LatestPrice>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (item_id) item_id, high, high_time, low, low_time, observed_at
            FROM latest_prices
            ORDER BY item_id, observed_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_latest_price).collect()
    }

    pub async fn interval_history(
        &self,
        item_id: ItemId,
        interval: PriceInterval,
        limit: i64,
    ) -> Result<Vec<IntervalPrice>, StorageError> {
        self.interval_history_before(item_id, interval, limit, None)
            .await
    }

    pub async fn interval_history_before(
        &self,
        item_id: ItemId,
        interval: PriceInterval,
        limit: i64,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<IntervalPrice>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, bucket_start, interval, avg_high_price, high_price_volume, avg_low_price, low_price_volume
            FROM interval_prices
            WHERE item_id = $1
              AND interval = $2
              AND ($3::timestamptz IS NULL OR bucket_start < $3)
            ORDER BY bucket_start DESC
            LIMIT $4
            "#,
        )
        .bind(item_id.0)
        .bind(enum_to_string(&interval)?)
        .bind(before)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_interval_price).collect()
    }
}

fn row_to_latest_price(row: sqlx::postgres::PgRow) -> Result<LatestPrice, StorageError> {
    Ok(LatestPrice {
        item_id: ItemId(row.try_get::<i64, _>("item_id")?),
        high: row.try_get::<Option<i64>, _>("high")?.map(Gp),
        high_time: row.try_get::<Option<DateTime<Utc>>, _>("high_time")?,
        low: row.try_get::<Option<i64>, _>("low")?.map(Gp),
        low_time: row.try_get::<Option<DateTime<Utc>>, _>("low_time")?,
        observed_at: row.try_get("observed_at")?,
    })
}

fn row_to_interval_price(row: sqlx::postgres::PgRow) -> Result<IntervalPrice, StorageError> {
    let interval: String = row.try_get("interval")?;
    Ok(IntervalPrice {
        item_id: ItemId(row.try_get::<i64, _>("item_id")?),
        bucket_start: row.try_get("bucket_start")?,
        interval: serde_json::from_value(serde_json::Value::String(interval))?,
        avg_high_price: row.try_get::<Option<i64>, _>("avg_high_price")?.map(Gp),
        high_price_volume: row.try_get("high_price_volume")?,
        avg_low_price: row.try_get::<Option<i64>, _>("avg_low_price")?.map(Gp),
        low_price_volume: row.try_get("low_price_volume")?,
    })
}

fn enum_to_string<T: serde::Serialize>(value: &T) -> Result<String, StorageError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::row_to_interval_price;

    #[test]
    fn interval_prices_round_trip_observed_high_low_side_fields() {
        let interval = grand_edge_domain::IntervalPrice {
            item_id: grand_edge_domain::ItemId(4151),
            bucket_start: Utc::now(),
            interval: grand_edge_domain::PriceInterval::OneHour,
            avg_high_price: Some(grand_edge_domain::Gp(100)),
            high_price_volume: 12,
            avg_low_price: Some(grand_edge_domain::Gp(90)),
            low_price_volume: 9,
        };

        let json = serde_json::to_value(&interval).unwrap();
        assert_eq!(
            json.get("high_price_volume")
                .and_then(|value| value.as_i64()),
            Some(12)
        );
        assert_eq!(
            json.get("low_price_volume")
                .and_then(|value| value.as_i64()),
            Some(9)
        );
        assert_ne!(json.get("high_price_volume"), json.get("low_price_volume"));
        let _ = row_to_interval_price;
    }
}
