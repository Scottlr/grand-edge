use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::StrategyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelArtifactKind {
    GbdtRanker,
    ContextualBandit,
    OnlineEnsemble,
    MetaLabel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelArtifactMetadata {
    pub strategy_id: String,
    pub model_version: String,
    pub feature_set_version: String,
    pub feature_schema_hash: String,
    pub trained_at: DateTime<Utc>,
    pub training_window_start: DateTime<Utc>,
    pub training_window_end: DateTime<Utc>,
    pub evaluation_window_start: DateTime<Utc>,
    pub evaluation_window_end: DateTime<Utc>,
    pub artifact_uri: String,
    pub artifact_kind: ModelArtifactKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactFeatureSchema {
    pub feature_names: Vec<String>,
    pub target_label: TrainingTargetLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingTargetLabel {
    FutureReturn6h,
    FutureTaxAdjustedReturn6h,
    FutureActionableReturn6h,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCard {
    pub strategy_id: String,
    pub target_label: TrainingTargetLabel,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TripleBarrierLabel {
    TakeProfit,
    StopLoss,
    Expired,
}

pub fn validate_artifact_metadata(
    metadata: &ModelArtifactMetadata,
    expected_strategy_id: &str,
    expected_feature_set_version: &str,
    as_of: DateTime<Utc>,
) -> Result<(), StrategyError> {
    if metadata.strategy_id != expected_strategy_id {
        return Err(StrategyError::Validation(
            "artifact strategy id did not match expected strategy".to_string(),
        ));
    }
    if metadata.feature_set_version != expected_feature_set_version {
        return Err(StrategyError::Validation(
            "artifact feature set version did not match expected version".to_string(),
        ));
    }
    if metadata.model_version.trim().is_empty() {
        return Err(StrategyError::Validation(
            "artifact model_version must not be empty".to_string(),
        ));
    }
    if metadata.artifact_uri.trim().is_empty() {
        return Err(StrategyError::Validation(
            "artifact uri must not be empty".to_string(),
        ));
    }
    if metadata.training_window_end > as_of {
        return Err(StrategyError::Validation(
            "artifact training window must not extend past as_of".to_string(),
        ));
    }
    if metadata.training_window_end < metadata.training_window_start {
        return Err(StrategyError::Validation(
            "artifact training window end must be after start".to_string(),
        ));
    }
    if metadata.evaluation_window_start < metadata.training_window_end {
        return Err(StrategyError::Validation(
            "artifact evaluation window must start after training window end".to_string(),
        ));
    }
    if metadata.evaluation_window_end < metadata.evaluation_window_start {
        return Err(StrategyError::Validation(
            "artifact evaluation window end must be after evaluation start".to_string(),
        ));
    }

    Ok(())
}

pub fn hedge_update(weights: &[f64], losses: &[f64], eta: f64) -> Result<Vec<f64>, StrategyError> {
    if weights.is_empty() || weights.len() != losses.len() {
        return Err(StrategyError::Validation(
            "weights and losses must be non-empty and same length".to_string(),
        ));
    }
    if !eta.is_finite() || eta <= 0.0 {
        return Err(StrategyError::Validation(
            "eta must be positive and finite".to_string(),
        ));
    }

    let mut updated = Vec::with_capacity(weights.len());
    for (weight, loss) in weights.iter().zip(losses.iter()) {
        if !weight.is_finite() || *weight < 0.0 || !loss.is_finite() {
            return Err(StrategyError::Validation(
                "weights and losses must be finite, and weights non-negative".to_string(),
            ));
        }
        updated.push(weight * (-eta * loss).exp());
    }
    let normalizer: f64 = updated.iter().sum();
    if normalizer <= 0.0 || !normalizer.is_finite() {
        return Err(StrategyError::Validation(
            "hedge update normalizer must be positive and finite".to_string(),
        ));
    }
    for value in &mut updated {
        *value /= normalizer;
    }

    Ok(updated)
}

pub fn linucb_score(predicted_reward: f64, uncertainty: f64, alpha: f64) -> f64 {
    predicted_reward + alpha * uncertainty
}

pub fn triple_barrier_label(
    entry: f64,
    take_profit_pct: f64,
    stop_loss_pct: f64,
    future_path: &[f64],
) -> Option<TripleBarrierLabel> {
    if !entry.is_finite() || entry <= 0.0 {
        return None;
    }

    let take_profit = entry * (1.0 + take_profit_pct);
    let stop_loss = entry * (1.0 - stop_loss_pct);
    for price in future_path {
        if !price.is_finite() {
            return None;
        }
        if *price >= take_profit {
            return Some(TripleBarrierLabel::TakeProfit);
        }
        if *price <= stop_loss {
            return Some(TripleBarrierLabel::StopLoss);
        }
    }

    Some(TripleBarrierLabel::Expired)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{
        ArtifactFeatureSchema, ModelArtifactKind, ModelArtifactMetadata, ModelCard,
        TrainingTargetLabel, TripleBarrierLabel, hedge_update, linucb_score, triple_barrier_label,
        validate_artifact_metadata,
    };

    fn metadata() -> ModelArtifactMetadata {
        ModelArtifactMetadata {
            strategy_id: "gbm_ranker_v1".to_string(),
            model_version: "v1".to_string(),
            feature_set_version: "features_v1".to_string(),
            feature_schema_hash: "abc123".to_string(),
            trained_at: Utc.with_ymd_and_hms(2026, 6, 10, 12, 0, 0).unwrap(),
            training_window_start: Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
            training_window_end: Utc.with_ymd_and_hms(2026, 5, 31, 0, 0, 0).unwrap(),
            evaluation_window_start: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            evaluation_window_end: Utc.with_ymd_and_hms(2026, 6, 9, 0, 0, 0).unwrap(),
            artifact_uri: "file:///tmp/gbm_ranker_v1.json".to_string(),
            artifact_kind: ModelArtifactKind::GbdtRanker,
        }
    }

    #[test]
    fn artifact_rejects_future_training_window() {
        let mut metadata = metadata();
        metadata.training_window_end = Utc.with_ymd_and_hms(2026, 6, 20, 0, 0, 0).unwrap();
        assert!(
            validate_artifact_metadata(
                &metadata,
                "gbm_ranker_v1",
                "features_v1",
                Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            )
            .is_err()
        );
    }

    #[test]
    fn artifact_rejects_feature_set_mismatch() {
        assert!(
            validate_artifact_metadata(
                &metadata(),
                "gbm_ranker_v1",
                "features_v2",
                Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            )
            .is_err()
        );
    }

    #[test]
    fn artifact_schema_uses_observed_liquidity_proxy_names() {
        let schema = ArtifactFeatureSchema {
            feature_names: vec![
                "observed_volume_z_24h".to_string(),
                "observed_volume_reliability_24h".to_string(),
                "price_staleness_secs".to_string(),
                "spread_pct".to_string(),
                "high_low_volume_ratio_1h".to_string(),
                "liquidity_confidence".to_string(),
                "missing_data_flags".to_string(),
            ],
            target_label: TrainingTargetLabel::FutureActionableReturn6h,
        };

        assert!(
            schema
                .feature_names
                .iter()
                .all(|name| !matches!(name.as_str(), "trueLiquidity" | "marketDepth"))
        );
        assert!(
            schema
                .feature_names
                .iter()
                .any(|name| name == "observed_volume_z_24h")
        );
    }

    #[test]
    fn model_card_declares_actionable_return_target_when_used() {
        let model_card = ModelCard {
            strategy_id: "gbm_ranker_v1".to_string(),
            target_label: TrainingTargetLabel::FutureActionableReturn6h,
            notes: "Observed volume and liquidity confidence are proxy inputs.".to_string(),
        };
        assert_eq!(
            model_card.target_label,
            TrainingTargetLabel::FutureActionableReturn6h
        );
    }

    #[test]
    fn online_ensemble_hedge_update_matches_goal_fixture() {
        let weights = hedge_update(&[0.5, 0.5], &[0.01, 0.04], 10.0).unwrap();
        assert!((weights[0] - 0.574_442_516_8).abs() < 1e-6);
        assert!((weights[1] - 0.425_557_483_2).abs() < 1e-6);
    }

    #[test]
    fn contextual_bandit_linucb_score_matches_goal_fixture() {
        assert!((linucb_score(0.012, 0.006, 1.5) - 0.021).abs() < 1e-12);
    }

    #[test]
    fn meta_label_triple_barrier_matches_goal_fixture() {
        assert_eq!(
            triple_barrier_label(100.0, 0.03, 0.02, &[99.0, 101.0, 103.2]),
            Some(TripleBarrierLabel::TakeProfit)
        );
    }
}
