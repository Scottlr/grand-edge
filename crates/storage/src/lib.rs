//! Storage ownership for Grand Edge persistence.

pub mod errors;
pub mod features;
pub mod items;
pub mod metrics;
pub mod pool;
pub mod positions;
pub mod prices;
pub mod recommendations;
pub mod simulations;
pub mod strategies;

pub use errors::StorageError;
pub use features::FeatureRepository;
pub use items::ItemRepository;
pub use metrics::MetricsRepository;
pub use pool::Storage;
pub use positions::PositionRepository;
pub use prices::PriceRepository;
pub use recommendations::RecommendationRepository;
pub use simulations::{SimulationRepository, StoredPaperBet};
pub use strategies::{StoredPrediction, StrategyRepository};
