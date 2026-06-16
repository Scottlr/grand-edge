use chrono::{DateTime, Utc};
use grand_edge_domain::{Gp, ItemId, PositionId, Quantity, UserId, UserPosition};
use sqlx::{PgPool, Row};
use uuid::Uuid;

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
        let rows = sqlx::query(
            r#"
            SELECT position_id, user_id, item_id, quantity, avg_buy_price, bought_at, notes
            FROM user_positions
            WHERE user_id = $1
            ORDER BY updated_at DESC
            "#,
        )
        .bind(user_id.0)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_position).collect()
    }

    pub async fn get_position(
        &self,
        position_id: PositionId,
    ) -> Result<Option<UserPosition>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT position_id, user_id, item_id, quantity, avg_buy_price, bought_at, notes
            FROM user_positions
            WHERE position_id = $1
            LIMIT 1
            "#,
        )
        .bind(position_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_position).transpose()
    }

    pub async fn active_position_for_user_item(
        &self,
        user_id: UserId,
        item_id: ItemId,
    ) -> Result<Option<UserPosition>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT position_id, user_id, item_id, quantity, avg_buy_price, bought_at, notes
            FROM user_positions
            WHERE user_id = $1 AND item_id = $2
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id.0)
        .bind(item_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_position).transpose()
    }
}

fn row_to_position(row: sqlx::postgres::PgRow) -> Result<UserPosition, StorageError> {
    Ok(UserPosition {
        position_id: PositionId(row.try_get::<Uuid, _>("position_id")?),
        user_id: UserId(row.try_get::<Uuid, _>("user_id")?),
        item_id: ItemId(row.try_get::<i64, _>("item_id")?),
        quantity: Quantity::try_from(row.try_get::<i64, _>("quantity")?)?,
        avg_buy_price: Gp::try_from(row.try_get::<i64, _>("avg_buy_price")?)?,
        bought_at: row.try_get::<Option<DateTime<Utc>>, _>("bought_at")?,
        notes: row.try_get("notes")?,
    })
}
