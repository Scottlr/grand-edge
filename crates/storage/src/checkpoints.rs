use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{PgPool, Row};

use crate::StorageError;

#[derive(Clone)]
pub struct CheckpointRepository {
    pool: PgPool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCheckpoint<T> {
    pub key: String,
    pub value: T,
    pub updated_at: DateTime<Utc>,
}

impl CheckpointRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_json<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<StoredCheckpoint<T>>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT checkpoint_key, checkpoint_value, updated_at
            FROM ingestion_checkpoints
            WHERE checkpoint_key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(StoredCheckpoint {
                key: row.try_get("checkpoint_key")?,
                value: serde_json::from_value(row.try_get("checkpoint_value")?)?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .transpose()
    }

    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<u64, StorageError> {
        let result = sqlx::query(
            r#"
            INSERT INTO ingestion_checkpoints (checkpoint_key, checkpoint_value, updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (checkpoint_key) DO UPDATE SET
                checkpoint_value = EXCLUDED.checkpoint_value,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(key)
        .bind(serde_json::to_value(value)?)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::StoredCheckpoint;

    #[test]
    fn stored_checkpoint_is_comparable_for_test_assertions() {
        let left = StoredCheckpoint {
            key: "mapping".to_string(),
            value: 42_u32,
            updated_at: chrono::Utc::now(),
        };
        let right = left.clone();
        assert_eq!(left, right);
    }
}
