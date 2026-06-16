use std::net::SocketAddr;

use grand_edge_configuration::GrandEdgeConfig;
use secrecy::ExposeSecret;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub cors_origin: Option<String>,
    pub default_user_id: Option<Uuid>,
}

impl ApiConfig {
    pub fn from_runtime(config: &GrandEdgeConfig) -> Self {
        Self {
            bind_addr: config.api.bind_addr,
            database_url: config.database.url.expose_secret().to_string(),
            cors_origin: config.api.cors_origin.clone(),
            default_user_id: config.api.default_user_id,
        }
    }
}
