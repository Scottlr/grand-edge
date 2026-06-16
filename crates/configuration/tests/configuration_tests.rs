use std::sync::{Mutex, OnceLock};

use grand_edge_configuration::{ConfigProfile, load_config};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn default_osrs_wiki_user_agent_contains_grandedge_contact_email() {
    let _guard = env_lock().lock().unwrap();
    let config = load_config(ConfigProfile::Test).unwrap();
    assert!(config.osrs_wiki.user_agent.contains("GrandEdge"));
    assert!(
        config
            .osrs_wiki
            .user_agent
            .contains("scott.rangeley@outlook.com")
    );
}

#[test]
fn rejects_user_agent_missing_contact_email() {
    let _guard = env_lock().lock().unwrap();
    unsafe {
        std::env::set_var(
            "GRAND_EDGE__OSRS_WIKI__USER_AGENT",
            "GrandEdge/0.1 (missing contact)",
        );
    }
    let result = load_config(ConfigProfile::Test);
    unsafe {
        std::env::remove_var("GRAND_EDGE__OSRS_WIKI__USER_AGENT");
    }
    assert!(result.is_err());
}

#[test]
fn production_rejects_replace_me_user_agent() {
    let _guard = env_lock().lock().unwrap();
    unsafe {
        std::env::set_var(
            "GRAND_EDGE__OSRS_WIKI__USER_AGENT",
            "GrandEdge replace-me scott.rangeley@outlook.com",
        );
    }
    let result = load_config(ConfigProfile::Production);
    unsafe {
        std::env::remove_var("GRAND_EDGE__OSRS_WIKI__USER_AGENT");
    }
    assert!(result.is_err());
}

#[test]
fn rejects_unsafe_osrs_wiki_rate_limit_config() {
    let _guard = env_lock().lock().unwrap();
    unsafe {
        std::env::set_var("GRAND_EDGE__OSRS_WIKI__MAX_REQUESTS_PER_SECOND", "0");
    }
    let result = load_config(ConfigProfile::Test);
    unsafe {
        std::env::remove_var("GRAND_EDGE__OSRS_WIKI__MAX_REQUESTS_PER_SECOND");
    }
    assert!(result.is_err());
}

#[test]
fn osrs_runtime_config_converts_to_ingest_config() {
    let _guard = env_lock().lock().unwrap();
    let config = load_config(ConfigProfile::Test).unwrap();
    let ingest = config.osrs_wiki.to_ingest_config().unwrap();
    assert_eq!(ingest.user_agent, config.osrs_wiki.user_agent);
    assert!(ingest.validate().is_ok());
}
