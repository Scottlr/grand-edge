use chrono::Utc;
use grand_edge_domain::{UserId, UserPosition};
use sqlx::PgPool;

use crate::StorageError;

#[derive(Clone)]
pub struct PositionRepository {
    pool: PgPool,
}

impl PositionRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_positions(&self, rows: &[UserPosition]) -> Result<u64, StorageError> {
        let mut affected = 0;
        for row in rows {
            let now = Utc::now();
            let result = sqlx::query(
                r#"
                INSERT INTO user_positions (
                    position_id, user_id, item_id, quantity, avg_buy_price, bought_at, notes, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9
                )
                ON CONFLICT (position_id) DO UPDATE SET
                    user_id = EXCLUDED.user_id,
                    item_id = EXCLUDED.item_id,
                    quantity = EXCLUDED.quantity,
                    avg_buy_price = EXCLUDED.avg_buy_price,
                    bought_at = EXCLUDED.bought_at,
                    notes = EXCLUDED.notes,
                    updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(row.position_id.0)
            .bind(row.user_id.0)
            .bind(row.item_id.0)
            .bind(row.quantity.0)
            .bind(row.avg_buy_price.0)
            .bind(row.bought_at)
            .bind(&row.notes)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn active_positions_for_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<UserPosition>, StorageError> {
        let _ = user_id;
        Ok(Vec::new())
    }
}
