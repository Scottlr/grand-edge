use grand_edge_domain::{
    EdgeObservation, GraphDomainError, GraphVersion, ItemGraphEdge, ItemGraphNode, ItemId,
    RecommendationId,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{StorageError, StoredMarketEvent};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationGraphLinkRecord {
    pub link_id: Uuid,
    pub recommendation_id: RecommendationId,
    pub graph_version: String,
    pub edge_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub contribution_weight: Option<f64>,
    pub explanation: serde_json::Value,
}

impl RecommendationGraphLinkRecord {
    pub fn validate(&self) -> Result<(), GraphDomainError> {
        if self.graph_version.trim().is_empty() {
            return Err(GraphDomainError::EmptyGraphVersion);
        }
        if self.edge_id.is_none() && self.event_id.is_none() {
            return Err(GraphDomainError::EmptyField {
                field: "edge_id_or_event_id",
            });
        }
        if let Some(weight) = self.contribution_weight {
            if !weight.is_finite() {
                return Err(GraphDomainError::NonFinite {
                    field: "contribution_weight",
                });
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct GraphRepository {
    pool: PgPool,
}

impl GraphRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_graph_version(&self, version: &GraphVersion) -> Result<(), StorageError> {
        version.validate()?;

        sqlx::query(
            r#"
            INSERT INTO graph_versions (graph_version, source_hash, description, created_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (graph_version) DO NOTHING
            "#,
        )
        .bind(&version.graph_version)
        .bind(&version.source_hash)
        .bind(&version.description)
        .bind(version.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_nodes(&self, nodes: &[ItemGraphNode]) -> Result<u64, StorageError> {
        let mut affected = 0;
        for node in nodes {
            let result = sqlx::query(
                r#"
                INSERT INTO item_graph_nodes (
                    graph_version, item_id, category, metadata, updated_at
                ) VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (graph_version, item_id) DO UPDATE SET
                    category = EXCLUDED.category,
                    metadata = EXCLUDED.metadata,
                    updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(&node.graph_version)
            .bind(node.item_id.0)
            .bind(&node.category)
            .bind(&node.metadata)
            .bind(node.updated_at)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn upsert_edges(&self, edges: &[ItemGraphEdge]) -> Result<u64, StorageError> {
        let mut affected = 0;
        for edge in edges {
            grand_edge_domain::validate_graph_edge(edge)?;
            let result = sqlx::query(
                r#"
                INSERT INTO item_edges (
                    edge_id, graph_version, from_item_id, to_item_id, edge_type, direction, sign,
                    weight, lag_seconds, confidence, source_type, source_ref, formula,
                    requires_review, active, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10, $11, $12, $13,
                    $14, $15, $16, $17
                )
                ON CONFLICT (edge_id) DO UPDATE SET
                    graph_version = EXCLUDED.graph_version,
                    from_item_id = EXCLUDED.from_item_id,
                    to_item_id = EXCLUDED.to_item_id,
                    edge_type = EXCLUDED.edge_type,
                    direction = EXCLUDED.direction,
                    sign = EXCLUDED.sign,
                    weight = EXCLUDED.weight,
                    lag_seconds = EXCLUDED.lag_seconds,
                    confidence = EXCLUDED.confidence,
                    source_type = EXCLUDED.source_type,
                    source_ref = EXCLUDED.source_ref,
                    formula = EXCLUDED.formula,
                    requires_review = EXCLUDED.requires_review,
                    active = EXCLUDED.active,
                    created_at = EXCLUDED.created_at,
                    updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(edge.edge_id)
            .bind(&edge.graph_version)
            .bind(edge.from_item_id.0)
            .bind(edge.to_item_id.0)
            .bind(enum_to_string(&edge.edge_type)?)
            .bind(enum_to_string(&edge.direction)?)
            .bind(edge.sign)
            .bind(edge.weight)
            .bind(edge.lag_seconds)
            .bind(edge.confidence)
            .bind(enum_to_string(&edge.source_type)?)
            .bind(&edge.source_ref)
            .bind(&edge.formula)
            .bind(edge.requires_review)
            .bind(edge.active)
            .bind(edge.created_at)
            .bind(edge.updated_at)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn active_edges_from(
        &self,
        graph_version: &str,
        item_id: ItemId,
    ) -> Result<Vec<ItemGraphEdge>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM item_edges
            WHERE graph_version = $1
              AND from_item_id = $2
              AND active = TRUE
            ORDER BY edge_id ASC
            "#,
        )
        .bind(graph_version)
        .bind(item_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut edges = Vec::with_capacity(rows.len());
        for row in rows {
            edges.push(self.row_to_edge(row).await?);
        }

        Ok(edges)
    }

    pub async fn active_edges_to(
        &self,
        graph_version: &str,
        item_id: ItemId,
    ) -> Result<Vec<ItemGraphEdge>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM item_edges
            WHERE graph_version = $1
              AND to_item_id = $2
              AND active = TRUE
            ORDER BY edge_id ASC
            "#,
        )
        .bind(graph_version)
        .bind(item_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut edges = Vec::with_capacity(rows.len());
        for row in rows {
            edges.push(self.row_to_edge(row).await?);
        }

        Ok(edges)
    }

    pub async fn insert_edge_observations(
        &self,
        observations: &[EdgeObservation],
    ) -> Result<u64, StorageError> {
        let mut affected = 0;
        for observation in observations {
            observation.validate()?;
            let result = sqlx::query(
                r#"
                INSERT INTO edge_observations (
                    edge_id, observed_at, method, window_start, window_end,
                    statistic, p_value, estimated_lag_seconds, estimated_effect, confidence, metadata
                ) VALUES (
                    $1, $2, $3, $4, $5,
                    $6, $7, $8, $9, $10, $11
                )
                ON CONFLICT (edge_id, observed_at, method) DO UPDATE SET
                    window_start = EXCLUDED.window_start,
                    window_end = EXCLUDED.window_end,
                    statistic = EXCLUDED.statistic,
                    p_value = EXCLUDED.p_value,
                    estimated_lag_seconds = EXCLUDED.estimated_lag_seconds,
                    estimated_effect = EXCLUDED.estimated_effect,
                    confidence = EXCLUDED.confidence,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(observation.edge_id)
            .bind(observation.observed_at)
            .bind(enum_to_string(&observation.method)?)
            .bind(observation.window_start)
            .bind(observation.window_end)
            .bind(observation.statistic)
            .bind(observation.p_value)
            .bind(observation.estimated_lag_seconds)
            .bind(observation.estimated_effect)
            .bind(observation.confidence)
            .bind(&observation.metadata)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn latest_edge_observations(
        &self,
        edge_id: Uuid,
    ) -> Result<Vec<EdgeObservation>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT
                edge_id, observed_at, method, window_start, window_end,
                statistic, p_value, estimated_lag_seconds, estimated_effect, confidence, metadata
            FROM edge_observations
            WHERE edge_id = $1
            ORDER BY observed_at DESC, method ASC
            "#,
        )
        .bind(edge_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_observation).collect()
    }

    pub async fn insert_recommendation_graph_links(
        &self,
        links: &[RecommendationGraphLinkRecord],
    ) -> Result<u64, StorageError> {
        let mut affected = 0;
        for link in links {
            link.validate()?;
            let result = sqlx::query(
                r#"
                INSERT INTO recommendation_graph_links (
                    link_id, recommendation_id, graph_version, edge_id, event_id, contribution_weight, explanation
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (link_id) DO UPDATE SET
                    recommendation_id = EXCLUDED.recommendation_id,
                    graph_version = EXCLUDED.graph_version,
                    edge_id = EXCLUDED.edge_id,
                    event_id = EXCLUDED.event_id,
                    contribution_weight = EXCLUDED.contribution_weight,
                    explanation = EXCLUDED.explanation
                "#,
            )
            .bind(link.link_id)
            .bind(link.recommendation_id.0)
            .bind(&link.graph_version)
            .bind(link.edge_id)
            .bind(link.event_id)
            .bind(link.contribution_weight)
            .bind(&link.explanation)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn recommendation_graph_links(
        &self,
        recommendation_id: RecommendationId,
    ) -> Result<Vec<RecommendationGraphLinkRecord>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT
                link_id, recommendation_id, graph_version, edge_id, event_id, contribution_weight, explanation
            FROM recommendation_graph_links
            WHERE recommendation_id = $1
            ORDER BY graph_version ASC, link_id ASC
            "#,
        )
        .bind(recommendation_id.0)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(RecommendationGraphLinkRecord {
                    link_id: row.try_get("link_id")?,
                    recommendation_id: RecommendationId(row.try_get("recommendation_id")?),
                    graph_version: row.try_get("graph_version")?,
                    edge_id: row.try_get("edge_id")?,
                    event_id: row.try_get("event_id")?,
                    contribution_weight: row.try_get("contribution_weight")?,
                    explanation: row.try_get("explanation")?,
                })
            })
            .collect()
    }

    pub async fn insert_event(&self, event: &StoredMarketEvent) -> Result<(), StorageError> {
        crate::MarketEventRepository::new(self.pool.clone())
            .upsert_event(event)
            .await
    }

    async fn row_to_edge(&self, row: sqlx::postgres::PgRow) -> Result<ItemGraphEdge, StorageError> {
        let edge_id: Uuid = row.try_get("edge_id")?;
        Ok(ItemGraphEdge {
            edge_id,
            graph_version: row.try_get("graph_version")?,
            from_item_id: ItemId(row.try_get::<i64, _>("from_item_id")?),
            to_item_id: ItemId(row.try_get::<i64, _>("to_item_id")?),
            edge_type: serde_json::from_value(serde_json::Value::String(
                row.try_get("edge_type")?,
            ))?,
            direction: serde_json::from_value(serde_json::Value::String(
                row.try_get("direction")?,
            ))?,
            sign: row.try_get("sign")?,
            weight: row.try_get("weight")?,
            lag_seconds: row.try_get("lag_seconds")?,
            confidence: row.try_get("confidence")?,
            source_type: serde_json::from_value(serde_json::Value::String(
                row.try_get("source_type")?,
            ))?,
            source_ref: row.try_get("source_ref")?,
            observations: self.latest_edge_observations(edge_id).await?,
            formula: row.try_get("formula")?,
            requires_review: row.try_get("requires_review")?,
            active: row.try_get("active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

pub(crate) fn row_to_observation(
    row: sqlx::postgres::PgRow,
) -> Result<EdgeObservation, StorageError> {
    let method: String = row.try_get("method")?;
    Ok(EdgeObservation {
        edge_id: row.try_get("edge_id")?,
        observed_at: row.try_get("observed_at")?,
        method: serde_json::from_value(serde_json::Value::String(method))?,
        window_start: row.try_get("window_start")?,
        window_end: row.try_get("window_end")?,
        statistic: row.try_get("statistic")?,
        p_value: row.try_get("p_value")?,
        estimated_lag_seconds: row.try_get("estimated_lag_seconds")?,
        estimated_effect: row.try_get("estimated_effect")?,
        confidence: row.try_get("confidence")?,
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::RecommendationGraphLinkRecord;
    use grand_edge_domain::RecommendationId;

    #[test]
    fn recommendation_graph_link_requires_edge_or_event() {
        let result = RecommendationGraphLinkRecord {
            link_id: Uuid::new_v4(),
            recommendation_id: RecommendationId(Uuid::new_v4()),
            graph_version: "graph_v1".to_string(),
            edge_id: None,
            event_id: None,
            contribution_weight: Some(0.5),
            explanation: serde_json::json!({}),
        }
        .validate();

        assert!(result.is_err());
    }
}
