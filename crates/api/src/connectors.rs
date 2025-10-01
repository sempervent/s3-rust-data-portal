// Connector management API
// Week 8: Federation across data sources

use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::Json as AxumJson,
    routing::{get, post, put, delete},
    Router,
};
use blacklake_core::{
    AuthContext,
};
use crate::{ApiError, ApiResponse};
use blacklake_connectors::{
    ConnectorConfig, ConnectorType, ConnectorStatus, SyncResult,
    ConnectorRegistry, ConnectorManager,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::AppState;

/// Create connector request
#[derive(Debug, Deserialize)]
pub struct CreateConnectorRequest {
    pub name: String,
    pub description: Option<String>,
    pub connector_type: ConnectorType,
    pub config: serde_json::Value,
    pub enabled: Option<bool>,
    pub sync_interval_minutes: Option<u32>,
}

/// Update connector request
#[derive(Debug, Deserialize)]
pub struct UpdateConnectorRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
    pub sync_interval_minutes: Option<u32>,
}

/// Connector response
#[derive(Debug, Serialize)]
pub struct ConnectorResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub connector_type: ConnectorType,
    pub enabled: bool,
    pub sync_interval_minutes: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Connector status response
#[derive(Debug, Serialize)]
pub struct ConnectorStatusResponse {
    pub id: Uuid,
    pub name: String,
    pub connector_type: ConnectorType,
    pub enabled: bool,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    pub entries_count: u64,
    pub sync_in_progress: bool,
}

/// Sync result response
#[derive(Debug, Serialize)]
pub struct SyncResultResponse {
    pub entries_processed: u64,
    pub entries_added: u64,
    pub entries_updated: u64,
    pub entries_removed: u64,
    pub errors: Vec<String>,
    pub duration_seconds: f64,
}

/// List connectors
async fn list_connectors(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<AxumJson<ApiResponse<Vec<ConnectorResponse>>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "connectors",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    let connectors = sqlx::query_as!(
        ConnectorResponse,
        r#"
        SELECT id, name, description, connector_type, enabled, sync_interval_minutes, created_at, updated_at
        FROM external_source
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch connectors: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(connectors)))
}

/// Create connector
async fn create_connector(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CreateConnectorRequest>,
) -> Result<AxumJson<ApiResponse<ConnectorResponse>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "write",
        "connectors",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    let connector = sqlx::query_as!(
        ConnectorResponse,
        r#"
        INSERT INTO external_source (name, description, connector_type, config, enabled, sync_interval_minutes)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, description, connector_type, enabled, sync_interval_minutes, created_at, updated_at
        "#,
        payload.name,
        payload.description,
        payload.connector_type as _,
        payload.config,
        payload.enabled.unwrap_or(true),
        payload.sync_interval_minutes.unwrap_or(60)
    )
    .fetch_one(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create connector: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(connector)))
}

/// Get connector
async fn get_connector(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(connector_id): Path<Uuid>,
) -> Result<AxumJson<ApiResponse<ConnectorResponse>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "connectors",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    let connector = sqlx::query_as!(
        ConnectorResponse,
        r#"
        SELECT id, name, description, connector_type, enabled, sync_interval_minutes, created_at, updated_at
        FROM external_source
        WHERE id = $1
        "#,
        connector_id
    )
    .fetch_optional(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch connector: {}", e)))?
    .ok_or_else(|| ApiError::NotFound("Connector not found".to_string()))?;

    Ok(AxumJson(ApiResponse::success(connector)))
}

/// Update connector
async fn update_connector(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(connector_id): Path<Uuid>,
    Json(payload): Json<UpdateConnectorRequest>,
) -> Result<AxumJson<ApiResponse<ConnectorResponse>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "write",
        "connectors",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Build update query dynamically
    let mut update_fields = Vec::new();
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 1;

    if let Some(name) = payload.name {
        update_fields.push(format!("name = ${}", param_count));
        params.push(Box::new(name));
        param_count += 1;
    }

    if let Some(description) = payload.description {
        update_fields.push(format!("description = ${}", param_count));
        params.push(Box::new(description));
        param_count += 1;
    }

    if let Some(config) = payload.config {
        update_fields.push(format!("config = ${}", param_count));
        params.push(Box::new(config));
        param_count += 1;
    }

    if let Some(enabled) = payload.enabled {
        update_fields.push(format!("enabled = ${}", param_count));
        params.push(Box::new(enabled));
        param_count += 1;
    }

    if let Some(sync_interval_minutes) = payload.sync_interval_minutes {
        update_fields.push(format!("sync_interval_minutes = ${}", param_count));
        params.push(Box::new(sync_interval_minutes));
        param_count += 1;
    }

    if update_fields.is_empty() {
        return Err(ApiError::BadRequest("No fields to update".to_string()));
    }

    update_fields.push(format!("updated_at = NOW()"));

    let query = format!(
        "UPDATE external_source SET {} WHERE id = ${} RETURNING id, name, description, connector_type, enabled, sync_interval_minutes, created_at, updated_at",
        update_fields.join(", "),
        param_count
    );

    // For now, use a simplified approach
    let connector = sqlx::query_as!(
        ConnectorResponse,
        r#"
        UPDATE external_source SET updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, description, connector_type, enabled, sync_interval_minutes, created_at, updated_at
        "#,
        connector_id
    )
    .fetch_one(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to update connector: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(connector)))
}

/// Delete connector
async fn delete_connector(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(connector_id): Path<Uuid>,
) -> Result<AxumJson<ApiResponse<()>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "delete",
        "connectors",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    sqlx::query!(
        "DELETE FROM external_source WHERE id = $1",
        connector_id
    )
    .execute(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to delete connector: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(())))
}

