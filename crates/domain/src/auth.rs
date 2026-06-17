use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DomainValidationError, ExecutionMode, Gp, Probability, Rate, SessionId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainValidationError> {
        let value = value.into().trim().to_lowercase();
        if value.is_empty() {
            return Err(DomainValidationError::EmptyValue { field: "email" });
        }
        if !value.contains('@') {
            return Err(DomainValidationError::InvalidFormat { field: "email" });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub email: EmailAddress,
    pub password: SecretString,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: EmailAddress,
    pub password: SecretString,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub email: EmailAddress,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AuthSession {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserRiskProfile {
    pub user_id: UserId,
    pub max_gp_per_item: Gp,
    pub max_portfolio_drawdown: Probability,
    pub min_expected_roi: Rate,
    pub min_confidence: Probability,
    pub participation_rate: Probability,
    pub preferred_execution_mode: ExecutionMode,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRiskProfile {
    pub max_gp_per_item: i64,
    pub max_portfolio_drawdown: f64,
    pub min_expected_roi: f64,
    pub min_confidence: f64,
    pub participation_rate: f64,
    pub preferred_execution_mode: ExecutionMode,
}

impl UserRiskProfile {
    pub fn from_update(
        user_id: UserId,
        update: UpdateRiskProfile,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainValidationError> {
        Ok(Self {
            user_id,
            max_gp_per_item: Gp::try_from(update.max_gp_per_item)?,
            max_portfolio_drawdown: Probability::new(update.max_portfolio_drawdown)?,
            min_expected_roi: Rate::new(update.min_expected_roi)?,
            min_confidence: Probability::new(update.min_confidence)?,
            participation_rate: Probability::new(update.participation_rate)?,
            preferred_execution_mode: update.preferred_execution_mode,
            updated_at,
        })
    }

    pub fn default_for_user(user_id: UserId, updated_at: DateTime<Utc>) -> Self {
        Self {
            user_id,
            max_gp_per_item: Gp(5_000_000),
            max_portfolio_drawdown: Probability(0.15),
            min_expected_roi: Rate(0.01),
            min_confidence: Probability(0.55),
            participation_rate: Probability(0.10),
            preferred_execution_mode: ExecutionMode::ConservativeInstant,
            updated_at,
        }
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = DomainValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<EmailAddress> for String {
    fn from(value: EmailAddress) -> Self {
        value.0
    }
}

impl From<Uuid> for SessionId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{EmailAddress, UpdateRiskProfile, UserRiskProfile};
    use crate::{ExecutionMode, UserId};

    #[test]
    fn email_address_normalizes_and_validates() {
        let email = EmailAddress::new(" Test@Example.com ").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
        assert!(EmailAddress::new("").is_err());
        assert!(EmailAddress::new("not-an-email").is_err());
    }

    #[test]
    fn risk_profile_rejects_invalid_probability_values() {
        let result = UserRiskProfile::from_update(
            UserId(uuid::Uuid::nil()),
            UpdateRiskProfile {
                max_gp_per_item: 1,
                max_portfolio_drawdown: 1.1,
                min_expected_roi: 0.01,
                min_confidence: 0.5,
                participation_rate: 0.1,
                preferred_execution_mode: ExecutionMode::ConservativeInstant,
            },
            Utc::now(),
        );
        assert!(result.is_err());
    }
}
