use std::time::Duration;

use reqwest::Url;

use crate::IngestError;

pub const GRAND_EDGE_APP_NAME: &str = "GrandEdge";
pub const GRAND_EDGE_CONTACT_EMAIL: &str = "scott.rangeley@outlook.com";
pub const GRAND_EDGE_USER_AGENT: &str = "GrandEdge/0.1 (OSRS Grand Exchange recommendation terminal; contact: scott.rangeley@outlook.com)";
const BLOCKED_USER_AGENT_FRAGMENTS: &[&str] = &[
    "python-requests",
    "Python-urllib",
    "Apache-HttpClient",
    "RestSharp",
    "Java/",
    "curl/",
];

#[derive(Debug, Clone)]
pub struct OsrsWikiConfig {
    pub base_url: Url,
    pub app_name: String,
    pub contact_email: String,
    pub user_agent: String,
    pub request_timeout: Duration,
    pub rate_limit: OsrsWikiRateLimitConfig,
}

#[derive(Debug, Clone)]
pub struct OsrsWikiRateLimitConfig {
    pub max_requests_per_second: f64,
    pub burst_size: u32,
    pub respect_retry_after: bool,
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
}

impl OsrsWikiConfig {
    pub fn grandedge_default() -> Result<Self, IngestError> {
        let config = Self {
            base_url: Url::parse("https://prices.runescape.wiki/api/v1/osrs")
                .map_err(|error| IngestError::InvalidConfig(error.to_string()))?,
            app_name: GRAND_EDGE_APP_NAME.to_owned(),
            contact_email: GRAND_EDGE_CONTACT_EMAIL.to_owned(),
            user_agent: GRAND_EDGE_USER_AGENT.to_owned(),
            request_timeout: Duration::from_secs(10),
            rate_limit: OsrsWikiRateLimitConfig {
                max_requests_per_second: 1.0,
                burst_size: 2,
                respect_retry_after: true,
                max_retries: 3,
                initial_backoff: Duration::from_millis(500),
                max_backoff: Duration::from_secs(30),
            },
        };

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), IngestError> {
        if self.user_agent.trim().is_empty() {
            return Err(IngestError::InvalidConfig(
                "user_agent must not be empty".to_string(),
            ));
        }
        if self.user_agent.contains("replace-me") {
            return Err(IngestError::InvalidConfig(
                "user_agent must not contain replace-me".to_string(),
            ));
        }
        if !self.user_agent.contains(GRAND_EDGE_APP_NAME) {
            return Err(IngestError::InvalidConfig(
                "user_agent must contain GrandEdge".to_string(),
            ));
        }
        if !self.user_agent.contains(GRAND_EDGE_CONTACT_EMAIL) {
            return Err(IngestError::InvalidConfig(
                "user_agent must contain scott.rangeley@outlook.com".to_string(),
            ));
        }
        if BLOCKED_USER_AGENT_FRAGMENTS
            .iter()
            .any(|fragment| self.user_agent.contains(fragment))
        {
            return Err(IngestError::InvalidConfig(
                "user_agent must not use a blocked default client string".to_string(),
            ));
        }
        if self.rate_limit.max_requests_per_second <= 0.0 {
            return Err(IngestError::InvalidConfig(
                "max_requests_per_second must be > 0".to_string(),
            ));
        }
        if self.rate_limit.burst_size == 0 {
            return Err(IngestError::InvalidConfig(
                "burst_size must be > 0".to_string(),
            ));
        }
        if self.rate_limit.max_retries > 10 {
            return Err(IngestError::InvalidConfig(
                "max_retries must be <= 10".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{GRAND_EDGE_USER_AGENT, OsrsWikiConfig};

    #[test]
    fn grandedge_default_matches_required_user_agent() {
        let config = OsrsWikiConfig::grandedge_default().unwrap();
        assert_eq!(config.user_agent, GRAND_EDGE_USER_AGENT);
    }

    #[test]
    fn validate_rejects_library_default_agents() {
        let mut config = OsrsWikiConfig::grandedge_default().unwrap();
        config.user_agent = "python-requests/2.32 GrandEdge scott.rangeley@outlook.com".into();
        assert!(config.validate().is_err());
    }
}
