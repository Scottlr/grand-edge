use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{
    AuthRepository, CheckpointRepository, CorpusSourceRepository, EvidenceRepository,
    FeatureRepository, GraphRepository, ItemRepository, MarketEventRepository, MetricsRepository,
    OutcomeRepository, PositionRepository, PredictionRepository, PriceRepository,
    ReasonOutcomeRepository, RecommendationRepository, SimulationRepository, StorageError,
    StrategyRepository,
};

#[derive(Clone)]
pub struct Storage {
    pool: PgPool,
}

impl Storage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(database_url: &str) -> Result<Self, StorageError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self::new(pool))
    }

    pub async fn migrate(&self) -> Result<(), StorageError> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn items(&self) -> ItemRepository {
        ItemRepository::new(self.pool.clone())
    }

    pub fn auth(&self) -> AuthRepository {
        AuthRepository::new(self.pool.clone())
    }

    pub fn checkpoints(&self) -> CheckpointRepository {
        CheckpointRepository::new(self.pool.clone())
    }

    pub fn prices(&self) -> PriceRepository {
        PriceRepository::new(self.pool.clone())
    }

    pub fn features(&self) -> FeatureRepository {
        FeatureRepository::new(self.pool.clone())
    }

    pub fn evidence(&self) -> EvidenceRepository {
        EvidenceRepository::new(self.pool.clone())
    }

    pub fn graph(&self) -> GraphRepository {
        GraphRepository::new(self.pool.clone())
    }

    pub fn market_events(&self) -> MarketEventRepository {
        MarketEventRepository::new(self.pool.clone())
    }

    pub fn corpus_sources(&self) -> CorpusSourceRepository {
        CorpusSourceRepository::new(self.pool.clone())
    }

    pub fn predictions(&self) -> PredictionRepository {
        PredictionRepository::new(self.pool.clone())
    }

    pub fn strategies(&self) -> StrategyRepository {
        StrategyRepository::new(self.pool.clone())
    }

    pub fn recommendations(&self) -> RecommendationRepository {
        RecommendationRepository::new(self.pool.clone())
    }

    pub fn outcomes(&self) -> OutcomeRepository {
        OutcomeRepository::new(self.pool.clone())
    }

    pub fn reason_outcomes(&self) -> ReasonOutcomeRepository {
        ReasonOutcomeRepository::new(self.pool.clone())
    }

    pub fn positions(&self) -> PositionRepository {
        PositionRepository::new(self.pool.clone())
    }

    pub fn simulations(&self) -> SimulationRepository {
        SimulationRepository::new(self.pool.clone())
    }

    pub fn metrics(&self) -> MetricsRepository {
        MetricsRepository::new(self.pool.clone())
    }
}
