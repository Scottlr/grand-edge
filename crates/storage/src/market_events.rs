use grand_edge_domain::{GraphDomainError, ItemId, MarketEventNode, Probability};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::StorageError;

#[derive(Debug, Clone, PartialEq)]
pub struct MarketEventItemLink {
    pub item_id: ItemId,
    pub relation: String,
    pub confidence: Probability,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredMarketEvent {
    pub event: MarketEventNode,
    pub item_links: Vec<MarketEventItemLink>,
}

#[derive(Clone)]
pub struct MarketEventRepository {
    pool: PgPool,
}

impl MarketEventRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_event(&self, row: &StoredMarketEvent) -> Result<(), StorageError> {
        row.event.validate()?;

        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO market_events (
                event_id, graph_version, event_type, title, occurred_at, source_ref, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (event_id) DO UPDATE SET
                graph_version = EXCLUDED.graph_version,
                event_type = EXCLUDED.event_type,
                title = EXCLUDED.title,
                occurred_at = EXCLUDED.occurred_at,
                source_ref = EXCLUDED.source_ref,
                metadata = EXCLUDED.metadata
            "#,
        )
        .bind(row.event.event_id)
        .bind(&row.event.graph_version)
        .bind(enum_to_string(&row.event.event_type)?)
        .bind(&row.event.title)
        .bind(row.event.occurred_at)
        .bind(&row.event.source_ref)
        .bind(&row.event.metadata)
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM market_event_items WHERE event_id = $1")
            .bind(row.event.event_id)
            .execute(&mut *tx)
            .await?;

        for link in &row.item_links {
            if link.relation.trim().is_empty() {
                return Err(StorageError::GraphDomainValidation(
                    GraphDomainError::EmptyField { field: "relation" },
                ));
            }

            sqlx::query(
                r#"
                INSERT INTO market_event_items (
                    event_id, item_id, relation, confidence
                ) VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(row.event.event_id)
            .bind(link.item_id.0)
            .bind(&link.relation)
            .bind(link.confidence.get())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_event(
        &self,
        event_id: Uuid,
    ) -> Result<Option<StoredMarketEvent>, StorageError> {
        let event_row = sqlx::query(
            r#"
            SELECT
                event_id,
                graph_version,
                event_type,
                title,
                occurred_at,
                source_ref,
                metadata
            FROM market_events
            WHERE event_id = $1
            "#,
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(event_row) = event_row else {
            return Ok(None);
        };

        let item_rows = sqlx::query(
            r#"
            SELECT item_id, relation, confidence
            FROM market_event_items
            WHERE event_id = $1
            ORDER BY item_id ASC, relation ASC
            "#,
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(StoredMarketEvent {
            event: row_to_market_event(
                event_row,
                item_rows
                    .iter()
                    .map(|row| ItemId(row.get::<i64, _>("item_id")))
                    .collect(),
            )?,
            item_links: item_rows
                .into_iter()
                .map(|row| {
                    Ok(MarketEventItemLink {
                        item_id: ItemId(row.try_get::<i64, _>("item_id")?),
                        relation: row.try_get("relation")?,
                        confidence: Probability::new(row.try_get::<f64, _>("confidence")?)?,
                    })
                })
                .collect::<Result<Vec<_>, StorageError>>()?,
        }))
    }
}

fn row_to_market_event(
    row: sqlx::postgres::PgRow,
    affected_item_ids: Vec<ItemId>,
) -> Result<MarketEventNode, StorageError> {
    let event_type: String = row.try_get("event_type")?;
    Ok(MarketEventNode {
        event_id: row.try_get("event_id")?,
        graph_version: row.try_get("graph_version")?,
        event_type: serde_json::from_value(serde_json::Value::String(event_type))?,
        title: row.try_get("title")?,
        occurred_at: row.try_get("occurred_at")?,
        source_ref: row.try_get("source_ref")?,
        affected_item_ids,
        metadata: row.try_get("metadata")?,
    })
}

fn enum_to_string<T: serde::Serialize>(value: &T) -> Result<String, StorageError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}
