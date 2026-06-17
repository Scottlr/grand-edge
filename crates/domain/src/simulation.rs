use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ItemId, OrderId, RecommendationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    ConservativeInstant,
    PassiveEstimated,
    HaircutPassive,
    WorstCase,
    UserPositionReplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperBetStatus {
    Created,
    Open,
    PartiallyFilled,
    Filled,
    Holding,
    ExitPending,
    Closed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperBet {
    pub paper_bet_id: OrderId,
    pub recommendation_id: Option<RecommendationId>,
    pub item_id: ItemId,
    pub created_at: DateTime<Utc>,
    pub status: PaperBetStatus,
    pub execution_mode: ExecutionMode,
}

#[cfg(test)]
mod tests {
    use super::ExecutionMode;

    #[test]
    fn execution_mode_serde_uses_conservative_proxy_names() {
        assert_eq!(
            serde_json::to_string(&ExecutionMode::ConservativeInstant).unwrap(),
            "\"conservative_instant\""
        );
        assert_eq!(
            serde_json::to_string(&ExecutionMode::PassiveEstimated).unwrap(),
            "\"passive_estimated\""
        );
        assert_eq!(
            serde_json::to_string(&ExecutionMode::HaircutPassive).unwrap(),
            "\"haircut_passive\""
        );
        assert_eq!(
            serde_json::to_string(&ExecutionMode::WorstCase).unwrap(),
            "\"worst_case\""
        );
    }
}
