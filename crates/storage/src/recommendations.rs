use grand_edge_domain::{
    Gp, ItemId, Probability, Rate, Recommendation, RecommendationAction, RecommendationExplanation,
    RecommendationId, RecommendationPredictionLink, UserId,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

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
            let result = execute_insert_recommendation(&self.pool, row).await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn insert_recommendation_with_links(
        &self,
        recommendation: &Recommendation,
        links: &[RecommendationPredictionLink],
    ) -> Result<(), StorageError> {
        let mut transaction = self.pool.begin().await?;
        execute_insert_recommendation(&mut *transaction, recommendation).await?;

        for link in links {
            let link = RecommendationPredictionLink::new(
                link.recommendation_id,
                link.prediction_id,
                link.contribution_weight,
            )?;
            sqlx::query(
                r#"
                INSERT INTO recommendation_prediction_links (
                    recommendation_id,
                    prediction_id,
                    contribution_weight
                ) VALUES ($1, $2, $3)
                ON CONFLICT (recommendation_id, prediction_id) DO UPDATE SET
                    contribution_weight = EXCLUDED.contribution_weight
                "#,
            )
            .bind(link.recommendation_id.0)
            .bind(link.prediction_id.0)
            .bind(link.contribution_weight)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;
        Ok(())
    }

    pub async fn list_recent_for_user(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> Result<Vec<Recommendation>, StorageError> {
        self.list_recent(Some(user_id), None, limit, 0).await
    }

    pub async fn list_recent(
        &self,
        user_id: Option<UserId>,
        action: Option<RecommendationAction>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Recommendation>, StorageError> {
        let action = action.map(|value| enum_to_string(&value)).transpose()?;
        let rows = sqlx::query(
            r#"
            SELECT
                recommendation_id,
                user_id,
                item_id,
                as_of,
                action,
                score,
                prediction_confidence,
                execution_confidence,
                recommendation_confidence,
                expected_net_gp,
                expected_roi,
                risk_label,
                reasons,
                explanation
            FROM recommendations
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR action = $2)
            ORDER BY as_of DESC, recommendation_id DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(user_id.map(|value| value.0))
        .bind(action)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_recommendation).collect()
    }

    pub async fn get_recommendation(
        &self,
        recommendation_id: RecommendationId,
    ) -> Result<Option<Recommendation>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT
                recommendation_id,
                user_id,
                item_id,
                as_of,
                action,
                score,
                prediction_confidence,
                execution_confidence,
                recommendation_confidence,
                expected_net_gp,
                expected_roi,
                risk_label,
                reasons,
                explanation
            FROM recommendations
            WHERE recommendation_id = $1
            "#,
        )
        .bind(recommendation_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_recommendation).transpose()
    }
}

pub(crate) fn row_to_recommendation(
    row: sqlx::postgres::PgRow,
) -> Result<Recommendation, StorageError> {
    let action: String = row.try_get("action")?;
    let explanation = row.try_get::<serde_json::Value, _>("explanation")?;

    Ok(Recommendation {
        recommendation_id: RecommendationId(row.try_get::<Uuid, _>("recommendation_id")?),
        user_id: row.try_get::<Option<Uuid>, _>("user_id")?.map(UserId),
        item_id: ItemId(row.try_get::<i64, _>("item_id")?),
        as_of: row.try_get("as_of")?,
        action: serde_json::from_value(serde_json::Value::String(action))?,
        score: Rate::new(row.try_get::<f64, _>("score")?)?,
        prediction_confidence: row
            .try_get::<Option<f64>, _>("prediction_confidence")?
            .map(Probability::new)
            .transpose()?,
        execution_confidence: row
            .try_get::<Option<f64>, _>("execution_confidence")?
            .map(Probability::new)
            .transpose()?,
        recommendation_confidence: Probability::new(
            row.try_get::<f64, _>("recommendation_confidence")?,
        )?,
        expected_net_gp: row.try_get::<Option<i64>, _>("expected_net_gp")?.map(Gp),
        expected_roi: row
            .try_get::<Option<f64>, _>("expected_roi")?
            .map(Rate::new)
            .transpose()?,
        risk_label: row.try_get("risk_label")?,
        reasons: serde_json::from_value(row.try_get("reasons")?)?,
        explanation: serde_json::from_value::<RecommendationExplanation>(explanation)?,
    })
}

async fn execute_insert_recommendation<'a, E>(
    executor: E,
    row: &Recommendation,
) -> Result<sqlx::postgres::PgQueryResult, StorageError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    Ok(sqlx::query(
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
    .execute(executor)
    .await?)
}

fn enum_to_string<T: serde::Serialize>(value: &T) -> Result<String, StorageError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}
