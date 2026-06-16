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
}
