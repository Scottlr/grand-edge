use chrono::{DateTime, Utc};
use grand_edge_domain::{FeatureVector, ItemId};
use sqlx::{PgPool, Row};

use crate::StorageError;

#[derive(Clone)]
pub struct FeatureRepository {
    pool: PgPool,
}

impl FeatureRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_feature_vectors(
        &self,
        rows: &[FeatureVector],
    ) -> Result<u64, StorageError> {
        let mut affected = 0;
        for row in rows {
            let result = sqlx::query(
                r#"
                INSERT INTO features (item_id, as_of, feature_set_version, features)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (item_id, as_of, feature_set_version) DO UPDATE SET
                    features = EXCLUDED.features
                "#,
            )
            .bind(row.item_id.0)
            .bind(row.as_of)
            .bind(&row.feature_set_version)
            .bind(serde_json::Value::Object(row.values.clone()))
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn latest_features(
        &self,
        feature_set_version: &str,
    ) -> Result<Vec<FeatureVector>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (item_id) item_id, as_of, feature_set_version, features
            FROM features
            WHERE feature_set_version = $1
            ORDER BY item_id, as_of DESC
            "#,
        )
        .bind(feature_set_version)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let values: serde_json::Value = row.try_get("features")?;
                let values = values.as_object().cloned().unwrap_or_default();
                Ok(FeatureVector {
                    item_id: grand_edge_domain::ItemId(row.try_get::<i64, _>("item_id")?),
                    as_of: row.try_get("as_of")?,
                    feature_set_version: row.try_get("feature_set_version")?,
                    values,
                })
            })
            .collect()
    }

    pub async fn list_between(
        &self,
        feature_set_version: &str,
        item_ids: Option<&[ItemId]>,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    ) -> Result<Vec<FeatureVector>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, as_of, feature_set_version, features
            FROM features
            WHERE feature_set_version = $1
              AND as_of >= $2
              AND as_of <= $3
            ORDER BY as_of ASC, item_id ASC
            "#,
        )
        .bind(feature_set_version)
        .bind(window_start)
        .bind(window_end)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .filter_map(|row| {
                let item_id = ItemId(row.try_get::<i64, _>("item_id").ok()?);
                if item_ids.is_some_and(|ids| !ids.contains(&item_id)) {
                    return None;
                }

                let values: serde_json::Value = row.try_get("features").ok()?;
                let values = values.as_object().cloned().unwrap_or_default();
                Some(Ok(FeatureVector {
                    item_id,
                    as_of: row.try_get("as_of").ok()?,
                    feature_set_version: row.try_get("feature_set_version").ok()?,
                    values,
                }))
            })
            .collect()
    }
}
