use std::{collections::BTreeMap, sync::Arc};

use grand_edge_domain::{FeatureVector, Item, LatestPrice, StrategySignal};
use grand_edge_storage::Storage;

use crate::{
    Strategy, StrategyConfig, StrategyContext, StrategyError, validation::validate_signal,
};

#[derive(Debug)]
pub struct StrategyRunResult {
    pub strategy_id: String,
    pub signal: Option<StrategySignal>,
    pub error: Option<StrategyError>,
}

pub struct StrategyRegistry {
    strategies: BTreeMap<String, Arc<dyn Strategy>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self {
            strategies: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, strategy: Arc<dyn Strategy>) -> Result<(), StrategyError> {
        let strategy_id = strategy.id().to_string();
        if self.strategies.contains_key(&strategy_id) {
            return Err(StrategyError::DuplicateStrategyId(strategy_id));
        }

        self.strategies.insert(strategy_id, strategy);
        Ok(())
    }

    pub fn enabled<'a>(&'a self, config: &'a StrategyConfig) -> Vec<&'a dyn Strategy> {
        config
            .enabled_strategies
            .iter()
            .filter_map(|strategy_id| self.strategies.get(strategy_id))
            .map(|strategy| strategy.as_ref())
            .collect()
    }

    pub fn get(&self, strategy_id: &str) -> Option<&dyn Strategy> {
        self.strategies.get(strategy_id).map(Arc::as_ref)
    }

    pub fn ids(&self) -> Vec<String> {
        self.strategies.keys().cloned().collect()
    }

    pub fn generate_all(
        &self,
        config: &StrategyConfig,
        ctx: &StrategyContext,
        item: &Item,
        latest: &LatestPrice,
        features: &FeatureVector,
    ) -> Vec<StrategyRunResult> {
        self.enabled(config)
            .into_iter()
            .map(
                |strategy| match strategy.generate(ctx, item, latest, features) {
                    Ok(signal) => match validate_signal(&signal, strategy.id(), strategy.version())
                    {
                        Ok(()) => StrategyRunResult {
                            strategy_id: strategy.id().to_string(),
                            signal: Some(signal),
                            error: None,
                        },
                        Err(error) => StrategyRunResult {
                            strategy_id: strategy.id().to_string(),
                            signal: None,
                            error: Some(error),
                        },
                    },
                    Err(error) => StrategyRunResult {
                        strategy_id: strategy.id().to_string(),
                        signal: None,
                        error: Some(error),
                    },
                },
            )
            .collect()
    }

    pub async fn generate_all_and_persist(
        &self,
        storage: &Storage,
        config: &StrategyConfig,
        ctx: &StrategyContext,
        item: &Item,
        latest: &LatestPrice,
        features: &FeatureVector,
    ) -> Result<Vec<StrategyRunResult>, StrategyError> {
        let results = self.generate_all(config, ctx, item, latest, features);
        let signals: Vec<StrategySignal> = results
            .iter()
            .filter_map(|result| result.signal.clone())
            .collect();
        if !signals.is_empty() {
            storage.strategies().insert_predictions(&signals).await?;
        }

        Ok(results)
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{TimeZone, Utc};
    use grand_edge_domain::{FeatureVector, Gp, Item, ItemId, LatestPrice, Probability};

    use super::StrategyRegistry;
    use crate::{
        StrategyConfig,
        builtin::{FailingTestStrategy, NoopTestStrategy, test_context},
    };

    fn feature_vector() -> FeatureVector {
        FeatureVector {
            item_id: ItemId(4151),
            as_of: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
            feature_set_version: "features_v1".to_string(),
            values: serde_json::Map::new(),
        }
    }

    fn item() -> Item {
        Item {
            item_id: ItemId(4151),
            name: "Abyssal whip".to_string(),
            examine: None,
            members: true,
            buy_limit: Some(70),
            low_alch: None,
            high_alch: None,
            value: None,
            icon: None,
            updated_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    fn latest() -> LatestPrice {
        LatestPrice {
            item_id: ItemId(4151),
            high: Some(Gp(100)),
            high_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 59, 0).unwrap()),
            low: Some(Gp(90)),
            low_time: Some(Utc.with_ymd_and_hms(2026, 6, 16, 11, 58, 0).unwrap()),
            observed_at: Utc.with_ymd_and_hms(2026, 6, 16, 12, 0, 0).unwrap(),
        }
    }

    #[test]
    fn register_rejects_duplicate_id() {
        let mut registry = StrategyRegistry::new();
        registry.register(Arc::new(NoopTestStrategy)).unwrap();
        assert!(registry.register(Arc::new(NoopTestStrategy)).is_err());
    }

    #[test]
    fn enabled_filters_disabled_strategies() {
        let mut registry = StrategyRegistry::new();
        registry.register(Arc::new(NoopTestStrategy)).unwrap();
        registry.register(Arc::new(FailingTestStrategy)).unwrap();

        let config = StrategyConfig {
            enabled_strategies: vec!["noop".to_string()],
            risk: crate::RiskConfig::default(),
            ..StrategyConfig::default()
        };

        let enabled = registry.enabled(&config);
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id(), "noop");
    }

    #[test]
    fn generate_all_isolates_strategy_failure() {
        let mut registry = StrategyRegistry::new();
        registry.register(Arc::new(NoopTestStrategy)).unwrap();
        registry.register(Arc::new(FailingTestStrategy)).unwrap();

        let config = StrategyConfig {
            enabled_strategies: vec!["noop".to_string(), "fail".to_string()],
            risk: crate::RiskConfig {
                min_confidence: Probability::new(0.5).unwrap().get(),
                ..crate::RiskConfig::default()
            },
            ..StrategyConfig::default()
        };
        let results = registry.generate_all(
            &config,
            &test_context(),
            &item(),
            &latest(),
            &feature_vector(),
        );

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|result| result.signal.is_some()));
        assert!(results.iter().any(|result| result.error.is_some()));
    }
}
