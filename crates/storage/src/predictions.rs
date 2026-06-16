use grand_edge_domain::{Prediction, PredictionId, PredictionInterval};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::StorageError;

#[derive(Clone)]
pub struct PredictionRepository {
    pool: PgPool,
}

impl PredictionRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_predictions(
        &self,
        predictions: &[Prediction],
    ) -> Result<u64, StorageError> {
        let mut affected = 0;
        for prediction in predictions {
            affected += insert_prediction_row(&self.pool, prediction).await?;
        }
        Ok(affected)
    }

    pub async fn insert_predictions_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        predictions: &[Prediction],
    ) -> Result<u64, StorageError> {
        let mut affected = 0;
        for prediction in predictions {
            affected += insert_prediction_row(&mut **tx, prediction).await?;
        }
        Ok(affected)
    }

    pub async fn predictions_for_feature_snapshot(
        &self,
        feature_snapshot_id: Uuid,
    ) -> Result<Vec<Prediction>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT
                prediction_id,
                feature_snapshot_id,
                item_id,
                as_of,
                horizon_secs,
                model_id,
                model_version,
                predicted_direction,
                predicted_return,
                confidence,
                prediction_interval_low,
                prediction_interval_high,
                explanation,
                created_at
            FROM predictions
            WHERE feature_snapshot_id = $1
            ORDER BY created_at ASC, prediction_id ASC
            "#,
        )
        .bind(feature_snapshot_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_prediction).collect()
    }
}

async fn insert_prediction_row<'a, E>(
    executor: E,
    prediction: &Prediction,
) -> Result<u64, StorageError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    prediction.validate_feature_snapshot_id()?;
    let result = sqlx::query(
        r#"
                INSERT INTO predictions (
                    prediction_id, feature_snapshot_id, item_id, as_of, horizon_secs,
                    model_id, model_version, predicted_direction, predicted_return,
                    confidence, prediction_interval_low, prediction_interval_high,
                    explanation, created_at
                ) VALUES (
                    $1, $2, $3, $4, $5,
                    $6, $7, $8, $9,
                    $10, $11, $12,
                    $13, $14
                )
                ON CONFLICT (prediction_id) DO UPDATE SET
                    feature_snapshot_id = EXCLUDED.feature_snapshot_id,
                    item_id = EXCLUDED.item_id,
                    as_of = EXCLUDED.as_of,
                    horizon_secs = EXCLUDED.horizon_secs,
                    model_id = EXCLUDED.model_id,
                    model_version = EXCLUDED.model_version,
                    predicted_direction = EXCLUDED.predicted_direction,
                    predicted_return = EXCLUDED.predicted_return,
                    confidence = EXCLUDED.confidence,
                    prediction_interval_low = EXCLUDED.prediction_interval_low,
                    prediction_interval_high = EXCLUDED.prediction_interval_high,
                    explanation = EXCLUDED.explanation,
                    created_at = EXCLUDED.created_at
                "#,
    )
    .bind(prediction.prediction_id.0)
    .bind(prediction.feature_snapshot_id)
    .bind(prediction.item_id.0)
    .bind(prediction.as_of)
    .bind(prediction.horizon_secs.0)
    .bind(&prediction.model_id.0)
    .bind(&prediction.model_version.0)
    .bind(enum_to_string(&prediction.predicted_direction)?)
    .bind(prediction.predicted_return.map(|value| value.get()))
    .bind(prediction.confidence.get())
    .bind(
        prediction
            .prediction_interval
            .as_ref()
            .and_then(|value| value.low)
            .map(|value| value.get()),
    )
    .bind(
        prediction
            .prediction_interval
            .as_ref()
            .and_then(|value| value.high)
            .map(|value| value.get()),
    )
    .bind(&prediction.explanation)
    .bind(prediction.created_at)
    .execute(executor)
    .await?;
    Ok(result.rows_affected())
}

pub(crate) fn row_to_prediction(row: sqlx::postgres::PgRow) -> Result<Prediction, StorageError> {
    let direction: String = row.try_get("predicted_direction")?;
    Ok(Prediction {
        prediction_id: PredictionId(row.try_get("prediction_id")?),
        feature_snapshot_id: row.try_get("feature_snapshot_id")?,
        item_id: grand_edge_domain::ItemId(row.try_get::<i64, _>("item_id")?),
        as_of: row.try_get("as_of")?,
        horizon_secs: grand_edge_domain::HorizonSecs(row.try_get::<i64, _>("horizon_secs")?),
        model_id: grand_edge_domain::StrategyId::new(row.try_get::<String, _>("model_id")?)?,
        model_version: grand_edge_domain::ModelVersion::new(
            row.try_get::<String, _>("model_version")?,
        )?,
        predicted_direction: serde_json::from_value(serde_json::Value::String(direction))?,
        predicted_return: row
            .try_get::<Option<f64>, _>("predicted_return")?
            .map(grand_edge_domain::Rate::new)
            .transpose()?,
        confidence: grand_edge_domain::Probability::new(row.try_get::<f64, _>("confidence")?)?,
        prediction_interval: prediction_interval_from_row(&row)?,
        explanation: row.try_get("explanation")?,
        created_at: row.try_get("created_at")?,
    })
}

fn prediction_interval_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<PredictionInterval>, StorageError> {
    let low = row
        .try_get::<Option<f64>, _>("prediction_interval_low")?
        .map(grand_edge_domain::Rate::new)
        .transpose()?;
    let high = row
        .try_get::<Option<f64>, _>("prediction_interval_high")?
        .map(grand_edge_domain::Rate::new)
        .transpose()?;

    if low.is_none() && high.is_none() {
        return Ok(None);
    }

    Ok(Some(PredictionInterval { low, high }))
}

fn enum_to_string<T: serde::Serialize>(value: &T) -> Result<String, StorageError> {
    let value = serde_json::to_value(value)?;
    Ok(value
        .as_str()
        .expect("serde rename_all enums serialize to string")
        .to_string())
}
