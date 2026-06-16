use grand_edge_domain::PaperBet;
use sqlx::PgPool;
use uuid::Uuid;

use crate::StorageError;

#[derive(Clone)]
pub struct SimulationRepository {
    pool: PgPool,
}

impl SimulationRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_simulation_run(
        &self,
        run_id: Uuid,
        name: &str,
        strategy_config: serde_json::Value,
        status: &str,
    ) -> Result<u64, StorageError> {
        let result = sqlx::query(
            r#"
            INSERT INTO simulation_runs (run_id, name, strategy_config, started_at, status)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT (run_id) DO UPDATE SET
                name = EXCLUDED.name,
                strategy_config = EXCLUDED.strategy_config,
                status = EXCLUDED.status
            "#,
        )
        .bind(run_id)
        .bind(name)
        .bind(strategy_config)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn insert_paper_bets(&self, _rows: &[PaperBet]) -> Result<u64, StorageError> {
        Ok(0)
    }
}
