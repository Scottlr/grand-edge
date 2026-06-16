use std::collections::HashMap;

use chrono::{DateTime, Utc};
use grand_edge_domain::{MarketRules, ModelAccuracySnapshot};

use crate::RiskConfig;

#[derive(Debug, Clone)]
pub struct StrategyContext {
    pub as_of: DateTime<Utc>,
    pub market_rules: MarketRules,
    pub risk: RiskConfig,
    pub recent_metrics: HashMap<String, ModelAccuracySnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookbackSpec {
    pub min_5m_buckets: usize,
    pub min_1h_buckets: usize,
}
