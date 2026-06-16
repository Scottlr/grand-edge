//! Recommendation orchestration for Grand Edge.

pub mod actions;
pub mod config;
pub mod engine;
pub mod errors;
pub mod explanations;
pub mod quantity;
pub mod scoring;

pub use config::RecommendationConfig;
pub use engine::{RecommendationEngine, RecommendationInput};
pub use errors::RecommendationError;
pub use scoring::{RecommendationScore, ScoreComponent};
