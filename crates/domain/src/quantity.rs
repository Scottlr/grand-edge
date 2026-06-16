use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::DomainValidationError;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Display,
)]
pub struct Quantity(pub i64);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Display,
)]
pub struct HorizonSecs(pub i64);

impl Quantity {
    pub fn positive(value: i64) -> Result<Self, DomainValidationError> {
        if value <= 0 {
            return Err(DomainValidationError::NonPositiveValue { field: "quantity" });
        }

        Ok(Self(value))
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl HorizonSecs {
    pub fn positive(value: i64) -> Result<Self, DomainValidationError> {
        if value <= 0 {
            return Err(DomainValidationError::NonPositiveValue {
                field: "horizon_secs",
            });
        }

        Ok(Self(value))
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for Quantity {
    type Error = DomainValidationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::positive(value)
    }
}

impl TryFrom<i64> for HorizonSecs {
    type Error = DomainValidationError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::positive(value)
    }
}

impl From<Quantity> for i64 {
    fn from(value: Quantity) -> Self {
        value.0
    }
}

impl From<HorizonSecs> for i64 {
    fn from(value: HorizonSecs) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use crate::Quantity;

    #[test]
    fn quantity_rejects_zero_for_order_quantity() {
        assert!(Quantity::positive(0).is_err());
    }
}
