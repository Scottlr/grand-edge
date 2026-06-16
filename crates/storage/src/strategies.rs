use grand_edge_domain::StrategySignal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::StorageError;

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
}

fn enum_to_string<T: serde::Serialize>(value: &T) -> Result<String, StorageError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}
