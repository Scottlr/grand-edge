use std::collections::HashSet;

use chrono::{DateTime, Utc};
use grand_edge_domain::{
    FeatureSnapshot, FeatureVector, GraphPath, ItemId, Prediction, Recommendation,
    RecommendationAction, RecommendationId, RecommendationPredictionContribution,
    RecommendationPredictionLink, StrategySignal,
};
use grand_edge_storage::{RecommendationGraphLinkRecord, Storage};
use grand_edge_strategies::strategy_output_to_prediction;
use uuid::Uuid;

use crate::RecommendationError;

pub fn build_prediction_links(
    recommendation_id: RecommendationId,
    contributions: &[RecommendationPredictionContribution],
) -> Result<Vec<RecommendationPredictionLink>, RecommendationError> {
    let mut seen = HashSet::new();
    let mut links = Vec::with_capacity(contributions.len());
    for contribution in contributions {
        if !seen.insert(contribution.prediction_id) {
            return Err(RecommendationError::DuplicatePredictionLink(
                contribution.prediction_id.0,
            ));
        }
        if !contribution.contribution_weight.is_finite() {
            return Err(RecommendationError::InvalidContributionWeight(
                contribution.contribution_weight,
            ));
        }
        if contribution.contribution_weight < 0.0 {
            return Err(RecommendationError::NegativeContributionWeight(
                contribution.contribution_weight,
            ));
        }
        links.push(RecommendationPredictionLink::new(
            recommendation_id,
            contribution.prediction_id,
            contribution.contribution_weight,
        )?);
    }

    Ok(links)
}

