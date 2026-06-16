use grand_edge_domain::{Gp, ModelVersion, Probability, Rate, ReasonOutcomeSummary, ReasonType};
use sqlx::{PgPool, Row};

use crate::StorageError;

#[derive(Clone)]
pub struct ReasonOutcomeRepository {
    pool: PgPool,
}

impl ReasonOutcomeRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_reason_outcome(
        &self,
        summary: &ReasonOutcomeSummary,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO reason_outcomes (
                reason_type, reason_key, model_version, recommendation_action,
                execution_mode, confidence_bucket, window_start, window_end,
                sample_size, publishable, win_rate, avg_actual_return, avg_net_gp, calibration_error
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14
            )
            ON CONFLICT (
                reason_type, reason_key, model_version, recommendation_action,
                execution_mode, confidence_bucket, window_start, window_end
            )
            DO UPDATE SET
                sample_size = EXCLUDED.sample_size,
                publishable = EXCLUDED.publishable,
                win_rate = EXCLUDED.win_rate,
                avg_actual_return = EXCLUDED.avg_actual_return,
                avg_net_gp = EXCLUDED.avg_net_gp,
                calibration_error = EXCLUDED.calibration_error
            "#,
        )
        .bind(enum_to_string(&summary.reason_type)?)
        .bind(&summary.reason_key)
        .bind(&summary.model_version.0)
        .bind(enum_to_string(&summary.recommendation_action)?)
        .bind(optional_key(
            summary
                .execution_mode
                .map(|value| enum_to_string(&value))
                .transpose()?,
        ))
        .bind(optional_key(summary.confidence_bucket.as_deref()))
        .bind(summary.window_start)
        .bind(summary.window_end)
        .bind(summary.sample_size)
        .bind(summary.publishable)
        .bind(summary.win_rate.map(|value| value.get()))
        .bind(summary.avg_actual_return.map(|value| value.get()))
        .bind(summary.avg_net_gp.map(|value| value.0))
        .bind(summary.calibration_error)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_reason_outcome_summaries(
        &self,
        summaries: &[ReasonOutcomeSummary],
    ) -> Result<u64, StorageError> {
        let mut affected = 0;
        for summary in summaries {
            let result = execute_upsert_reason_outcome(&self.pool, summary).await?;
            affected += result.rows_affected();
        }

        Ok(affected)
    }

    pub async fn list_reason_outcomes(
        &self,
        reason_type: ReasonType,
        reason_key: &str,
        model_version: &str,
    ) -> Result<Vec<ReasonOutcomeSummary>, StorageError> {
        let rows = sqlx::query(
            r#"
            SELECT
                reason_type,
                reason_key,
                model_version,
                recommendation_action,
                execution_mode,
                confidence_bucket,
                window_start,
                window_end,
                sample_size,
                publishable,
                win_rate,
                avg_actual_return,
                avg_net_gp,
                calibration_error
            FROM reason_outcomes
            WHERE reason_type = $1
              AND reason_key = $2
              AND model_version = $3
            ORDER BY window_end DESC
            "#,
        )
        .bind(enum_to_string(&reason_type)?)
        .bind(reason_key)
        .bind(model_version)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_reason_outcome).collect()
    }
}

fn row_to_reason_outcome(row: sqlx::postgres::PgRow) -> Result<ReasonOutcomeSummary, StorageError> {
    let reason_type: String = row.try_get("reason_type")?;
    let recommendation_action: String = row.try_get("recommendation_action")?;
    let execution_mode = optional_key_to_option(row.try_get::<String, _>("execution_mode")?);
    let confidence_bucket = optional_key_to_option(row.try_get::<String, _>("confidence_bucket")?);
    Ok(ReasonOutcomeSummary {
        reason_type: serde_json::from_value(serde_json::Value::String(reason_type))?,
        reason_key: row.try_get("reason_key")?,
        model_version: ModelVersion::new(row.try_get::<String, _>("model_version")?)?,
        recommendation_action: serde_json::from_value(serde_json::Value::String(
            recommendation_action,
        ))?,
        execution_mode: execution_mode
            .map(|value| serde_json::from_value(serde_json::Value::String(value)))
            .transpose()?,
        confidence_bucket,
        window_start: row.try_get("window_start")?,
        window_end: row.try_get("window_end")?,
        sample_size: row.try_get("sample_size")?,
        publishable: row.try_get("publishable")?,
        win_rate: row
            .try_get::<Option<f64>, _>("win_rate")?
            .map(Probability::new)
            .transpose()?,
        avg_actual_return: row
            .try_get::<Option<f64>, _>("avg_actual_return")?
            .map(Rate::new)
            .transpose()?,
        avg_net_gp: row.try_get::<Option<i64>, _>("avg_net_gp")?.map(Gp),
        calibration_error: row.try_get("calibration_error")?,
    })
}

async fn execute_upsert_reason_outcome<'a, E>(
    executor: E,
    summary: &ReasonOutcomeSummary,
) -> Result<sqlx::postgres::PgQueryResult, StorageError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    Ok(sqlx::query(
        r#"
        INSERT INTO reason_outcomes (
            reason_type, reason_key, model_version, recommendation_action,
            execution_mode, confidence_bucket, window_start, window_end,
            sample_size, publishable, win_rate, avg_actual_return, avg_net_gp, calibration_error
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7, $8,
            $9, $10, $11, $12, $13, $14
        )
        ON CONFLICT (
            reason_type, reason_key, model_version, recommendation_action,
            execution_mode, confidence_bucket, window_start, window_end
        )
        DO UPDATE SET
            sample_size = EXCLUDED.sample_size,
            publishable = EXCLUDED.publishable,
            win_rate = EXCLUDED.win_rate,
            avg_actual_return = EXCLUDED.avg_actual_return,
            avg_net_gp = EXCLUDED.avg_net_gp,
            calibration_error = EXCLUDED.calibration_error
        "#,
    )
    .bind(enum_to_string(&summary.reason_type)?)
    .bind(&summary.reason_key)
    .bind(&summary.model_version.0)
    .bind(enum_to_string(&summary.recommendation_action)?)
    .bind(optional_key(
        summary
            .execution_mode
            .map(|value| enum_to_string(&value))
            .transpose()?,
    ))
    .bind(optional_key(summary.confidence_bucket.as_deref()))
    .bind(summary.window_start)
    .bind(summary.window_end)
    .bind(summary.sample_size)
    .bind(summary.publishable)
    .bind(summary.win_rate.map(|value| value.get()))
    .bind(summary.avg_actual_return.map(|value| value.get()))
    .bind(summary.avg_net_gp.map(|value| value.0))
    .bind(summary.calibration_error)
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

fn optional_key(value: Option<impl AsRef<str>>) -> String {
    value
        .map(|value| value.as_ref().to_string())
        .unwrap_or_default()
}

fn optional_key_to_option(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}
