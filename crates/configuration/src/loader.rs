use std::path::PathBuf;

use clap::ValueEnum;
use config::{Environment, File, FileFormat};

use crate::{ConfigurationError, GrandEdgeConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ConfigProfile {
    Local,
    Test,
    Production,
}

pub fn load_config(profile: ConfigProfile) -> Result<GrandEdgeConfig, ConfigurationError> {
    let _ = dotenvy::dotenv();
    let root = repo_root()?;
    let config_dir = root.join("configs");
    let profile_name = profile.as_str();

    let mut builder = config::Config::builder()
        .add_source(File::from(config_dir.join("default.toml")).format(FileFormat::Toml))
        .add_source(
            File::from(config_dir.join(format!("{profile_name}.toml")))
                .format(FileFormat::Toml)
                .required(false),
        )
        .add_source(
            File::from(config_dir.join(format!("{profile_name}.local.toml")))
                .format(FileFormat::Toml)
                .required(false),
        )
        .add_source(
            File::from(config_dir.join("local.toml"))
                .format(FileFormat::Toml)
                .required(false),
        )
        .add_source(Environment::with_prefix("GRAND_EDGE").separator("__"));

    if let Ok(value) = std::env::var("DATABASE_URL") {
        builder = builder.set_override("database.url", value)?;
    }
    if let Ok(value) = std::env::var("GRAND_EDGE_USER_AGENT") {
        builder = builder.set_override("osrs_wiki.user_agent", value)?;
    }
    if let Ok(value) = std::env::var("OSRS_WIKI_BASE_URL") {
        builder = builder.set_override("osrs_wiki.base_url", value)?;
    }

    let config = builder.build()?.try_deserialize::<GrandEdgeConfig>()?;
    config.validate(profile)?;
    Ok(config)
}

impl ConfigProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Test => "test",
            Self::Production => "production",
        }
    }
}

fn repo_root() -> Result<PathBuf, ConfigurationError> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use super::{ConfigProfile, load_config};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn loads_default_config() {
        let _guard = env_lock().lock().unwrap();
        let config = load_config(ConfigProfile::Test).unwrap();
        assert_eq!(config.osrs_wiki.app_name, "GrandEdge");
        assert_eq!(config.ingest.poll_latest_seconds, 60);
    }

    #[test]
    fn env_overrides_nested_config() {
        let _guard = env_lock().lock().unwrap();
        unsafe {
            std::env::set_var("GRAND_EDGE__API__BIND_ADDR", "127.0.0.1:3456");
        }
        let config = load_config(ConfigProfile::Test).unwrap();
        unsafe {
            std::env::remove_var("GRAND_EDGE__API__BIND_ADDR");
        }
        assert_eq!(config.api.bind_addr.to_string(), "127.0.0.1:3456");
    }
}