pub async fn persist_recommendation_decision(
    storage: &Storage,
    recommendation: &Recommendation,
    feature_vector: &FeatureVector,
    predictions: &[Prediction],
    contributions: &[RecommendationPredictionContribution],
) -> Result<(), RecommendationError> {
    if predictions.is_empty()
        && !matches!(
            recommendation.action,
            RecommendationAction::Watch | RecommendationAction::Avoid
        )
    {
        return Err(RecommendationError::MissingPredictionsForAction(
            recommendation.action,
        ));
    }

    let links = build_prediction_links(recommendation.recommendation_id, contributions)?;
    let graph_links = build_graph_links(recommendation)?;
    let mut transaction = storage.pool().begin().await?;

    if let Some(snapshot) = compatibility_feature_snapshot(feature_vector, recommendation.as_of) {
        storage
            .evidence()
            .insert_feature_snapshot_in_tx(&mut transaction, &snapshot)
            .await?;
    }

    storage
        .predictions()
        .insert_predictions_in_tx(&mut transaction, predictions)
        .await?;
    storage
        .recommendations()
        .insert_recommendation_with_links_in_tx(&mut transaction, recommendation, &links)
        .await?;
    if !graph_links.is_empty() {
        storage
            .graph()
            .insert_recommendation_graph_links_in_tx(&mut transaction, &graph_links)
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}

pub fn compatibility_feature_snapshot(
    feature_vector: &FeatureVector,
    created_at: DateTime<Utc>,
) -> Option<FeatureSnapshot> {
    if feature_vector.values.is_empty() {
        return None;
    }

    Some(FeatureSnapshot {
        feature_snapshot_id: feature_snapshot_id(feature_vector.item_id, feature_vector),
        item_id: feature_vector.item_id,
        as_of: feature_vector.as_of,
        feature_set_version: feature_vector.feature_set_version.clone(),
        graph_version: feature_vector
            .values
            .get("graph_version")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        source_window_start: feature_vector.as_of,
        source_window_end: feature_vector.as_of,
        features: feature_vector.values.clone(),
        created_at,
    })
}

pub fn prediction_contributions(
    feature_snapshot_id: Uuid,
    created_at: DateTime<Utc>,
    votes: &[StrategySignal],
) -> Result<(Vec<Prediction>, Vec<RecommendationPredictionContribution>), RecommendationError> {
    if votes.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let confidence_sum = votes.iter().map(|vote| vote.confidence.get()).sum::<f64>();
    let default_weight = 1.0 / votes.len() as f64;

    let mut predictions = Vec::with_capacity(votes.len());
    let mut contributions = Vec::with_capacity(votes.len());
    for vote in votes {
        let prediction = strategy_output_to_prediction(vote, feature_snapshot_id, created_at)?;
        let contribution_weight = if confidence_sum > 0.0 {
            vote.confidence.get() / confidence_sum
        } else {
            default_weight
        };
        contributions.push(RecommendationPredictionContribution {
            prediction_id: prediction.prediction_id,
            contribution_weight,
            source_model_id: prediction.model_id.clone(),
            source_model_version: prediction.model_version.clone(),
        });
        predictions.push(prediction);
    }

    Ok((predictions, contributions))
}

fn feature_snapshot_id(item_id: ItemId, feature_vector: &FeatureVector) -> Uuid {
    let _ = (item_id, feature_vector);
    Uuid::new_v4()
}

fn build_graph_links(
    recommendation: &Recommendation,
) -> Result<Vec<RecommendationGraphLinkRecord>, RecommendationError> {
    let Some(graph_context) = recommendation.explanation.graph_context.as_ref() else {
        return Ok(Vec::new());
    };

    let mut links = Vec::new();
    for path in &graph_context.paths {
        append_graph_path_links(
            &mut links,
            recommendation.recommendation_id,
            &graph_context.graph_version,
            path,
            graph_context.edge_confidence,
        );
    }

    Ok(links)
}

fn append_graph_path_links(
    links: &mut Vec<RecommendationGraphLinkRecord>,
    recommendation_id: RecommendationId,
    graph_version: &str,
    path: &GraphPath,
    edge_confidence: Option<f64>,
) {
    for step in &path.steps {
        links.push(RecommendationGraphLinkRecord {
            link_id: Uuid::new_v4(),
            recommendation_id,
            graph_version: graph_version.to_string(),
            edge_id: Some(step.edge_id),
            event_id: None,
            contribution_weight: edge_confidence.or(Some(path.path_confidence)),
            explanation: serde_json::json!({
                "source_item_id": path.source_item_id.0,
                "target_item_id": path.target_item_id.0,
                "edge_type": step.edge_type,
                "path_confidence": path.path_confidence,
                "expected_impact": path.expected_impact,
            }),
        });
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{
        FeatureVector, GraphPath, GraphPathStep, GraphRecommendationAction, PredictionId,
        Recommendation, RecommendationAction, RecommendationExplanation, RecommendationId,
        RecommendationPredictionContribution, StructuredRecommendationExplanation,
    };
    use uuid::Uuid;

    use super::{
        build_graph_links, build_prediction_links, compatibility_feature_snapshot,
        prediction_contributions,
    };

    #[test]
    fn build_prediction_links_rejects_duplicate_prediction_id() {
        let prediction_id = PredictionId(Uuid::new_v4());
        let contributions = vec![
            RecommendationPredictionContribution {
                prediction_id,
                contribution_weight: 0.5,
                source_model_id: grand_edge_domain::StrategyId::new("spread_edge").unwrap(),
                source_model_version: grand_edge_domain::ModelVersion::new("v1").unwrap(),
            },
            RecommendationPredictionContribution {
                prediction_id,
                contribution_weight: 0.5,
                source_model_id: grand_edge_domain::StrategyId::new("momentum").unwrap(),
                source_model_version: grand_edge_domain::ModelVersion::new("v1").unwrap(),
            },
        ];

        let error = build_prediction_links(
            grand_edge_domain::RecommendationId(Uuid::new_v4()),
            &contributions,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            crate::RecommendationError::DuplicatePredictionLink(_)
        ));
    }

    #[test]
    fn compatibility_feature_snapshot_uses_feature_vector_identity() {
        let feature_vector = FeatureVector {
            item_id: grand_edge_domain::ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values: serde_json::Map::from_iter([(
                "spread_pct".to_string(),
                serde_json::json!(0.02),
            )]),
        };

        let snapshot =
            compatibility_feature_snapshot(&feature_vector, feature_vector.as_of).unwrap();
        assert_eq!(snapshot.item_id, feature_vector.item_id);
        assert_eq!(snapshot.feature_set_version, "features_v1");
        assert_eq!(snapshot.source_window_start, feature_vector.as_of);
    }

    #[test]
    fn prediction_contributions_normalize_signal_confidence() {
        let signal = grand_edge_domain::StrategySignal {
            item_id: grand_edge_domain::ItemId(4151),
            strategy_id: grand_edge_domain::StrategyId::new("spread_edge").unwrap(),
            model_version: grand_edge_domain::ModelVersion::new("v1").unwrap(),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            side: grand_edge_domain::SignalSide::Buy,
            horizon_secs: grand_edge_domain::HorizonSecs(3600),
            confidence: grand_edge_domain::Probability::new(0.8).unwrap(),
            expected_return: grand_edge_domain::Rate::new(0.04).unwrap(),
            expected_net_gp_per_unit: grand_edge_domain::Gp(1200),
            target_entry: None,
            target_exit: None,
            stop_loss: None,
            take_profit: None,
            max_quantity: None,
            execution_estimate: None,
            explanation: serde_json::json!({}),
        };
        let (predictions, contributions) =
            prediction_contributions(Uuid::new_v4(), signal.as_of, &[signal]).unwrap();
        assert_eq!(predictions.len(), 1);
        assert_eq!(contributions.len(), 1);
        assert_eq!(contributions[0].contribution_weight, 1.0);
    }

    #[test]
    fn graph_links_persist_for_graph_recommendation() {
        let recommendation_id = RecommendationId(Uuid::new_v4());
        let links = build_graph_links(&Recommendation {
            recommendation_id,
            user_id: None,
            item_id: grand_edge_domain::ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            action: RecommendationAction::Watch,
            score: grand_edge_domain::Rate::new(0.2).unwrap(),
            prediction_confidence: None,
            execution_confidence: None,
            recommendation_confidence: grand_edge_domain::Probability::new(0.4).unwrap(),
            expected_net_gp: None,
            expected_roi: None,
            risk_label: None,
            reasons: Vec::new(),
            explanation: RecommendationExplanation {
                feature_set_version: "features_v1".to_string(),
                market_rules_version: "rules_v1".to_string(),
                graph_version: Some("graph_v1".to_string()),
                graph_context: Some(grand_edge_domain::GraphRecommendationContext {
                    graph_version: "graph_v1".to_string(),
                    graph_action: Some(GraphRecommendationAction::WatchSecondOrder),
                    paths: vec![GraphPath {
                        source_item_id: grand_edge_domain::ItemId(11840),
                        target_item_id: grand_edge_domain::ItemId(4151),
                        steps: vec![GraphPathStep {
                            from_item_id: grand_edge_domain::ItemId(11840),
                            to_item_id: grand_edge_domain::ItemId(4151),
                            edge_id: Uuid::new_v4(),
                            edge_type: grand_edge_domain::GraphEdgeType::IngredientOf,
                            confidence: 0.9,
                            weight: 0.8,
                        }],
                        path_confidence: 0.9,
                        expected_impact: Some(0.05),
                    }],
                    edge_confidence: Some(0.9),
                    historical_path_performance: None,
                }),
                strategy_votes: Vec::new(),
                score_components: Vec::new(),
                accuracy_snapshot: None,
                structured_explanation: StructuredRecommendationExplanation::default(),
            },
        })
        .unwrap();

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].graph_version, "graph_v1");
        assert!(links[0].edge_id.is_some());
    }
}
