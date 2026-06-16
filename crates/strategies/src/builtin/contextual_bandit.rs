use grand_edge_domain::{FeatureVector, Item, LatestPrice};

use crate::{
    LookbackSpec, Strategy, StrategyContext, StrategyError, artifacts::ModelArtifactMetadata,
};

const STRATEGY_ID: &str = "contextual_bandit_v1";
const STRATEGY_VERSION: &str = "v1";

#[derive(Debug, Clone, Default)]
pub struct ContextualBanditStrategy {
    pub artifact: Option<ModelArtifactMetadata>,
}

impl Strategy for ContextualBanditStrategy {
    fn id(&self) -> &'static str {
        STRATEGY_ID
    }

    fn version(&self) -> &'static str {
        STRATEGY_VERSION
    }

    fn required_lookback(&self) -> LookbackSpec {
        LookbackSpec {
            min_5m_buckets: 1,
            min_1h_buckets: 24,
        }
    }

    fn generate(
        &self,
        _ctx: &StrategyContext,
        _item: &Item,
        _latest: &LatestPrice,
        _features: &FeatureVector,
    ) -> Result<grand_edge_domain::StrategySignal, StrategyError> {
        if self.artifact.is_none() {
            return Err(StrategyError::MissingArtifact(self.id().to_string()));
        }
        Err(StrategyError::Validation(
            "contextual bandit adapter requires validated artifact runtime support".to_string(),
        ))
    }
}