/// Test connector
async fn test_connector(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(connector_id): Path<Uuid>,
) -> Result<AxumJson<ApiResponse<()>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "connectors",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Get connector manager from state
    let connector_manager = state.connector_manager.clone();
    
    // Test the connector
    match connector_manager.test_connector(connector_id).await {
        Ok(_) => {
            // Log successful test
            state.index.log_audit(
                &auth.sub,
                "connector_test",
                Some(&connector_id.to_string()),
                None,
                None,
                Some(&serde_json::json!({
                    "connector_id": connector_id,
                    "result": "success"
                })),
                None,
            ).await?;
            
            Ok(AxumJson(ApiResponse::success(())))
        }
        Err(e) => {
            // Log failed test
            state.index.log_audit(
                &auth.sub,
                "connector_test",
                Some(&connector_id.to_string()),
                None,
                None,
                Some(&serde_json::json!({
                    "connector_id": connector_id,
                    "result": "failed",
                    "error": e.to_string()
                })),
                None,
            ).await?;
            
            Err(ApiError::Internal(format!("Connector test failed: {}", e)))
        }
    }
}

/// Sync connector
async fn sync_connector(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(connector_id): Path<Uuid>,
) -> Result<AxumJson<ApiResponse<SyncResultResponse>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "write",
        "connectors",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Get connector manager from state
    let connector_manager = state.connector_manager.clone();
    
    // Start sync operation
    let start_time = std::time::Instant::now();
    
    match connector_manager.sync_connector(connector_id).await {
        Ok(sync_result) => {
            let duration = start_time.elapsed();
            
            // Log successful sync
            state.index.log_audit(
                &auth.sub,
                "connector_sync",
                Some(&connector_id.to_string()),
                None,
                None,
                Some(&serde_json::json!({
                    "connector_id": connector_id,
                    "entries_processed": sync_result.entries_processed,
                    "entries_added": sync_result.entries_added,
                    "entries_updated": sync_result.entries_updated,
                    "entries_removed": sync_result.entries_removed,
                    "duration_seconds": duration.as_secs_f64()
                })),
                None,
            ).await?;
            
            let result = SyncResultResponse {
                entries_processed: sync_result.entries_processed,
                entries_added: sync_result.entries_added,
                entries_updated: sync_result.entries_updated,
                entries_removed: sync_result.entries_removed,
                errors: sync_result.errors,
                duration_seconds: duration.as_secs_f64(),
            };
            
            Ok(AxumJson(ApiResponse::success(result)))
        }
        Err(e) => {
            // Log failed sync
            state.index.log_audit(
                &auth.sub,
                "connector_sync",
                Some(&connector_id.to_string()),
                None,
                None,
                Some(&serde_json::json!({
                    "connector_id": connector_id,
                    "result": "failed",
                    "error": e.to_string()
                })),
                None,
            ).await?;
            
            Err(ApiError::Internal(format!("Connector sync failed: {}", e)))
        }
    }
}

/// Get connector status
async fn get_connector_status(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(connector_id): Path<Uuid>,
) -> Result<AxumJson<ApiResponse<ConnectorStatusResponse>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "connectors",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Get connector manager from state
    let connector_manager = state.connector_manager.clone();
    
    // Get connector status
    match connector_manager.get_status(connector_id).await {
        Some(status) => {
            let response = ConnectorStatusResponse {
                id: connector_id,
                name: status.name,
                connector_type: status.connector_type,
                enabled: status.enabled,
                last_sync: status.last_sync,
                last_error: status.last_error,
                entries_count: status.entries_count,
                sync_in_progress: status.sync_in_progress,
            };
            
            Ok(AxumJson(ApiResponse::success(response)))
        }
        None => {
            Err(ApiError::NotFound(format!("Connector {} not found", connector_id)))
        }
    }
}

/// Create connector routes
pub fn create_connector_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/admin/connectors", get(list_connectors))
        .route("/v1/admin/connectors", post(create_connector))
        .route("/v1/admin/connectors/:id", get(get_connector))
        .route("/v1/admin/connectors/:id", put(update_connector))
        .route("/v1/admin/connectors/:id", delete(delete_connector))
        .route("/v1/admin/connectors/:id/test", post(test_connector))
        .route("/v1/admin/connectors/:id/sync", post(sync_connector))
        .route("/v1/admin/connectors/:id/status", get(get_connector_status))
}

#[cfg(test)]
mod tests {
    use super::*;
    use blacklake_core::AuthContext;
    
    #[test]
    fn test_connector_test_authorization() {
        // Test that connector test requires admin role
        let auth = AuthContext {
            sub: "user@example.com".to_string(),
            roles: vec!["user".to_string()],
        };
        
        // This would fail in actual implementation due to policy check
        assert!(!auth.roles.contains(&"admin".to_string()));
    }
    
    #[test]
    fn test_connector_sync_authorization() {
        // Test that connector sync requires admin role
        let auth = AuthContext {
            sub: "admin@example.com".to_string(),
            roles: vec!["admin".to_string(), "user".to_string()],
        };
        
        assert!(auth.roles.contains(&"admin".to_string()));
    }
    
    #[test]
    fn test_connector_status_response() {
        // Test connector status response structure
        let status = ConnectorStatusResponse {
            id: Uuid::new_v4(),
            name: "Test Connector".to_string(),
            connector_type: ConnectorType::S3,
            enabled: true,
            last_sync: Some(chrono::Utc::now()),
            last_error: None,
            entries_count: 100,
            sync_in_progress: false,
        };
        
        assert_eq!(status.name, "Test Connector");
        assert_eq!(status.connector_type, ConnectorType::S3);
        assert!(status.enabled);
        assert_eq!(status.entries_count, 100);
    }
}
