use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
pub struct Gp(pub i64);

impl Gp {
    pub const ZERO: Self = Self(0);

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    pub fn non_negative(value: i64) -> Result<Self, DomainValidationError> {
        if value < 0 {
            return Err(DomainValidationError::NegativeValue { field: "gp" });
        }

        Ok(Self(value))
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for Gp {
    type Error = DomainValidationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::non_negative(value)
    }
}

impl From<Gp> for i64 {
    fn from(value: Gp) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use crate::Gp;

    #[test]
    fn gp_rejects_negative_price_values() {
        assert!(Gp::non_negative(-1).is_err());
    }
}
