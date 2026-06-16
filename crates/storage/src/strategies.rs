use chrono::{DateTime, Utc};
use grand_edge_domain::{
    Gp, ItemId, ModelVersion, Probability, Quantity, Rate, SignalSide, StrategyId, StrategySignal,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::StorageError;

#[derive(Debug, Clone)]
pub struct StoredPrediction {
    pub strategy_id: StrategyId,
    pub model_version: ModelVersion,
    pub item_id: ItemId,
    pub as_of: DateTime<Utc>,
    pub horizon_secs: i64,
    pub side: SignalSide,
    pub expected_return: Rate,
    pub confidence: Probability,
    pub expected_net_gp_per_unit: Gp,
    pub target_entry: Option<Gp>,
    pub target_exit: Option<Gp>,
    pub stop_loss: Option<Gp>,
    pub take_profit: Option<Gp>,
    pub max_quantity: Option<Quantity>,
    pub explanation: serde_json::Value,
}

#[derive(Clone)]
pub struct StrategyRepository {
    pool: PgPool,
}

impl StrategyRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_predictions(&self, rows: &[StrategySignal]) -> Result<u64, StorageError> {
        let mut affected = 0;
        for row in rows {
            let result = sqlx::query(
                r#"
                INSERT INTO strategy_predictions (
                    prediction_id, strategy_id, model_version, item_id, as_of, horizon_secs, side,
                    expected_return, confidence, expected_net_gp_per_unit, target_entry, target_exit,
                    stop_loss, take_profit, max_quantity, explanation
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10, $11, $12,
                    $13, $14, $15, $16
                )
                ON CONFLICT (prediction_id) DO UPDATE SET
                    strategy_id = EXCLUDED.strategy_id,
                    model_version = EXCLUDED.model_version,
                    item_id = EXCLUDED.item_id,
                    as_of = EXCLUDED.as_of,
                    horizon_secs = EXCLUDED.horizon_secs,
                    side = EXCLUDED.side,
                    expected_return = EXCLUDED.expected_return,
                    confidence = EXCLUDED.confidence,
                    expected_net_gp_per_unit = EXCLUDED.expected_net_gp_per_unit,
                    target_entry = EXCLUDED.target_entry,
                    target_exit = EXCLUDED.target_exit,
                    stop_loss = EXCLUDED.stop_loss,
                    take_profit = EXCLUDED.take_profit,
                    max_quantity = EXCLUDED.max_quantity,
                    explanation = EXCLUDED.explanation
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(&row.strategy_id.0)
            .bind(&row.model_version.0)
            .bind(row.item_id.0)
            .bind(row.as_of)
            .bind(row.horizon_secs.0)
            .bind(enum_to_string(&row.side)?)
            .bind(row.expected_return.get())
            .bind(row.confidence.get())
            .bind(row.expected_net_gp_per_unit.0)
            .bind(row.target_entry.map(|value| value.0))
            .bind(row.target_exit.map(|value| value.0))
            .bind(row.stop_loss.map(|value| value.0))
            .bind(row.take_profit.map(|value| value.0))
            .bind(row.max_quantity.map(|value| value.0))
            .bind(&row.explanation)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn list_predictions_for_strategy(
        &self,
        strategy_id: &str,
        model_version: &str,
        horizon_secs: i64,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    ) -> Result<Vec<StoredPrediction>, StorageError> {
        let rows = sqlx::query_as::<_, PredictionRow>(
            r#"
            SELECT
                strategy_id,
                model_version,
                item_id,
                as_of,
                horizon_secs,
                side,
                expected_return,
                confidence,
                expected_net_gp_per_unit,
                target_entry,
                target_exit,
                stop_loss,
                take_profit,
                max_quantity,
                explanation
            FROM strategy_predictions
            WHERE strategy_id = $1
              AND model_version = $2
              AND horizon_secs = $3
              AND as_of >= $4
              AND as_of <= $5
            ORDER BY as_of ASC
            "#,
        )
        .bind(strategy_id)
        .bind(model_version)
        .bind(horizon_secs)
        .bind(window_start)
        .bind(window_end)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryFrom::try_from).collect()
    }

    pub async fn list_latest_predictions(
        &self,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<StoredPrediction>, StorageError> {
        let rows = sqlx::query_as::<_, PredictionRow>(
            r#"
            SELECT DISTINCT ON (strategy_id, model_version, item_id)
                strategy_id,
                model_version,
                item_id,
                as_of,
                horizon_secs,
                side,
                expected_return,
                confidence,
                expected_net_gp_per_unit,
                target_entry,
                target_exit,
                stop_loss,
                take_profit,
                max_quantity,
                explanation
            FROM strategy_predictions
            WHERE as_of <= $1
            ORDER BY strategy_id, model_version, item_id, as_of DESC
            "#,
        )
        .bind(as_of)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryFrom::try_from).collect()
    }

    pub async fn list_latest_predictions_for_item(
        &self,
        item_id: ItemId,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<StoredPrediction>, StorageError> {
        let rows = sqlx::query_as::<_, PredictionRow>(
            r#"
            SELECT DISTINCT ON (strategy_id, model_version, item_id)
                strategy_id,
                model_version,
                item_id,
                as_of,
                horizon_secs,
                side,
                expected_return,
                confidence,
                expected_net_gp_per_unit,
                target_entry,
                target_exit,
                stop_loss,
                take_profit,
                max_quantity,
                explanation
            FROM strategy_predictions
            WHERE item_id = $1
              AND as_of <= $2
            ORDER BY strategy_id, model_version, item_id, as_of DESC
            "#,
        )
        .bind(item_id.0)
        .bind(as_of)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(TryFrom::try_from).collect()
    }
}

fn enum_to_string<T: serde::Serialize>(value: &T) -> Result<String, StorageError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}

#[derive(sqlx::FromRow)]
struct PredictionRow {
    strategy_id: String,
    model_version: String,
    item_id: i64,
    as_of: DateTime<Utc>,
    horizon_secs: i64,
    side: String,
    expected_return: f64,
    confidence: f64,
    expected_net_gp_per_unit: i64,
    target_entry: Option<i64>,
    target_exit: Option<i64>,
    stop_loss: Option<i64>,
    take_profit: Option<i64>,
    max_quantity: Option<i64>,
    explanation: serde_json::Value,
}

impl TryFrom<PredictionRow> for StoredPrediction {
    type Error = StorageError;

    fn try_from(value: PredictionRow) -> Result<Self, Self::Error> {
        let side = serde_json::from_value(serde_json::Value::String(value.side))?;
        Ok(Self {
            strategy_id: StrategyId::new(value.strategy_id)?,
            model_version: ModelVersion::new(value.model_version)?,
            item_id: ItemId(value.item_id),
            as_of: value.as_of,
            horizon_secs: value.horizon_secs,
            side,
            expected_return: Rate::new(value.expected_return)?,
            confidence: Probability::new(value.confidence)?,
            expected_net_gp_per_unit: Gp(value.expected_net_gp_per_unit),
            target_entry: value.target_entry.map(Gp),
            target_exit: value.target_exit.map(Gp),
            stop_loss: value.stop_loss.map(Gp),
            take_profit: value.take_profit.map(Gp),
            max_quantity: value.max_quantity.map(Quantity),
            explanation: value.explanation,
        })
    }
}
