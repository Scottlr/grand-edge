use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

use crate::DomainValidationError;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Display, From,
)]
pub struct BasisPoints(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Probability(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Rate(pub f64);

impl Probability {
    pub fn new(value: f64) -> Result<Self, DomainValidationError> {
        if !value.is_finite() {
            return Err(DomainValidationError::NonFiniteValue {
                field: "probability",
            });
        }

        if !(0.0..=1.0).contains(&value) {
            return Err(DomainValidationError::ProbabilityOutOfRange {
                field: "probability",
            });
        }

        Ok(Self(value))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl Rate {
    pub fn new(value: f64) -> Result<Self, DomainValidationError> {
        if !value.is_finite() {
            return Err(DomainValidationError::NonFiniteValue { field: "rate" });
        }

        Ok(Self(value))
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::Probability;

    #[test]
    fn probability_rejects_nan_and_out_of_range() {
        assert!(Probability::new(f64::NAN).is_err());
        assert!(Probability::new(-0.1).is_err());
        assert!(Probability::new(1.1).is_err());
    }
}
