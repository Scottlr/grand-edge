use chrono::{DateTime, Utc};
use grand_edge_domain::ModelAccuracySnapshot;
use sqlx::PgPool;

use crate::StorageError;

#[derive(Clone)]
pub struct MetricsRepository {
    pool: PgPool,
}

impl MetricsRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_strategy_metric(
        &self,
        snapshot: &ModelAccuracySnapshot,
        horizon_secs: i64,
        window_name: &str,
        window_start: chrono::DateTime<chrono::Utc>,
        window_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, StorageError> {
        let result = sqlx::query(
            r#"
            INSERT INTO strategy_metrics (
                strategy_id, model_version, horizon_secs, window_name, window_start, window_end, metrics
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (strategy_id, model_version, horizon_secs, window_name, window_start, window_end) DO UPDATE SET
                metrics = EXCLUDED.metrics
            "#,
        )
        .bind(&snapshot.strategy_id.0)
        .bind(&snapshot.model_version.0)
        .bind(horizon_secs)
        .bind(window_name)
        .bind(window_start)
        .bind(window_end)
        .bind(serde_json::to_value(snapshot)?)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn latest_strategy_metric(
        &self,
        strategy_id: &str,
        model_version: &str,
        window_name: &str,
    ) -> Result<Option<ModelAccuracySnapshot>, StorageError> {
        let row = sqlx::query_scalar::<_, serde_json::Value>(
            r#"
            SELECT metrics
            FROM strategy_metrics
            WHERE strategy_id = $1
              AND model_version = $2
              AND window_name = $3
            ORDER BY window_end DESC
            LIMIT 1
            "#,
        )
        .bind(strategy_id)
        .bind(model_version)
        .bind(window_name)
        .fetch_optional(&self.pool)
        .await?;

        row.map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }

    pub async fn metric_exists(
        &self,
        strategy_id: &str,
        model_version: &str,
        horizon_secs: i64,
        window_name: &str,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    ) -> Result<bool, StorageError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM strategy_metrics
                WHERE strategy_id = $1
                  AND model_version = $2
                  AND horizon_secs = $3
                  AND window_name = $4
                  AND window_start = $5
                  AND window_end = $6
            )
            "#,
        )
        .bind(strategy_id)
        .bind(model_version)
        .bind(horizon_secs)
        .bind(window_name)
        .bind(window_start)
        .bind(window_end)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
