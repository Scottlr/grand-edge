use std::{net::SocketAddr, time::Duration};

use grand_edge_configuration::GrandEdgeConfig;
use secrecy::ExposeSecret;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApiAuthConfig {
    pub session_secret: String,
    pub session_ttl: Duration,
    pub local_default_user_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub cors_origin: Option<String>,
    pub default_user_id: Option<Uuid>,
    pub swagger_ui_enabled: bool,
    pub auth: ApiAuthConfig,
}

impl ApiConfig {
    pub fn from_runtime(config: &GrandEdgeConfig) -> Self {
        Self {
            bind_addr: config.api.bind_addr,
            database_url: config.database.url.expose_secret().to_string(),
            cors_origin: config.api.cors_origin.clone(),
            default_user_id: config.api.default_user_id,
            swagger_ui_enabled: config.api.swagger_ui_enabled,
            auth: ApiAuthConfig {
                session_secret: config.auth.session_secret.expose_secret().to_string(),
                session_ttl: Duration::from_secs(config.auth.session_ttl_hours * 60 * 60),
                local_default_user_enabled: config.auth.local_default_user_enabled,
            },
        }
    }
}
