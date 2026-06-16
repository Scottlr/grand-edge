use grand_edge_domain::{
    Gp, ItemId, Probability, Rate, Recommendation, RecommendationAction, RecommendationExplanation,
    RecommendationId, RecommendationOutcome, RecommendationPredictionLink, UserId,
};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::StorageError;

#[derive(Clone)]
pub struct RecommendationRepository {
    pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct EvaluatedRecommendationRecord {
    pub recommendation: Recommendation,
    pub outcome: RecommendationOutcome,
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
        self.insert_recommendation_with_links_in_tx(&mut transaction, recommendation, links)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn insert_recommendation_with_links_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        recommendation: &Recommendation,
        links: &[RecommendationPredictionLink],
    ) -> Result<(), StorageError> {
        execute_insert_recommendation(&mut **tx, recommendation).await?;

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
            .execute(&mut **tx)
            .await?;
        }

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

    pub async fn list_pending_outcomes(
        &self,
        limit: i64,
    ) -> Result<Vec<Recommendation>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT
                r.recommendation_id,
                r.user_id,
                r.item_id,
                r.as_of,
                r.action,
                r.score,
                r.prediction_confidence,
                r.execution_confidence,
                r.recommendation_confidence,
                r.expected_net_gp,
                r.expected_roi,
                r.risk_label,
                r.reasons,
                r.explanation
            FROM recommendations r
            LEFT JOIN recommendation_outcomes o
              ON o.recommendation_id = r.recommendation_id
            WHERE o.recommendation_id IS NULL
            ORDER BY r.as_of ASC, r.recommendation_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_recommendation).collect()
    }

    pub async fn list_evaluated_between(
        &self,
        window_start: chrono::DateTime<chrono::Utc>,
        window_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<EvaluatedRecommendationRecord>, StorageError> {
        let recommendation_rows = sqlx::query(
            r#"
            SELECT
                r.recommendation_id,
                r.user_id,
                r.item_id,
                r.as_of,
                r.action,
                r.score,
                r.prediction_confidence,
                r.execution_confidence,
                r.recommendation_confidence,
                r.expected_net_gp,
                r.expected_roi,
                r.risk_label,
                r.reasons,
                r.explanation,
                o.recommendation_id AS outcome_recommendation_id,
                o.evaluated_at,
                o.horizon_secs,
                o.actual_return,
                o.actual_net_gp,
                o.direction_correct,
                o.hit_take_profit,
                o.hit_stop_loss,
                o.max_favourable_excursion,
                o.max_adverse_excursion,
                o.outcome_label
            FROM recommendations r
            INNER JOIN recommendation_outcomes o
              ON o.recommendation_id = r.recommendation_id
            WHERE o.evaluated_at >= $1
              AND o.evaluated_at <= $2
            ORDER BY o.evaluated_at ASC, r.recommendation_id ASC
            "#,
        )
        .bind(window_start)
        .bind(window_end)
        .fetch_all(&self.pool)
        .await?;

        recommendation_rows
            .into_iter()
            .map(row_to_evaluated_recommendation)
            .collect()
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

fn row_to_evaluated_recommendation(
    row: sqlx::postgres::PgRow,
) -> Result<EvaluatedRecommendationRecord, StorageError> {
    let action: String = row.try_get("action")?;
    let explanation = row.try_get::<serde_json::Value, _>("explanation")?;
    let outcome_label: String = row.try_get("outcome_label")?;
    let recommendation = Recommendation {
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
    };
    let outcome = RecommendationOutcome {
        recommendation_id: RecommendationId(row.try_get::<Uuid, _>("outcome_recommendation_id")?),
        evaluated_at: row.try_get("evaluated_at")?,
        horizon_secs: grand_edge_domain::HorizonSecs(row.try_get::<i64, _>("horizon_secs")?),
        actual_return: row
            .try_get::<Option<f64>, _>("actual_return")?
            .map(Rate::new)
            .transpose()?,
        actual_net_gp: row.try_get::<Option<i64>, _>("actual_net_gp")?.map(Gp),
        direction_correct: row.try_get("direction_correct")?,
        hit_take_profit: row.try_get("hit_take_profit")?,
        hit_stop_loss: row.try_get("hit_stop_loss")?,
        max_favourable_excursion: row
            .try_get::<Option<f64>, _>("max_favourable_excursion")?
            .map(Rate::new)
            .transpose()?,
        max_adverse_excursion: row
            .try_get::<Option<f64>, _>("max_adverse_excursion")?
            .map(Rate::new)
            .transpose()?,
        outcome_label: serde_json::from_value(serde_json::Value::String(outcome_label))?,
    };
    Ok(EvaluatedRecommendationRecord {
        recommendation,
        outcome,
    })
}
