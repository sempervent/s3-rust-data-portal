// BlackLake Session Management API
// Week 6: Server-side sessions with Redis backend

use axum::{
    extract::{State, Request},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use blacklake_core::{
    AuthContext,
};
use blacklake_core::sessions::{AuthSession, CSRFToken, SessionError};
use crate::{ApiError, ApiResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;
use tower_sessions_redis_store::RedisStore;
use crate::AppState;
use crate::health::{
    SESSION_CREATIONS_TOTAL, SESSION_DESTROYALS_TOTAL, ACTIVE_SESSIONS,
    CSRF_TOKEN_REQUESTS_TOTAL, CSRF_TOKEN_VALIDATIONS_TOTAL, CSRF_TOKEN_VALIDATION_FAILURES_TOTAL,
};

/// Session login request
#[derive(Debug, Deserialize)]
pub struct SessionLoginRequest {
    pub oidc_token: String,
}

/// Session login response
#[derive(Debug, Serialize)]
pub struct SessionLoginResponse {
    pub success: bool,
    pub message: String,
}

/// CSRF token response
#[derive(Debug, Serialize)]
pub struct CSRFTokenResponse {
    pub csrf_token: String,
}

/// Session logout response
#[derive(Debug, Serialize)]
pub struct SessionLogoutResponse {
    pub success: bool,
    pub message: String,
}

/// Create session after OIDC login
async fn session_login(
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<SessionLoginRequest>,
) -> Result<Json<ApiResponse<SessionLoginResponse>>, ApiError> {
    // TODO: Validate OIDC token and extract user info
    // For now, we'll create a mock session
    
    let auth_session = AuthSession::new(
        "user-123".to_string(),
        "user@example.com".to_string(),
        vec!["user".to_string()],
        Some(serde_json::json!({
            "iss": "http://keycloak:8080/realms/master",
            "aud": "blacklake",
            "exp": 1640995200
        })),
    );

    // Store session in Redis
    session.insert("auth_session", &auth_session)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to store session: {}", e)))?;

    // Update metrics
    SESSION_CREATIONS_TOTAL.inc();
    ACTIVE_SESSIONS.inc();

    // Log audit
    state.index.log_audit(
        &auth_session.sub,
        "session_login",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "email": auth_session.email,
            "roles": auth_session.roles
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(SessionLoginResponse {
        success: true,
        message: "Session created successfully".to_string(),
    })))
}

/// Get CSRF token
async fn get_csrf_token(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<ApiResponse<CSRFTokenResponse>>, ApiError> {
    // Get existing session or create new one
    let auth_session: Option<AuthSession> = session.get("auth_session")
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get session: {}", e)))?;

    let auth_session = auth_session.ok_or_else(|| {
        ApiError::Auth("No active session found".to_string())
    })?;

    // Update metrics
    CSRF_TOKEN_REQUESTS_TOTAL.inc();

    Ok(Json(ApiResponse::success(CSRFTokenResponse {
        csrf_token: auth_session.csrf_token.as_str().to_string(),
    })))
}

/// Logout and revoke session
async fn session_logout(
    State(state): State<AppState>,
    session: Session,
    auth: AuthContext,
) -> Result<Json<ApiResponse<SessionLogoutResponse>>, ApiError> {
    // Log audit before clearing session
    state.index.log_audit(
        &auth.sub,
        "session_logout",
        None,
        None,
        None,
        None,
        None,
    ).await?;

    // Clear session from Redis
    session.delete("auth_session")
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to clear session: {}", e)))?;

    // Update metrics
    SESSION_DESTROYALS_TOTAL.inc();
    ACTIVE_SESSIONS.dec();

    Ok(Json(ApiResponse::success(SessionLogoutResponse {
        success: true,
        message: "Session revoked successfully".to_string(),
    })))
}

/// Get current session info
async fn get_session_info(
    State(state): State<AppState>,
    session: Session,
    auth: AuthContext,
) -> Result<Json<ApiResponse<AuthSession>>, ApiError> {
    let auth_session: Option<AuthSession> = session.get("auth_session")
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get session: {}", e)))?;

    let auth_session = auth_session.ok_or_else(|| {
        ApiError::Auth("No active session found".to_string())
    })?;

    Ok(Json(ApiResponse::success(auth_session)))
}

/// Create session API routes
pub fn create_session_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/session/login", post(session_login))
        .route("/v1/csrf", get(get_csrf_token))
        .route("/v1/session/logout", post(session_logout))
        .route("/v1/session/info", get(get_session_info))
}

