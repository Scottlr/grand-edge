use std::{collections::HashSet, sync::Arc};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use grand_edge_domain::{
    AuthSession, AuthenticatedUser, Gp, Item, ItemId, LoginRequest, PositionId, PriceInterval,
    Quantity, Recommendation, RecommendationAction, RegisterRequest, RunId, SessionId,
    UpdateRiskProfile, UserId, UserPosition, UserRiskProfile,
};
use grand_edge_recommender::RecommendationConfig;
use grand_edge_simulator::{SimulationEngine, SimulatorConfig};
use grand_edge_storage::{NewUserIdentity, Storage, StoredSimulationRun};
use grand_edge_strategies::{StrategyConfig, StrategyRegistry, register_baseline_strategies};
use rand_core::OsRng;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::{
    config::{ApiAuthConfig, ApiConfig},
    errors::ApiError,
    routes::live::LiveEvent,
};

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

#[derive(Debug, Clone)]
pub struct AuthSessionCookie {
    pub session_id: SessionId,
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
    ) -> Result<Option<Recommendation>, ApiError>;
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
    async fn register(&self, request: RegisterRequest) -> Result<AuthenticatedUser, ApiError>;
    async fn login(
        &self,
        request: LoginRequest,
    ) -> Result<(AuthenticatedUser, AuthSessionCookie), ApiError>;
    async fn logout(&self, session_id: SessionId) -> Result<(), ApiError>;
    async fn current_user(
        &self,
        session_id: SessionId,
    ) -> Result<Option<AuthenticatedUser>, ApiError>;
    async fn local_default_user(&self) -> Result<Option<UserId>, ApiError>;
    async fn get_risk_profile(&self, user_id: UserId) -> Result<UserRiskProfile, ApiError>;
    async fn update_risk_profile(
        &self,
        user_id: UserId,
        update: UpdateRiskProfile,
    ) -> Result<UserRiskProfile, ApiError>;
    async fn list_positions(&self, user_id: UserId) -> Result<Vec<UserPosition>, ApiError>;
    async fn create_position(
        &self,
        user_id: UserId,
        input: PositionUpsert,
    ) -> Result<UserPosition, ApiError>;
    async fn update_position(
        &self,
        user_id: UserId,
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
    simulator: Arc<SimulationEngine>,
    strategy_registry: Arc<RwLock<StrategyRegistry>>,
    strategy_config: Arc<RwLock<StrategyConfig>>,
    default_user_id: Option<UserId>,
    auth: ApiAuthConfig,
}

