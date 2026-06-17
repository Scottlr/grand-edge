use axum::{Json, extract::State};
use axum_extra::extract::CookieJar;
use grand_edge_domain::{
    AuthenticatedUser, EmailAddress, ExecutionMode, LoginRequest, RegisterRequest,
    UpdateRiskProfile, UserRiskProfile,
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    auth::{
        build_session_cookie, clear_session_cookie, require_authenticated_user, require_user_id,
        session_id_from_jar,
    },
    errors::ApiError,
    state::AppState,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequestDto {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequestDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatedUserDto {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionModeDto {
    ConservativeInstant,
    PassiveEstimated,
    HaircutPassive,
    WorstCase,
    UserPositionReplay,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RiskProfileDto {
    pub user_id: uuid::Uuid,
    pub max_gp_per_item: i64,
    pub max_portfolio_drawdown: f64,
    pub min_expected_roi: f64,
    pub min_confidence: f64,
    pub participation_rate: f64,
    pub preferred_execution_mode: ExecutionModeDto,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRiskProfileRequest {
    pub max_gp_per_item: i64,
    pub max_portfolio_drawdown: f64,
    pub min_expected_roi: f64,
    pub min_confidence: f64,
    pub participation_rate: f64,
    pub preferred_execution_mode: ExecutionModeDto,
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequestDto,
    responses((status = 200, body = AuthenticatedUserDto))
)]
pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<RegisterRequestDto>,
) -> Result<(CookieJar, Json<AuthenticatedUserDto>), ApiError> {
    let user = state
        .services
        .register(RegisterRequest {
            email: EmailAddress::new(request.email)?,
            password: SecretString::new(request.password.clone().into()),
            display_name: request.display_name,
        })
        .await?;
    let (_, session) = state
        .services
        .login(LoginRequest {
            email: user.email.clone(),
            password: SecretString::new(request.password.into()),
        })
        .await?;
    let jar = jar.add(build_session_cookie(session.session_id));
    Ok((jar, Json(AuthenticatedUserDto::from(user))))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequestDto,
    responses((status = 200, body = AuthenticatedUserDto))
)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequestDto>,
) -> Result<(CookieJar, Json<AuthenticatedUserDto>), ApiError> {
    let (user, session) = state
        .services
        .login(LoginRequest {
            email: EmailAddress::new(request.email)?,
            password: SecretString::new(request.password.into()),
        })
        .await?;
    let jar = jar.add(build_session_cookie(session.session_id));
    Ok((jar, Json(AuthenticatedUserDto::from(user))))
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses((status = 200))
)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<serde_json::Value>), ApiError> {
    if let Some(session_id) = session_id_from_jar(&jar) {
        state.services.logout(session_id).await?;
    }
    Ok((
        jar.remove(clear_session_cookie()),
        Json(serde_json::json!({ "status": "ok" })),
    ))
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses((status = 200, body = AuthenticatedUserDto), (status = 401))
)]
pub async fn me(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<AuthenticatedUserDto>, ApiError> {
    let user = require_authenticated_user(&state, &jar).await?;
    Ok(Json(AuthenticatedUserDto::from(user)))
}

#[utoipa::path(
    get,
    path = "/api/users/me/risk-profile",
    responses((status = 200, body = RiskProfileDto), (status = 401))
)]
pub async fn get_risk_profile(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<RiskProfileDto>, ApiError> {
    let user_id = require_user_id(&state, &jar).await?;
    let profile = state.services.get_risk_profile(user_id).await?;
    Ok(Json(RiskProfileDto::from(profile)))
}

#[utoipa::path(
    patch,
    path = "/api/users/me/risk-profile",
    request_body = UpdateRiskProfileRequest,
    responses((status = 200, body = RiskProfileDto), (status = 401))
)]
pub async fn update_risk_profile(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<UpdateRiskProfileRequest>,
) -> Result<Json<RiskProfileDto>, ApiError> {
    let user_id = require_user_id(&state, &jar).await?;
    let profile = state
        .services
        .update_risk_profile(
            user_id,
            UpdateRiskProfile {
                max_gp_per_item: request.max_gp_per_item,
                max_portfolio_drawdown: request.max_portfolio_drawdown,
                min_expected_roi: request.min_expected_roi,
                min_confidence: request.min_confidence,
                participation_rate: request.participation_rate,
                preferred_execution_mode: ExecutionMode::from(request.preferred_execution_mode),
            },
        )
        .await?;
    Ok(Json(RiskProfileDto::from(profile)))
}

impl From<AuthenticatedUser> for AuthenticatedUserDto {
    fn from(value: AuthenticatedUser) -> Self {
        Self {
            user_id: value.user_id.0,
            email: value.email.as_str().to_string(),
            display_name: value.display_name,
        }
    }
}

impl From<UserRiskProfile> for RiskProfileDto {
    fn from(value: UserRiskProfile) -> Self {
        Self {
            user_id: value.user_id.0,
            max_gp_per_item: value.max_gp_per_item.0,
            max_portfolio_drawdown: value.max_portfolio_drawdown.get(),
            min_expected_roi: value.min_expected_roi.get(),
            min_confidence: value.min_confidence.get(),
            participation_rate: value.participation_rate.get(),
            preferred_execution_mode: ExecutionModeDto::from(value.preferred_execution_mode),
            updated_at: value.updated_at,
        }
    }
}

impl From<ExecutionModeDto> for ExecutionMode {
    fn from(value: ExecutionModeDto) -> Self {
        match value {
            ExecutionModeDto::ConservativeInstant => ExecutionMode::ConservativeInstant,
            ExecutionModeDto::PassiveEstimated => ExecutionMode::PassiveEstimated,
            ExecutionModeDto::HaircutPassive => ExecutionMode::HaircutPassive,
            ExecutionModeDto::WorstCase => ExecutionMode::WorstCase,
            ExecutionModeDto::UserPositionReplay => ExecutionMode::UserPositionReplay,
        }
    }
}

impl From<ExecutionMode> for ExecutionModeDto {
    fn from(value: ExecutionMode) -> Self {
        match value {
            ExecutionMode::ConservativeInstant => ExecutionModeDto::ConservativeInstant,
            ExecutionMode::PassiveEstimated => ExecutionModeDto::PassiveEstimated,
            ExecutionMode::HaircutPassive => ExecutionModeDto::HaircutPassive,
            ExecutionMode::WorstCase => ExecutionModeDto::WorstCase,
            ExecutionMode::UserPositionReplay => ExecutionModeDto::UserPositionReplay,
        }
    }
}
