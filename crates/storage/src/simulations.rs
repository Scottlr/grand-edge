use chrono::{DateTime, Utc};
use grand_edge_domain::{Gp, ItemId, ModelVersion, PaperBet, StrategyId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::StorageError;

#[derive(Debug, Clone)]
pub struct StoredPaperBet {
    pub bet_id: Uuid,
    pub run_id: Uuid,
    pub recommendation_id: Option<Uuid>,
    pub strategy_id: StrategyId,
    pub model_version: ModelVersion,
    pub item_id: ItemId,
    pub entry_time: DateTime<Utc>,
    pub entry_price: Gp,
    pub quantity: i64,
    pub target_exit: Option<Gp>,
    pub stop_loss: Option<Gp>,
    pub exit_time: Option<DateTime<Utc>>,
    pub exit_price: Option<Gp>,
    pub tax_paid: i64,
    pub realized_profit_gp: Option<i64>,
    pub realized_roi: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub hit_reason: Option<String>,
    pub status: String,
    pub explanation: serde_json::Value,
}

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

    pub async fn list_paper_bets_for_strategy(
        &self,
        strategy_id: &str,
        model_version: &str,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    ) -> Result<Vec<StoredPaperBet>, StorageError> {
        let rows = sqlx::query_as::<_, PaperBetRow>(
            r#"
            SELECT
                bet_id,
                run_id,
                recommendation_id,
                strategy_id,
                model_version,
                item_id,
                entry_time,
                entry_price,
                quantity,
                target_exit,
                stop_loss,
                exit_time,
                exit_price,
                tax_paid,
                realized_profit_gp,
                realized_roi,
                max_drawdown,
                hit_reason,
                status,
                explanation
            FROM paper_bets
            WHERE strategy_id = $1
              AND model_version = $2
              AND entry_time >= $3
              AND entry_time <= $4
            ORDER BY entry_time ASC
            "#,
        )
        .bind(strategy_id)
        .bind(model_version)
        .bind(window_start)
        .bind(window_end)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryFrom::try_from).collect()
    }
}

#[derive(sqlx::FromRow)]
struct PaperBetRow {
    bet_id: Uuid,
    run_id: Uuid,
    recommendation_id: Option<Uuid>,
    strategy_id: String,
    model_version: String,
    item_id: i64,
    entry_time: DateTime<Utc>,
    entry_price: i64,
    quantity: i64,
    target_exit: Option<i64>,
    stop_loss: Option<i64>,
    exit_time: Option<DateTime<Utc>>,
    exit_price: Option<i64>,
    tax_paid: i64,
    realized_profit_gp: Option<i64>,
    realized_roi: Option<f64>,
    max_drawdown: Option<f64>,
    hit_reason: Option<String>,
    status: String,
    explanation: serde_json::Value,
}

impl TryFrom<PaperBetRow> for StoredPaperBet {
    type Error = StorageError;

    fn try_from(value: PaperBetRow) -> Result<Self, Self::Error> {
        Ok(Self {
            bet_id: value.bet_id,
            run_id: value.run_id,
            recommendation_id: value.recommendation_id,
            strategy_id: StrategyId::new(value.strategy_id)?,
            model_version: ModelVersion::new(value.model_version)?,
            item_id: ItemId(value.item_id),
            entry_time: value.entry_time,
            entry_price: Gp(value.entry_price),
            quantity: value.quantity,
            target_exit: value.target_exit.map(Gp),
            stop_loss: value.stop_loss.map(Gp),
            exit_time: value.exit_time,
            exit_price: value.exit_price.map(Gp),
            tax_paid: value.tax_paid,
            realized_profit_gp: value.realized_profit_gp,
            realized_roi: value.realized_roi,
            max_drawdown: value.max_drawdown,
            hit_reason: value.hit_reason,
            status: value.status,
            explanation: value.explanation,
        })
    }
}
