//! Deterministic feature computation for Grand Edge.

pub mod calculations;
pub mod config;
pub mod engine;
pub mod errors;
pub mod fixtures;
pub mod snapshot;

pub use config::FeatureEngineConfig;
pub use engine::{FEATURE_SET_VERSION, FeatureEngine};
pub use errors::FeatureError;
pub use snapshot::ItemFeatureInput;
