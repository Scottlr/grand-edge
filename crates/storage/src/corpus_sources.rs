use chrono::{DateTime, Utc};
use grand_edge_domain::CorpusSourceEntry;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::StorageError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredCorpusSource {
    pub source: CorpusSourceEntry,
    pub metadata: serde_json::Value,
}

#[derive(Clone)]
pub struct CorpusSourceRepository {
    pool: PgPool,
}

impl CorpusSourceRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_sources(&self, rows: &[StoredCorpusSource]) -> Result<u64, StorageError> {
        let mut affected = 0;
        for row in rows {
            row.source.validate()?;
            let result = sqlx::query(
                r#"
                INSERT INTO corpus_sources (
                    source_id, source_type, title, url, retrieved_at, license_note, content_hash, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (source_id) DO UPDATE SET
                    source_type = EXCLUDED.source_type,
                    title = EXCLUDED.title,
                    url = EXCLUDED.url,
                    retrieved_at = EXCLUDED.retrieved_at,
                    license_note = EXCLUDED.license_note,
                    content_hash = EXCLUDED.content_hash,
                    metadata = EXCLUDED.metadata
                "#,
            )
            .bind(&row.source.source_id)
            .bind(enum_to_string(&row.source.source_type)?)
            .bind(&row.source.title)
            .bind(&row.source.url)
            .bind(row.source.retrieved_at)
            .bind(&row.source.license_note)
            .bind(&row.source.content_hash)
            .bind(&row.metadata)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn get_source(
        &self,
        source_id: &str,
    ) -> Result<Option<StoredCorpusSource>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT
                source_id,
                source_type,
                title,
                url,
                retrieved_at,
                license_note,
                content_hash,
                metadata
            FROM corpus_sources
            WHERE source_id = $1
            "#,
        )
        .bind(source_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_corpus_source).transpose()
    }
}

pub(crate) fn row_to_corpus_source(
    row: sqlx::postgres::PgRow,
) -> Result<StoredCorpusSource, StorageError> {
    let source_type: String = row.try_get("source_type")?;
    Ok(StoredCorpusSource {
        source: CorpusSourceEntry {
            source_id: row.try_get("source_id")?,
            title: row.try_get("title")?,
            url: row.try_get("url")?,
            retrieved_at: row.try_get::<Option<DateTime<Utc>>, _>("retrieved_at")?,
            license_note: row.try_get("license_note")?,
            content_hash: row.try_get("content_hash")?,
            source_type: serde_json::from_value(serde_json::Value::String(source_type))?,
        },
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
