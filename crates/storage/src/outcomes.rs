use grand_edge_domain::{Gp, HorizonSecs, Rate, RecommendationId, RecommendationOutcome};
use sqlx::{PgPool, Row};

use crate::StorageError;

#[derive(Clone)]
pub struct OutcomeRepository {
    pool: PgPool,
}

impl OutcomeRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_recommendation_outcome(
        &self,
        outcome: &RecommendationOutcome,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO recommendation_outcomes (
                recommendation_id, evaluated_at, horizon_secs, actual_return, actual_net_gp,
                direction_correct, hit_take_profit, hit_stop_loss,
                max_favourable_excursion, max_adverse_excursion, outcome_label
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8,
                $9, $10, $11
            )
            ON CONFLICT (recommendation_id) DO UPDATE SET
                evaluated_at = EXCLUDED.evaluated_at,
                horizon_secs = EXCLUDED.horizon_secs,
                actual_return = EXCLUDED.actual_return,
                actual_net_gp = EXCLUDED.actual_net_gp,
                direction_correct = EXCLUDED.direction_correct,
                hit_take_profit = EXCLUDED.hit_take_profit,
                hit_stop_loss = EXCLUDED.hit_stop_loss,
                max_favourable_excursion = EXCLUDED.max_favourable_excursion,
                max_adverse_excursion = EXCLUDED.max_adverse_excursion,
                outcome_label = EXCLUDED.outcome_label
            "#,
        )
        .bind(outcome.recommendation_id.0)
        .bind(outcome.evaluated_at)
        .bind(outcome.horizon_secs.0)
        .bind(outcome.actual_return.map(|value| value.get()))
        .bind(outcome.actual_net_gp.map(|value| value.0))
        .bind(outcome.direction_correct)
        .bind(outcome.hit_take_profit)
        .bind(outcome.hit_stop_loss)
        .bind(outcome.max_favourable_excursion.map(|value| value.get()))
        .bind(outcome.max_adverse_excursion.map(|value| value.get()))
        .bind(enum_to_string(&outcome.outcome_label)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_recommendation_outcome(
        &self,
        recommendation_id: RecommendationId,
    ) -> Result<Option<RecommendationOutcome>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT
                recommendation_id,
                evaluated_at,
                horizon_secs,
                actual_return,
                actual_net_gp,
                direction_correct,
                hit_take_profit,
                hit_stop_loss,
                max_favourable_excursion,
                max_adverse_excursion,
                outcome_label
            FROM recommendation_outcomes
            WHERE recommendation_id = $1
            "#,
        )
        .bind(recommendation_id.0)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_recommendation_outcome).transpose()
    }
}

pub(crate) fn row_to_recommendation_outcome(
    row: sqlx::postgres::PgRow,
) -> Result<RecommendationOutcome, StorageError> {
    let outcome_label: String = row.try_get("outcome_label")?;
    Ok(RecommendationOutcome {
        recommendation_id: RecommendationId(row.try_get("recommendation_id")?),
        evaluated_at: row.try_get("evaluated_at")?,
        horizon_secs: HorizonSecs(row.try_get::<i64, _>("horizon_secs")?),
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
    })
}

fn enum_to_string<T: serde::Serialize>(value: &T) -> Result<String, StorageError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}
