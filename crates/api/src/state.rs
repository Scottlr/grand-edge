use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grand_edge_domain::{
    Gp, Item, ItemId, PositionId, PriceInterval, Quantity, Recommendation, RecommendationAction,
    RecommendationExplanation, RunId, UserId, UserPosition,
};
use grand_edge_metrics::MetricsEngine;
use grand_edge_recommender::{RecommendationConfig, RecommendationEngine};
use grand_edge_simulator::{SimulationEngine, SimulatorConfig};
use grand_edge_storage::{Storage, StoredSimulationRun};
use grand_edge_strategies::{StrategyConfig, StrategyRegistry, register_baseline_strategies};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::{config::ApiConfig, errors::ApiError, routes::live::LiveEvent};

#[derive(Debug, Clone)]
pub struct StrategyStatusRecord {
    pub strategy_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PositionUpsert {
    pub item_id: i64,
    pub quantity: i64,
    pub avg_buy_price: i64,
    pub bought_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SimulationRunDraft {
    pub name: String,
    pub strategy_config: serde_json::Value,
}

#[async_trait]
pub trait ApiServices: Send + Sync {
    async fn list_items(&self, limit: i64, offset: i64) -> Result<Vec<Item>, ApiError>;
    async fn get_item(&self, item_id: ItemId) -> Result<Option<Item>, ApiError>;
    async fn item_history(
        &self,
        item_id: ItemId,
        interval: PriceInterval,
        limit: i64,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<grand_edge_domain::IntervalPrice>, ApiError>;
    async fn list_recommendations(
        &self,
        user_id: Option<UserId>,
        action: Option<RecommendationAction>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Recommendation>, ApiError>;
    async fn get_recommendation_explanation(
        &self,
        recommendation_id: grand_edge_domain::RecommendationId,
    ) -> Result<Option<RecommendationExplanation>, ApiError>;
    async fn list_strategies(&self) -> Result<Vec<StrategyStatusRecord>, ApiError>;
    async fn patch_strategy(
        &self,
        strategy_id: &str,
        enabled: bool,
    ) -> Result<Option<StrategyStatusRecord>, ApiError>;
    async fn list_simulation_runs(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredSimulationRun>, ApiError>;
    async fn create_simulation_run(
        &self,
        draft: SimulationRunDraft,
    ) -> Result<StoredSimulationRun, ApiError>;
    async fn list_positions(&self) -> Result<Vec<UserPosition>, ApiError>;
    async fn create_position(&self, input: PositionUpsert) -> Result<UserPosition, ApiError>;
    async fn update_position(
        &self,
        position_id: PositionId,
        input: PositionUpsert,
    ) -> Result<Option<UserPosition>, ApiError>;
}

#[derive(Clone)]
pub struct AppState {
    pub services: Arc<dyn ApiServices>,
    pub live_events: LiveEventBus,
}

impl AppState {
    pub fn new(services: Arc<dyn ApiServices>, live_events: LiveEventBus) -> Self {
        Self {
            services,
            live_events,
        }
    }

    pub async fn from_config(config: ApiConfig) -> Result<Self, ApiError> {
        let services = RuntimeServices::from_config(&config).await?;
        Ok(Self::new(Arc::new(services), LiveEventBus::default()))
    }
}

#[derive(Clone)]
pub struct LiveEventBus {
    tx: broadcast::Sender<LiveEvent>,
}

impl Default for LiveEventBus {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self { tx }
    }
}

impl LiveEventBus {
    pub fn publish(&self, event: LiveEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LiveEvent> {
        self.tx.subscribe()
    }
}

pub struct RuntimeServices {
    storage: Storage,
    #[allow(dead_code)]
    recommender: Arc<RecommendationEngine>,
    simulator: Arc<SimulationEngine>,
    strategy_registry: Arc<RwLock<StrategyRegistry>>,
    strategy_config: Arc<RwLock<StrategyConfig>>,
    default_user_id: Option<UserId>,
}

impl RuntimeServices {
    async fn from_config(config: &ApiConfig) -> Result<Self, ApiError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&config.database_url)
            .map_err(|error| ApiError::Config(error.to_string()))?;
        let storage = Storage::new(pool);

        let metrics = MetricsEngine::new(storage.clone());
        let simulator = Arc::new(SimulationEngine::new(
            storage.clone(),
            SimulatorConfig::default(),
        ));
        let recommender = Arc::new(RecommendationEngine::new(
            storage.clone(),
            metrics,
            SimulationEngine::new(storage.clone(), SimulatorConfig::default()),
            RecommendationConfig::default(),
        ));

        let mut registry = StrategyRegistry::new();
        register_baseline_strategies(&mut registry)?;
        let strategy_config = StrategyConfig::default();

        Ok(Self {
            storage,
            recommender,
            simulator,
            strategy_registry: Arc::new(RwLock::new(registry)),
            strategy_config: Arc::new(RwLock::new(strategy_config)),
            default_user_id: config.default_user_id.map(UserId),
        })
    }

    fn require_default_user_id(&self) -> Result<UserId, ApiError> {
        self.default_user_id
            .ok_or_else(|| ApiError::Config("default user id is not configured".to_string()))
    }
}

#[async_trait]
impl ApiServices for RuntimeServices {
    async fn list_items(&self, limit: i64, offset: i64) -> Result<Vec<Item>, ApiError> {
        Ok(self.storage.items().list_items(limit, offset).await?)
    }

    async fn get_item(&self, item_id: ItemId) -> Result<Option<Item>, ApiError> {
        Ok(self.storage.items().get_item(item_id).await?)
    }

    async fn item_history(
        &self,
        item_id: ItemId,
        interval: PriceInterval,
        limit: i64,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<grand_edge_domain::IntervalPrice>, ApiError> {
        Ok(self
            .storage
            .prices()
            .interval_history_before(item_id, interval, limit, before)
            .await?)
    }

    async fn list_recommendations(
        &self,
        user_id: Option<UserId>,
        action: Option<RecommendationAction>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Recommendation>, ApiError> {
        Ok(self
            .storage
            .recommendations()
            .list_recent(user_id, action, limit, offset)
            .await?)
    }

    async fn get_recommendation_explanation(
        &self,
        recommendation_id: grand_edge_domain::RecommendationId,
    ) -> Result<Option<RecommendationExplanation>, ApiError> {
        Ok(self
            .storage
            .recommendations()
            .get_recommendation(recommendation_id)
            .await?
            .map(|recommendation| recommendation.explanation))
    }

    async fn list_strategies(&self) -> Result<Vec<StrategyStatusRecord>, ApiError> {
        let registry = self.strategy_registry.read().await;
        let config = self.strategy_config.read().await;
        let enabled = config
            .enabled_strategies
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        Ok(registry
            .ids()
            .into_iter()
            .map(|strategy_id| StrategyStatusRecord {
                enabled: enabled.contains(&strategy_id),
                strategy_id,
            })
            .collect())
    }

    async fn patch_strategy(
        &self,
        strategy_id: &str,
        enabled: bool,
    ) -> Result<Option<StrategyStatusRecord>, ApiError> {
        let registry = self.strategy_registry.read().await;
        if registry.get(strategy_id).is_none() {
            return Ok(None);
        }
        drop(registry);

        let mut config = self.strategy_config.write().await;
        if enabled {
            if !config
                .enabled_strategies
                .iter()
                .any(|value| value == strategy_id)
            {
                config.enabled_strategies.push(strategy_id.to_string());
                config.enabled_strategies.sort();
            }
        } else {
            config
                .enabled_strategies
                .retain(|value| value != strategy_id);
        }

        Ok(Some(StrategyStatusRecord {
            strategy_id: strategy_id.to_string(),
            enabled,
        }))
    }

    async fn list_simulation_runs(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredSimulationRun>, ApiError> {
        Ok(self.storage.simulations().list_runs(limit, offset).await?)
    }

    async fn create_simulation_run(
        &self,
        draft: SimulationRunDraft,
    ) -> Result<StoredSimulationRun, ApiError> {
        let run_id = self
            .simulator
            .create_run(&draft.name, draft.strategy_config)
            .await?;

        self.storage
            .simulations()
            .get_run(RunId(run_id))
            .await?
            .ok_or_else(|| ApiError::Internal("simulation run was not persisted".to_string()))
    }

    async fn list_positions(&self) -> Result<Vec<UserPosition>, ApiError> {
        Ok(self
            .storage
            .positions()
            .active_positions_for_user(self.require_default_user_id()?)
            .await?)
    }

    async fn create_position(&self, input: PositionUpsert) -> Result<UserPosition, ApiError> {
        let position = UserPosition {
            position_id: PositionId(Uuid::new_v4()),
            user_id: self.require_default_user_id()?,
            item_id: ItemId::try_from(input.item_id)?,
            quantity: Quantity::try_from(input.quantity)?,
            avg_buy_price: Gp::try_from(input.avg_buy_price)?,
            bought_at: input.bought_at,
            notes: input.notes,
        };
        self.storage
            .positions()
            .upsert_positions(std::slice::from_ref(&position))
            .await?;
        Ok(position)
    }

    async fn update_position(
        &self,
        position_id: PositionId,
        input: PositionUpsert,
    ) -> Result<Option<UserPosition>, ApiError> {
        let Some(existing) = self.storage.positions().get_position(position_id).await? else {
            return Ok(None);
        };

        let position = UserPosition {
            position_id,
            user_id: existing.user_id,
            item_id: ItemId::try_from(input.item_id)?,
            quantity: Quantity::try_from(input.quantity)?,
            avg_buy_price: Gp::try_from(input.avg_buy_price)?,
            bought_at: input.bought_at,
            notes: input.notes,
        };
        self.storage
            .positions()
            .upsert_positions(std::slice::from_ref(&position))
            .await?;
        Ok(Some(position))
    }
}
