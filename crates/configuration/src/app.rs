use std::{fmt, net::SocketAddr, time::Duration};

use secrecy::SecretString;
use serde::Deserialize;

use crate::ConfigurationError;

#[derive(Debug, Clone, Deserialize)]
pub struct GrandEdgeConfig {
    pub database: DatabaseConfig,
    pub osrs_wiki: OsrsWikiRuntimeConfig,
    pub ingest: IngestRuntimeConfig,
    pub api: ApiRuntimeConfig,
    pub logging: LoggingConfig,
    pub artifacts: ArtifactRuntimeConfig,
    pub auth: AuthRuntimeConfig,
}

#[derive(Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: SecretString,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OsrsWikiRuntimeConfig {
    pub base_url: String,
    pub app_name: String,
    pub contact_email: String,
    pub user_agent: String,
    pub request_timeout_ms: u64,
    pub max_requests_per_second: f64,
    pub burst_size: u32,
    pub respect_retry_after: bool,
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngestRuntimeConfig {
    pub poll_latest_seconds: u64,
    pub poll_5m_seconds: u64,
    pub poll_1h_seconds: u64,
    pub sync_mapping_seconds: u64,
    pub max_timeseries_items_per_run: usize,
    pub max_timeseries_requests_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiRuntimeConfig {
    pub bind_addr: SocketAddr,
    pub cors_origin: Option<String>,
    pub default_user_id: Option<uuid::Uuid>,
    #[serde(default = "default_swagger_ui_enabled")]
    pub swagger_ui_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub format: LogFormat,
    pub level: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Clone, Deserialize)]
pub struct ArtifactRuntimeConfig {
    pub root_dir: String,
    pub registry_token: Option<SecretString>,
}

#[derive(Clone, Deserialize)]
pub struct AuthRuntimeConfig {
    pub session_key: SecretString,
    pub jwt_issuer: String,
}

impl GrandEdgeConfig {
    pub fn validate(
        &self,
        profile: crate::loader::ConfigProfile,
    ) -> Result<(), ConfigurationError> {
        validate_user_agent(&self.osrs_wiki.user_agent)?;

        if !self.osrs_wiki.respect_retry_after {
            return Err(ConfigurationError::Invalid(
                "osrs_wiki.respect_retry_after must remain true".to_string(),
            ));
        }
        if self.osrs_wiki.max_requests_per_second <= 0.0 {
            return Err(ConfigurationError::Invalid(
                "osrs_wiki.max_requests_per_second must be > 0".to_string(),
            ));
        }
        if self.osrs_wiki.burst_size == 0 {
            return Err(ConfigurationError::Invalid(
                "osrs_wiki.burst_size must be > 0".to_string(),
            ));
        }

        validate_ingest_defaults(&self.ingest)?;

        if matches!(profile, crate::loader::ConfigProfile::Production)
            && (self.osrs_wiki.user_agent.contains("replace-me")
                || self.auth.jwt_issuer.contains("replace-me"))
        {
            return Err(ConfigurationError::Invalid(
                "production config must not use replace-me placeholders".to_string(),
            ));
        }

        Ok(())
    }

    pub fn osrs_request_timeout(&self) -> Duration {
        Duration::from_millis(self.osrs_wiki.request_timeout_ms)
    }
}

fn default_swagger_ui_enabled() -> bool {
    true
}

impl OsrsWikiRuntimeConfig {
    pub fn to_ingest_config(
        &self,
    ) -> Result<grand_edge_ingest::OsrsWikiConfig, ConfigurationError> {
        let config = grand_edge_ingest::OsrsWikiConfig {
            base_url: reqwest::Url::parse(&self.base_url)
                .map_err(|error| ConfigurationError::Invalid(error.to_string()))?,
            app_name: self.app_name.clone(),
            contact_email: self.contact_email.clone(),
            user_agent: self.user_agent.clone(),
            request_timeout: Duration::from_millis(self.request_timeout_ms),
            rate_limit: grand_edge_ingest::OsrsWikiRateLimitConfig {
                max_requests_per_second: self.max_requests_per_second,
                burst_size: self.burst_size,
                respect_retry_after: self.respect_retry_after,
                max_retries: self.max_retries,
                initial_backoff: Duration::from_millis(self.initial_backoff_ms),
                max_backoff: Duration::from_millis(self.max_backoff_ms),
            },
        };
        config.validate()?;
        Ok(config)
    }
}

impl fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("url", &"<redacted>")
            .field("max_connections", &self.max_connections)
            .finish()
    }
}

impl fmt::Debug for ArtifactRuntimeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArtifactRuntimeConfig")
            .field("root_dir", &self.root_dir)
            .field(
                "registry_token",
                &self.registry_token.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl fmt::Debug for AuthRuntimeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthRuntimeConfig")
            .field("session_key", &"<redacted>")
            .field("jwt_issuer", &self.jwt_issuer)
            .finish()
    }
}

fn validate_user_agent(user_agent: &str) -> Result<(), ConfigurationError> {
    if !user_agent.contains("GrandEdge") {
        return Err(ConfigurationError::Invalid(
            "osrs_wiki.user_agent must contain GrandEdge".to_string(),
        ));
    }
    if !user_agent.contains("scott.rangeley@outlook.com") {
        return Err(ConfigurationError::Invalid(
            "osrs_wiki.user_agent must contain scott.rangeley@outlook.com".to_string(),
        ));
    }

    Ok(())
}

fn validate_ingest_defaults(config: &IngestRuntimeConfig) -> Result<(), ConfigurationError> {
    if config.poll_latest_seconds < 60 {
        return Err(ConfigurationError::Invalid(
            "ingest.poll_latest_seconds must be >= 60".to_string(),
        ));
    }
    if config.poll_5m_seconds < 300 {
        return Err(ConfigurationError::Invalid(
            "ingest.poll_5m_seconds must be >= 300".to_string(),
        ));
    }
    if config.poll_1h_seconds < 3600 {
        return Err(ConfigurationError::Invalid(
            "ingest.poll_1h_seconds must be >= 3600".to_string(),
        ));
    }
    if config.sync_mapping_seconds < 3600 {
        return Err(ConfigurationError::Invalid(
            "ingest.sync_mapping_seconds must be >= 3600".to_string(),
        ));
    }
    if config.max_timeseries_requests_per_minute == 0 {
        return Err(ConfigurationError::Invalid(
            "ingest.max_timeseries_requests_per_minute must be > 0".to_string(),
        ));
    }

    Ok(())
}
