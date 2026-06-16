use derive_more::{Display, From};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::DomainValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From, JsonSchema)]
pub struct StrategyId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From, JsonSchema)]
pub struct ModelVersion(pub String);

impl StrategyId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainValidationError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainValidationError::EmptyValue {
                field: "strategy_id",
            });
        }

        Ok(Self(value))
    }
}

impl ModelVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainValidationError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainValidationError::EmptyValue {
                field: "model_version",
            });
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use crate::ModelVersion;

    #[test]
    fn model_version_rejects_empty_value() {
        assert!(ModelVersion::new("   ").is_err());
    }
}
