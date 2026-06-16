use grand_edge_domain::{FeatureVector, Item, LatestPrice, StrategySignal};

use crate::{LookbackSpec, StrategyContext, StrategyError};

pub trait Strategy: Send + Sync {
    fn id(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn required_lookback(&self) -> LookbackSpec;

    fn generate(
        &self,
        ctx: &StrategyContext,
        item: &Item,
        latest: &LatestPrice,
        features: &FeatureVector,
    ) -> Result<StrategySignal, StrategyError>;
}
