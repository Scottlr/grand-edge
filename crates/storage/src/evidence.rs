use grand_edge_domain::{
    FeatureSnapshot, Prediction, PredictionId, PredictionInterval, Recommendation,
    RecommendationExplanation, RecommendationId, RecommendationOutcome,
    StructuredRecommendationExplanation,
};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::StorageError;

#[derive(Debug, Clone, PartialEq)]
pub struct RecommendationEvidenceRecord {
    pub recommendation: Recommendation,
    pub linked_predictions: Vec<LinkedPredictionRecord>,
    pub outcome: Option<RecommendationOutcome>,
    pub graph: Option<RecommendationGraphEvidence>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkedPredictionRecord {
    pub prediction: Prediction,
    pub feature_snapshot: FeatureSnapshot,
    pub contribution_weight: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecommendationGraphEvidence {
    pub graph_version: String,
    pub graph_links: Vec<RecommendationGraphLinkSummary>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecommendationGraphLinkSummary {
    pub relation_type: String,
    pub source_item_id: i64,
    pub target_item_id: i64,
    pub edge_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub weight: Option<f64>,
    pub explanation: serde_json::Value,
}

#[derive(Clone)]
pub struct EvidenceRepository {
    pool: PgPool,
}

impl EvidenceRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_feature_snapshot(
        &self,
        snapshot: &FeatureSnapshot,
    ) -> Result<(), StorageError> {
        execute_insert_feature_snapshot(&self.pool, snapshot).await?;
        Ok(())
    }

    pub async fn insert_feature_snapshot_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        snapshot: &FeatureSnapshot,
    ) -> Result<(), StorageError> {
        execute_insert_feature_snapshot(&mut **tx, snapshot).await?;
        Ok(())
    }

    pub async fn get_feature_snapshot(
        &self,
        id: Uuid,
    ) -> Result<Option<FeatureSnapshot>, StorageError> {
        let row = sqlx::query(
            r#"
            SELECT
                feature_snapshot_id,
                item_id,
                as_of,
                feature_set_version,
                graph_version,
                source_window_start,
                source_window_end,
                features,
                created_at
            FROM feature_snapshots
            WHERE feature_snapshot_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_feature_snapshot).transpose()
    }

    pub async fn evidence_for_recommendation(
        &self,
        recommendation_id: RecommendationId,
    ) -> Result<Option<RecommendationEvidenceRecord>, StorageError> {
        let recommendation = sqlx::query(
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

        let Some(row) = recommendation else {
            return Ok(None);
        };
        let recommendation = crate::recommendations::row_to_recommendation(row)?;

        let link_rows = sqlx::query(
            r#"
            SELECT
                p.prediction_id,
                p.feature_snapshot_id,
                p.item_id,
                p.as_of,
                p.horizon_secs,
                p.model_id,
                p.model_version,
                p.predicted_direction,
                p.predicted_return,
                p.confidence,
                p.prediction_interval_low,
                p.prediction_interval_high,
                p.explanation,
                p.created_at,
                fs.item_id AS fs_item_id,
                fs.as_of AS fs_as_of,
                fs.feature_set_version,
                fs.graph_version,
                fs.source_window_start,
                fs.source_window_end,
                fs.features,
                fs.created_at AS fs_created_at,
                l.contribution_weight
            FROM recommendation_prediction_links l
            INNER JOIN predictions p
                ON p.prediction_id = l.prediction_id
            INNER JOIN feature_snapshots fs
                ON fs.feature_snapshot_id = p.feature_snapshot_id
            WHERE l.recommendation_id = $1
            ORDER BY p.created_at ASC, p.prediction_id ASC
            "#,
        )
        .bind(recommendation_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut linked_predictions = Vec::with_capacity(link_rows.len());
        for row in link_rows {
            linked_predictions.push(row_to_linked_prediction(row)?);
        }

        let outcome = sqlx::query(
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

        let graph_links = sqlx::query(
            r#"
            SELECT
                edge_id,
                event_id,
                contribution_weight,
                explanation
            FROM recommendation_graph_links
            WHERE recommendation_id = $1
            ORDER BY graph_version ASC, link_id ASC
            "#,
        )
        .bind(recommendation_id.0)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(RecommendationEvidenceRecord {
            graph: graph_from_explanation(&recommendation.explanation, graph_links),
            recommendation,
            linked_predictions,
            outcome: outcome
                .map(crate::outcomes::row_to_recommendation_outcome)
                .transpose()?,
        }))
    }
}

async fn execute_insert_feature_snapshot<'a, E>(
    executor: E,
    snapshot: &FeatureSnapshot,
) -> Result<sqlx::postgres::PgQueryResult, StorageError>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    Ok(sqlx::query(
        r#"
        INSERT INTO feature_snapshots (
            feature_snapshot_id, item_id, as_of, feature_set_version, graph_version,
            source_window_start, source_window_end, features, created_at
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9
        )
        ON CONFLICT (feature_snapshot_id) DO UPDATE SET
            item_id = EXCLUDED.item_id,
            as_of = EXCLUDED.as_of,
            feature_set_version = EXCLUDED.feature_set_version,
            graph_version = EXCLUDED.graph_version,
            source_window_start = EXCLUDED.source_window_start,
            source_window_end = EXCLUDED.source_window_end,
            features = EXCLUDED.features,
            created_at = EXCLUDED.created_at
        "#,
    )
    .bind(snapshot.feature_snapshot_id)
    .bind(snapshot.item_id.0)
    .bind(snapshot.as_of)
    .bind(&snapshot.feature_set_version)
    .bind(&snapshot.graph_version)
    .bind(snapshot.source_window_start)
    .bind(snapshot.source_window_end)
    .bind(serde_json::Value::Object(snapshot.features.clone()))
    .bind(snapshot.created_at)
    .execute(executor)
    .await?)
}

pub(crate) fn row_to_feature_snapshot(
    row: sqlx::postgres::PgRow,
) -> Result<FeatureSnapshot, StorageError> {
    let features: serde_json::Value = row.try_get("features")?;
    Ok(FeatureSnapshot {
        feature_snapshot_id: row.try_get("feature_snapshot_id")?,
        item_id: grand_edge_domain::ItemId(row.try_get::<i64, _>("item_id")?),
        as_of: row.try_get("as_of")?,
        feature_set_version: row.try_get("feature_set_version")?,
        graph_version: row.try_get("graph_version")?,
        source_window_start: row.try_get("source_window_start")?,
        source_window_end: row.try_get("source_window_end")?,
        features: features.as_object().cloned().unwrap_or_default(),
        created_at: row.try_get("created_at")?,
    })
}

fn row_to_linked_prediction(
    row: sqlx::postgres::PgRow,
) -> Result<LinkedPredictionRecord, StorageError> {
    let direction: String = row.try_get("predicted_direction")?;
    let prediction = Prediction {
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
    };
    let features: serde_json::Value = row.try_get("features")?;
    let feature_snapshot = FeatureSnapshot {
        feature_snapshot_id: prediction.feature_snapshot_id,
        item_id: grand_edge_domain::ItemId(row.try_get::<i64, _>("fs_item_id")?),
        as_of: row.try_get("fs_as_of")?,
        feature_set_version: row.try_get("feature_set_version")?,
        graph_version: row.try_get("graph_version")?,
        source_window_start: row.try_get("source_window_start")?,
        source_window_end: row.try_get("source_window_end")?,
        features: features.as_object().cloned().unwrap_or_default(),
        created_at: row.try_get("fs_created_at")?,
    };

    Ok(LinkedPredictionRecord {
        prediction,
        feature_snapshot,
        contribution_weight: row.try_get("contribution_weight")?,
    })
}

fn graph_from_explanation(
    explanation: &RecommendationExplanation,
    rows: Vec<sqlx::postgres::PgRow>,
) -> Option<RecommendationGraphEvidence> {
    let structured: &StructuredRecommendationExplanation = &explanation.structured_explanation;

    let graph_version = structured
        .graph_version
        .clone()
        .or_else(|| explanation.graph_version.clone())?;
    let graph_links = rows
        .into_iter()
        .map(|row| RecommendationGraphLinkSummary {
            relation_type: explanation_value(&row, "edge_type")
                .unwrap_or_else(|| "linked".to_string()),
            source_item_id: explanation_i64(&row, "source_item_id").unwrap_or_default(),
            target_item_id: explanation_i64(&row, "target_item_id").unwrap_or_default(),
            edge_id: row.try_get("edge_id").ok(),
            event_id: row.try_get("event_id").ok(),
            weight: row.try_get("contribution_weight").ok(),
            explanation: row
                .try_get::<serde_json::Value, _>("explanation")
                .unwrap_or(serde_json::Value::Null),
        })
        .collect();
    Some(RecommendationGraphEvidence {
        graph_version,
        graph_links,
    })
}

fn explanation_value(row: &sqlx::postgres::PgRow, key: &str) -> Option<String> {
    let explanation = row.try_get::<serde_json::Value, _>("explanation").ok()?;
    explanation.get(key)?.as_str().map(str::to_string)
}

fn explanation_i64(row: &sqlx::postgres::PgRow, key: &str) -> Option<i64> {
    let explanation = row.try_get::<serde_json::Value, _>("explanation").ok()?;
    explanation.get(key)?.as_i64()
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
