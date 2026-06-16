use grand_edge_domain::{Recommendation, UserId};
use sqlx::PgPool;

use crate::StorageError;

#[derive(Clone)]
pub struct RecommendationRepository {
    pool: PgPool,
}

impl RecommendationRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_recommendations(
        &self,
        rows: &[Recommendation],
    ) -> Result<u64, StorageError> {
        let mut affected = 0;
        for row in rows {
            let result = sqlx::query(
                r#"
                INSERT INTO recommendations (
                    recommendation_id, user_id, item_id, as_of, action, score,
                    prediction_confidence, execution_confidence, recommendation_confidence,
                    expected_net_gp, expected_roi, risk_label, reasons, explanation
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9,
                    $10, $11, $12, $13, $14
                )
                ON CONFLICT (recommendation_id) DO UPDATE SET
                    user_id = EXCLUDED.user_id,
                    item_id = EXCLUDED.item_id,
                    as_of = EXCLUDED.as_of,
                    action = EXCLUDED.action,
                    score = EXCLUDED.score,
                    prediction_confidence = EXCLUDED.prediction_confidence,
                    execution_confidence = EXCLUDED.execution_confidence,
                    recommendation_confidence = EXCLUDED.recommendation_confidence,
                    expected_net_gp = EXCLUDED.expected_net_gp,
                    expected_roi = EXCLUDED.expected_roi,
                    risk_label = EXCLUDED.risk_label,
                    reasons = EXCLUDED.reasons,
                    explanation = EXCLUDED.explanation
                "#,
            )
            .bind(row.recommendation_id.0)
            .bind(row.user_id.map(|value| value.0))
            .bind(row.item_id.0)
            .bind(row.as_of)
            .bind(enum_to_string(&row.action)?)
            .bind(row.score.get())
            .bind(row.prediction_confidence.map(|value| value.get()))
            .bind(row.execution_confidence.map(|value| value.get()))
            .bind(row.recommendation_confidence.get())
            .bind(row.expected_net_gp.map(|value| value.0))
            .bind(row.expected_roi.map(|value| value.get()))
            .bind(&row.risk_label)
            .bind(serde_json::to_value(&row.reasons)?)
            .bind(serde_json::to_value(&row.explanation)?)
            .execute(&self.pool)
            .await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn list_recent_for_user(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<Recommendation>, StorageError> {
        let _ = (user_id, limit);
        Ok(Vec::new())
    }
}

fn enum_to_string<T: serde::Serialize>(value: &T) -> Result<String, StorageError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}