impl RuntimeServices {
    async fn from_config(config: &ApiConfig) -> Result<Self, ApiError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&config.database_url)
            .map_err(|error| ApiError::Config(error.to_string()))?;
        let storage = Storage::new(pool);

        let simulator = Arc::new(SimulationEngine::new(
            storage.clone(),
            SimulatorConfig::default(),
        ));

        let mut registry = StrategyRegistry::new();
        register_baseline_strategies(&mut registry)?;
        let strategy_config = StrategyConfig::default();

        Ok(Self {
            storage,
            simulator,
            strategy_registry: Arc::new(RwLock::new(registry)),
            strategy_config: Arc::new(RwLock::new(strategy_config)),
            default_user_id: config.default_user_id.map(UserId),
            auth: config.auth.clone(),
        })
    }

    fn hash_password(&self, password: &secrecy::SecretString) -> Result<String, ApiError> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Argon2::default()
            .hash_password(password.expose_secret().as_bytes(), &salt)
            .map_err(|error| ApiError::Internal(error.to_string()))?
            .to_string())
    }

    fn verify_password(
        &self,
        password: &secrecy::SecretString,
        expected_hash: &str,
    ) -> Result<bool, ApiError> {
        let hash = PasswordHash::new(expected_hash)
            .map_err(|error| ApiError::Internal(error.to_string()))?;
        Ok(Argon2::default()
            .verify_password(password.expose_secret().as_bytes(), &hash)
            .is_ok())
    }

    fn issue_session(&self, user_id: UserId) -> AuthSession {
        let created_at = Utc::now();
        let ttl = chrono::Duration::from_std(self.auth.session_ttl)
            .unwrap_or_else(|_| chrono::Duration::hours(24));
        AuthSession {
            session_id: SessionId(Uuid::new_v4()),
            user_id,
            created_at,
            expires_at: created_at + ttl,
        }
    }

    async fn recommendation_config_for_user(
        &self,
        user_id: Option<UserId>,
    ) -> Result<RecommendationConfig, ApiError> {
        let mut config = RecommendationConfig::default();
        if let Some(user_id) = user_id {
            let profile = self
                .storage
                .auth()
                .get_risk_profile(user_id)
                .await?
                .unwrap_or_else(|| UserRiskProfile::default_for_user(user_id, Utc::now()));
            config.min_expected_roi = profile.min_expected_roi.get();
            config.min_confidence = profile.min_confidence.get();
            config.min_execution_confidence = profile.min_confidence.get().min(0.95);
        }
        Ok(config)
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
        let config = self.recommendation_config_for_user(user_id).await?;
        let recommendations = self
            .storage
            .recommendations()
            .list_recent(user_id, action, limit, offset)
            .await?;
        Ok(recommendations
            .into_iter()
            .filter(|recommendation| recommendation_matches_risk_profile(recommendation, &config))
            .collect())
    }

    async fn get_recommendation_explanation(
        &self,
        recommendation_id: grand_edge_domain::RecommendationId,
    ) -> Result<Option<Recommendation>, ApiError> {
        Ok(self
            .storage
            .recommendations()
            .get_recommendation(recommendation_id)
            .await?)
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

    async fn register(&self, request: RegisterRequest) -> Result<AuthenticatedUser, ApiError> {
        if self
            .storage
            .auth()
            .get_user_by_email(&request.email)
            .await?
            .is_some()
        {
            return Err(ApiError::BadRequest(
                "an account with that email already exists".to_string(),
            ));
        }

        let identity = NewUserIdentity {
            user_id: UserId(Uuid::new_v4()),
            email: request.email,
            password_hash: self.hash_password(&request.password)?,
            display_name: request.display_name,
            created_at: Utc::now(),
        };
        self.storage
            .auth()
            .create_user_identity(&identity)
            .await
            .map_err(Into::into)
    }

    async fn login(
        &self,
        request: LoginRequest,
    ) -> Result<(AuthenticatedUser, AuthSessionCookie), ApiError> {
        let Some((user, password_hash)) = self
            .storage
            .auth()
            .get_user_by_email(&request.email)
            .await?
        else {
            return Err(ApiError::Unauthorized(
                "invalid email or password".to_string(),
            ));
        };
        if !self.verify_password(&request.password, &password_hash)? {
            return Err(ApiError::Unauthorized(
                "invalid email or password".to_string(),
            ));
        }

        let session = self.issue_session(user.user_id);
        self.storage.auth().create_session(&session).await?;
        Ok((
            user,
            AuthSessionCookie {
                session_id: session.session_id,
            },
        ))
    }

    async fn logout(&self, session_id: SessionId) -> Result<(), ApiError> {
        self.storage.auth().revoke_session(session_id).await?;
        Ok(())
    }

    async fn current_user(
        &self,
        session_id: SessionId,
    ) -> Result<Option<AuthenticatedUser>, ApiError> {
        let Some(session) = self.storage.auth().get_active_session(session_id).await? else {
            return Ok(None);
        };
        self.storage
            .auth()
            .get_user_by_id(session.user_id)
            .await
            .map_err(Into::into)
    }

    async fn local_default_user(&self) -> Result<Option<UserId>, ApiError> {
        if self.auth.local_default_user_enabled {
            Ok(self.default_user_id)
        } else {
            Ok(None)
        }
    }

    async fn get_risk_profile(&self, user_id: UserId) -> Result<UserRiskProfile, ApiError> {
        Ok(self
            .storage
            .auth()
            .get_risk_profile(user_id)
            .await?
            .unwrap_or_else(|| UserRiskProfile::default_for_user(user_id, Utc::now())))
    }

    async fn update_risk_profile(
        &self,
        user_id: UserId,
        update: UpdateRiskProfile,
    ) -> Result<UserRiskProfile, ApiError> {
        self.storage
            .auth()
            .update_risk_profile(user_id, update)
            .await
            .map_err(Into::into)
    }

    async fn list_positions(&self, user_id: UserId) -> Result<Vec<UserPosition>, ApiError> {
        Ok(self
            .storage
            .positions()
            .active_positions_for_user(user_id)
            .await?)
    }

    async fn create_position(
        &self,
        user_id: UserId,
        input: PositionUpsert,
    ) -> Result<UserPosition, ApiError> {
        let position = UserPosition {
            position_id: PositionId(Uuid::new_v4()),
            user_id,
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
        user_id: UserId,
        position_id: PositionId,
        input: PositionUpsert,
    ) -> Result<Option<UserPosition>, ApiError> {
        let Some(existing) = self.storage.positions().get_position(position_id).await? else {
            return Ok(None);
        };
        if existing.user_id != user_id {
            return Err(ApiError::NotFound(format!(
                "position {} was not found",
                position_id.0
            )));
        }

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

fn recommendation_matches_risk_profile(
    recommendation: &Recommendation,
    config: &RecommendationConfig,
) -> bool {
    if matches!(
        recommendation.action,
        RecommendationAction::Cashout | RecommendationAction::Hold
    ) {
        return true;
    }

    let meets_confidence = recommendation.recommendation_confidence.get() >= config.min_confidence;
    let meets_roi = recommendation
        .expected_roi
        .map(|value| value.get() >= config.min_expected_roi)
        .unwrap_or(false);

    if matches!(
        recommendation.action,
        RecommendationAction::Buy | RecommendationAction::Add
    ) {
        return meets_confidence && meets_roi;
    }

    true
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use grand_edge_domain::{
        Gp, ItemId, Probability, Rate, Recommendation, RecommendationAction,
        RecommendationExplanation, RecommendationId,
    };
    use uuid::Uuid;

    use super::recommendation_matches_risk_profile;

    #[test]
    fn recommendation_risk_profile_filters_buy_requiring_more_confidence() {
        let mut recommendation = fixture_recommendation(RecommendationAction::Buy);
        recommendation.recommendation_confidence = Probability::new(0.40).unwrap();

        let mut config = grand_edge_recommender::RecommendationConfig::default();
        config.min_confidence = 0.55;

        assert!(!recommendation_matches_risk_profile(
            &recommendation,
            &config
        ));
    }

    #[test]
    fn recommendation_risk_profile_keeps_cashout_even_when_roi_is_low() {
        let recommendation = fixture_recommendation(RecommendationAction::Cashout);
        let mut config = grand_edge_recommender::RecommendationConfig::default();
        config.min_expected_roi = 0.25;

        assert!(recommendation_matches_risk_profile(
            &recommendation,
            &config
        ));
    }

    fn fixture_recommendation(action: RecommendationAction) -> Recommendation {
        Recommendation {
            recommendation_id: RecommendationId(Uuid::new_v4()),
            user_id: Some(grand_edge_domain::UserId(Uuid::new_v4())),
            item_id: ItemId(4151),
            as_of: Utc::now(),
            action,
            score: Rate::new(0.8).unwrap(),
            prediction_confidence: Some(Probability::new(0.8).unwrap()),
            execution_confidence: Some(Probability::new(0.7).unwrap()),
            recommendation_confidence: Probability::new(0.7).unwrap(),
            expected_net_gp: Some(Gp(1000)),
            expected_roi: Some(Rate::new(0.03).unwrap()),
            risk_label: Some("low".to_string()),
            reasons: vec!["fixture".to_string()],
            explanation: RecommendationExplanation {
                feature_set_version: "features_v1".to_string(),
                market_rules_version: "rules_v1".to_string(),
                graph_version: None,
                graph_context: None,
                strategy_votes: Vec::new(),
                score_components: Vec::new(),
                accuracy_snapshot: None,
                structured_explanation:
                    grand_edge_domain::StructuredRecommendationExplanation::default(),
            },
        }
    }
}
