use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{GraphDomainError, ItemId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketEventType {
    GameUpdate,
    BossAnnouncement,
    DropTableChange,
    SkillUpdate,
    PvpUpdate,
    LeagueOrDeadman,
    BotBanWave,
    TaxOrRuleChange,
    ItemSink,
    CombatRebalance,
    MarketAnalysisNote,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketEventNode {
    pub event_id: Uuid,
    pub graph_version: String,
    pub event_type: MarketEventType,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
    pub source_ref: String,
    pub affected_item_ids: Vec<ItemId>,
    pub metadata: serde_json::Value,
}

impl MarketEventNode {
    pub fn validate(&self) -> Result<(), GraphDomainError> {
        if self.graph_version.trim().is_empty() {
            return Err(GraphDomainError::EmptyGraphVersion);
        }
        if self.title.trim().is_empty() {
            return Err(GraphDomainError::EmptyField { field: "title" });
        }
        if self.source_ref.trim().is_empty() {
            return Err(GraphDomainError::EmptyField {
                field: "source_ref",
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MarketEventType;

    #[test]
    fn market_event_type_serde_uses_snake_case() {
        assert_eq!(
            serde_json::to_string(&MarketEventType::TaxOrRuleChange).unwrap(),
            "\"tax_or_rule_change\""
        );
    }
}
