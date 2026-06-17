use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DomainValidationError;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Display,
    JsonSchema,
)]
pub struct ItemId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, JsonSchema)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, JsonSchema)]
pub struct RecommendationId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, JsonSchema)]
pub struct PositionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, JsonSchema)]
pub struct SessionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, JsonSchema)]
pub struct RunId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, JsonSchema)]
pub struct OrderId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, JsonSchema)]
pub struct PredictionId(pub Uuid);

impl TryFrom<i64> for ItemId {
    type Error = DomainValidationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value <= 0 {
            return Err(DomainValidationError::NonPositiveValue { field: "item_id" });
        }

        Ok(Self(value))
    }
}

impl From<ItemId> for i64 {
    fn from(value: ItemId) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use crate::ItemId;

    #[test]
    fn item_id_rejects_non_positive_values() {
        assert!(ItemId::try_from(0).is_err());
        assert!(ItemId::try_from(-1).is_err());
    }
}
