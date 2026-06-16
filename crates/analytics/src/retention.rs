use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::AnalyticsError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub latest_5m_hot_days: u32,
    pub hourly_hot_days: u32,
    pub feature_snapshots_hot_days: u32,
    pub predictions_hot_days: u32,
    pub recommendations_hot_days: u32,
    pub keep_raw_interval_data_forever: bool,
    pub keep_model_artifacts_forever: bool,
    pub keep_model_cards_forever: bool,
    pub keep_feature_schemas_forever: bool,
    pub keep_recommendation_outcomes_forever: bool,
    pub keep_strategy_summaries_forever: bool,
    pub keep_graph_versions_forever: bool,
    pub keep_edge_observations_forever: bool,
    pub keep_market_events_forever: bool,
    pub keep_blast_simulations_forever: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            latest_5m_hot_days: 90,
            hourly_hot_days: 730,
            feature_snapshots_hot_days: 90,
            predictions_hot_days: 180,
            recommendations_hot_days: 180,
            keep_raw_interval_data_forever: true,
            keep_model_artifacts_forever: true,
            keep_model_cards_forever: true,
            keep_feature_schemas_forever: true,
            keep_recommendation_outcomes_forever: true,
            keep_strategy_summaries_forever: true,
            keep_graph_versions_forever: true,
            keep_edge_observations_forever: true,
            keep_market_events_forever: true,
            keep_blast_simulations_forever: true,
        }
    }
}

impl RetentionPolicy {
    pub fn validate(&self) -> Result<(), AnalyticsError> {
        for (name, value) in [
            ("latest_5m_hot_days", self.latest_5m_hot_days),
            ("hourly_hot_days", self.hourly_hot_days),
            (
                "feature_snapshots_hot_days",
                self.feature_snapshots_hot_days,
            ),
            ("predictions_hot_days", self.predictions_hot_days),
            ("recommendations_hot_days", self.recommendations_hot_days),
        ] {
            if value == 0 {
                return Err(AnalyticsError::InvalidRetentionPolicy(name));
            }
        }

        Ok(())
    }

    pub fn cutoff_days(&self, as_of: DateTime<Utc>, hot_days: u32) -> DateTime<Utc> {
        as_of - Duration::days(i64::from(hot_days))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::RetentionPolicy;

    #[test]
    fn default_retention_policy_matches_planning_windows() {
        let policy = RetentionPolicy::default();
        assert_eq!(policy.latest_5m_hot_days, 90);
        assert_eq!(policy.hourly_hot_days, 730);
        assert_eq!(policy.feature_snapshots_hot_days, 90);
        assert_eq!(policy.predictions_hot_days, 180);
        assert_eq!(policy.recommendations_hot_days, 180);
        assert!(policy.keep_recommendation_outcomes_forever);
        assert!(policy.keep_graph_versions_forever);
        assert!(policy.keep_market_events_forever);
    }

    #[test]
    fn retention_policy_cutoff_uses_hot_days() {
        let policy = RetentionPolicy::default();
        let as_of = Utc.with_ymd_and_hms(2026, 6, 16, 0, 0, 0).unwrap();
        assert_eq!(
            policy.cutoff_days(as_of, policy.predictions_hot_days),
            Utc.with_ymd_and_hms(2025, 12, 18, 0, 0, 0).unwrap()
        );
    }
}
