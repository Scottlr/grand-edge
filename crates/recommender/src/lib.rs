//! Recommendation orchestration for Grand Edge.

pub mod actions;
pub mod confidence;
pub mod config;
pub mod engine;
pub mod errors;
pub mod explanations;
pub mod graph_actions;
pub mod prediction_links;
pub mod quantity;
pub mod reason_atoms;
pub mod scoring;

pub use config::RecommendationConfig;
pub use engine::{RecommendationEngine, RecommendationInput};
pub use errors::RecommendationError;
pub use prediction_links::{
    build_prediction_links, compatibility_feature_snapshot, persist_recommendation_decision,
    prediction_contributions,
};
pub use scoring::{RecommendationScore, ScoreComponent};
