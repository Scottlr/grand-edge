use secrecy::{ExposeSecret, SecretString};

use crate::GrandEdgeConfig;

pub fn redact_secret(secret: &SecretString) -> &'static str {
    if secret.expose_secret().is_empty() {
        "<empty>"
    } else {
        "<redacted>"
    }
}

pub fn redacted_config_summary(config: &GrandEdgeConfig) -> String {
    format!(
        "database.url={}\nauth.session_key={}\nosrs_wiki.user_agent={}\napi.bind_addr={}",
        redact_secret(&config.database.url),
        redact_secret(&config.auth.session_key),
        config.osrs_wiki.user_agent,
        config.api.bind_addr
    )
}

#[cfg(test)]
mod tests {
    use crate::{ConfigProfile, load_config};

    use super::redacted_config_summary;

    #[test]
    fn config_print_redacts_secrets() {
        let config = load_config(ConfigProfile::Test).unwrap();
        let summary = redacted_config_summary(&config);
        assert!(summary.contains("<redacted>"));
        assert!(!summary.contains("postgres://"));
    }
}
