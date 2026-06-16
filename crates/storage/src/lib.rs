//! Storage ownership for Grand Edge persistence.

pub mod checkpoints;
pub mod corpus_sources;
pub mod errors;
pub mod evidence;
pub mod features;
pub mod graph;
pub mod items;
pub mod market_events;
pub mod metrics;
pub mod outcomes;
pub mod pool;
pub mod positions;
pub mod predictions;
pub mod prices;
pub mod reason_outcomes;
pub mod recommendations;
pub mod simulations;
pub mod strategies;

pub use checkpoints::{CheckpointRepository, StoredCheckpoint};
pub use corpus_sources::{CorpusSourceRepository, StoredCorpusSource};
pub use errors::StorageError;
pub use evidence::{
    EvidenceRepository, LinkedPredictionRecord, RecommendationEvidenceRecord,
    RecommendationGraphEvidence, RecommendationGraphLinkSummary,
};
pub use features::FeatureRepository;
pub use graph::{GraphRepository, RecommendationGraphLinkRecord};
pub use items::ItemRepository;
pub use market_events::{MarketEventItemLink, MarketEventRepository, StoredMarketEvent};
pub use metrics::MetricsRepository;
pub use outcomes::OutcomeRepository;
pub use pool::Storage;
pub use positions::PositionRepository;
pub use predictions::PredictionRepository;
pub use prices::PriceRepository;
pub use reason_outcomes::ReasonOutcomeRepository;
pub use recommendations::{EvaluatedRecommendationRecord, RecommendationRepository};
pub use simulations::{SimulationRepository, StoredPaperBet, StoredSimulationRun};
pub use strategies::{StoredPrediction, StrategyRepository};
