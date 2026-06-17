use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use grand_edge_domain::{AuthenticatedUser, SessionId, UserId};

use crate::{errors::ApiError, state::AppState};

pub const SESSION_COOKIE_NAME: &str = "grand_edge_session";

pub async fn require_authenticated_user(
    state: &AppState,
    jar: &CookieJar,
) -> Result<AuthenticatedUser, ApiError> {
    let session_id = session_id_from_jar(jar)
        .ok_or_else(|| ApiError::Unauthorized("authentication required".to_string()))?;
    state
        .services
        .current_user(session_id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("authentication required".to_string()))
}

pub async fn request_user_id(
    state: &AppState,
    jar: &CookieJar,
) -> Result<Option<UserId>, ApiError> {
    if let Some(session_id) = session_id_from_jar(jar) {
        return Ok(state
            .services
            .current_user(session_id)
            .await?
            .map(|user| user.user_id));
    }
    state.services.local_default_user().await
}

pub async fn require_user_id(state: &AppState, jar: &CookieJar) -> Result<UserId, ApiError> {
    request_user_id(state, jar)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("authentication required".to_string()))
}

pub fn session_id_from_jar(jar: &CookieJar) -> Option<SessionId> {
    let raw = jar.get(SESSION_COOKIE_NAME)?.value().to_string();
    uuid::Uuid::parse_str(&raw).ok().map(SessionId)
}

pub fn build_session_cookie(session_id: SessionId) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, session_id.0.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build()
}

pub fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build()
}
