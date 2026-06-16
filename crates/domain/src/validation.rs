use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum DomainValidationError {
    #[error("{field} must be positive")]
    NonPositiveValue { field: &'static str },
    #[error("{field} must be non-negative")]
    NegativeValue { field: &'static str },
    #[error("{field} must be finite")]
    NonFiniteValue { field: &'static str },
    #[error("{field} must be within [0.0, 1.0]")]
    ProbabilityOutOfRange { field: &'static str },
    #[error("{field} must not be empty")]
    EmptyValue { field: &'static str },
}
