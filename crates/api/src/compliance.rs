use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put, delete},
    Router,
};
use blacklake_core::{
    ApiError, ApiResponse, AuthContext, compliance::{
        ComplianceService, RetentionPolicy, LegalHold, AuditLog, ComplianceExport,
        ExportType, ExportStatus, LegalHoldStatus
    },
};
use sqlx::{PgPool, query, query_as};
use uuid::Uuid;
use tracing::{info, error};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateRetentionPolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub retention_days: i32,
    pub legal_hold_override: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRetentionPolicyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub retention_days: Option<i32>,
    pub legal_hold_override: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLegalHoldRequest {
    pub entry_id: Uuid,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateComplianceExportRequest {
    pub export_type: ExportType,
    pub filters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get all retention policies
async fn list_retention_policies(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
) -> Result<Json<ApiResponse<Vec<RetentionPolicy>>>, ApiError> {
    // TODO: Add admin role check
    let policies = query_as!(RetentionPolicy, "SELECT id, name, description, retention_days, legal_hold_override, created_at, updated_at FROM retention_policy ORDER BY created_at DESC")
        .fetch_all(&state.index.get_pool())
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch retention policies: {}", e)))?;
    
    Ok(Json(ApiResponse::success(policies)))
}

/// Create a new retention policy
async fn create_retention_policy(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
    Json(payload): Json<CreateRetentionPolicyRequest>,
) -> Result<Json<ApiResponse<RetentionPolicy>>, ApiError> {
    // TODO: Add admin role check
    let compliance_service = ComplianceService::new(state.index.get_pool());
    
    let policy = compliance_service.create_retention_policy(
        &payload.name,
        payload.description.as_deref(),
        payload.retention_days,
        payload.legal_hold_override,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to create retention policy: {}", e)))?;

    // Log audit event
    compliance_service.log_audit_event(
        auth.user_id,
        "create_retention_policy",
        "retention_policy",
        policy.id,
        serde_json::json!({
            "name": policy.name,
            "retention_days": policy.retention_days,
            "legal_hold_override": policy.legal_hold_override
        }),
        None,
        None,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to log audit event: {}", e)))?;

    info!("Created retention policy: {}", policy.name);
    Ok(Json(ApiResponse::success(policy)))
}

/// Get a specific retention policy
async fn get_retention_policy(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<RetentionPolicy>>, ApiError> {
    // TODO: Add admin role check
    let policy = query_as!(RetentionPolicy, "SELECT id, name, description, retention_days, legal_hold_override, created_at, updated_at FROM retention_policy WHERE id = $1", id)
        .fetch_optional(&state.index.get_pool())
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch retention policy: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Retention policy not found".to_string()))?;
    
    Ok(Json(ApiResponse::success(policy)))
}

/// Update an existing retention policy
async fn update_retention_policy(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRetentionPolicyRequest>,
) -> Result<Json<ApiResponse<RetentionPolicy>>, ApiError> {
    // TODO: Add admin role check
    let policy = query_as!(
        RetentionPolicy,
        "UPDATE retention_policy SET name = COALESCE($1, name), description = COALESCE($2, description), retention_days = COALESCE($3, retention_days), legal_hold_override = COALESCE($4, legal_hold_override), updated_at = NOW() WHERE id = $5 RETURNING id, name, description, retention_days, legal_hold_override, created_at, updated_at",
        payload.name,
        payload.description,
        payload.retention_days,
        payload.legal_hold_override,
        id
    )
    .fetch_one(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to update retention policy: {}", e)))?
    .ok_or_else(|| ApiError::NotFound("Retention policy not found".to_string()))?;

    // Log audit event
    let compliance_service = ComplianceService::new(state.index.get_pool());
    compliance_service.log_audit_event(
        auth.user_id,
        "update_retention_policy",
        "retention_policy",
        policy.id,
        serde_json::json!({
            "name": policy.name,
            "retention_days": policy.retention_days,
            "legal_hold_override": policy.legal_hold_override
        }),
        None,
        None,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to log audit event: {}", e)))?;

    info!("Updated retention policy: {}", policy.name);
    Ok(Json(ApiResponse::success(policy)))
}

/// Delete a retention policy
async fn delete_retention_policy(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // TODO: Add admin role check
    let result = query!("DELETE FROM retention_policy WHERE id = $1", id)
        .execute(&state.index.get_pool())
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to delete retention policy: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Retention policy not found".to_string()));
    }

    // Log audit event
    let compliance_service = ComplianceService::new(state.index.get_pool());
    compliance_service.log_audit_event(
        auth.user_id,
        "delete_retention_policy",
        "retention_policy",
        id,
        serde_json::json!({}),
        None,
        None,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to log audit event: {}", e)))?;

    info!("Deleted retention policy: {}", id);
    Ok(Json(ApiResponse::success("Retention policy deleted successfully".to_string())))
}

/// Get all legal holds
async fn list_legal_holds(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
) -> Result<Json<ApiResponse<Vec<LegalHold>>>, ApiError> {
    // TODO: Add admin role check
    let legal_holds = query_as!(LegalHold, "SELECT id, entry_id, reason, created_by, created_at, expires_at, status FROM legal_hold ORDER BY created_at DESC")
        .fetch_all(&state.index.get_pool())
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch legal holds: {}", e)))?;
    
    Ok(Json(ApiResponse::success(legal_holds)))
}

/// Create a new legal hold
async fn create_legal_hold(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
    Json(payload): Json<CreateLegalHoldRequest>,
) -> Result<Json<ApiResponse<LegalHold>>, ApiError> {
    // TODO: Add admin role check
    let compliance_service = ComplianceService::new(state.index.get_pool());
    
    let legal_hold = compliance_service.create_legal_hold(
        payload.entry_id,
        &payload.reason,
        auth.user_id,
        payload.expires_at,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to create legal hold: {}", e)))?;

    // Log audit event
    compliance_service.log_audit_event(
        auth.user_id,
        "create_legal_hold",
        "legal_hold",
        legal_hold.id,
        serde_json::json!({
            "entry_id": legal_hold.entry_id,
            "reason": legal_hold.reason,
            "expires_at": legal_hold.expires_at
        }),
        None,
        None,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to log audit event: {}", e)))?;

    info!("Created legal hold for entry {}: {}", payload.entry_id, payload.reason);
    Ok(Json(ApiResponse::success(legal_hold)))
}

/// Release a legal hold
async fn release_legal_hold(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // TODO: Add admin role check
    let compliance_service = ComplianceService::new(state.index.get_pool());
    
    compliance_service.release_legal_hold(id, auth.user_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to release legal hold: {}", e)))?;

    // Log audit event
    compliance_service.log_audit_event(
        auth.user_id,
        "release_legal_hold",
        "legal_hold",
        id,
        serde_json::json!({}),
        None,
        None,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to log audit event: {}", e)))?;

    info!("Released legal hold: {}", id);
    Ok(Json(ApiResponse::success("Legal hold released successfully".to_string())))
}

/// Get audit logs
async fn get_audit_logs(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
    Query(params): Query<AuditLogQuery>,
) -> Result<Json<ApiResponse<Vec<AuditLog>>>, ApiError> {
    // TODO: Add admin role check
    let compliance_service = ComplianceService::new(state.index.get_pool());
    
    let logs = compliance_service.get_audit_logs(
        params.user_id,
        params.action.as_deref(),
        params.resource_type.as_deref(),
        params.start_date,
        params.end_date,
        params.limit,
        params.offset,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch audit logs: {}", e)))?;
    
    Ok(Json(ApiResponse::success(logs)))
}

/// Create a compliance export
async fn create_compliance_export(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
    Json(payload): Json<CreateComplianceExportRequest>,
) -> Result<Json<ApiResponse<ComplianceExport>>, ApiError> {
    // TODO: Add admin role check
    let compliance_service = ComplianceService::new(state.index.get_pool());
    
    let export = compliance_service.create_compliance_export(
        payload.export_type,
        payload.filters,
        auth.user_id,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to create compliance export: {}", e)))?;

    // Log audit event
    compliance_service.log_audit_event(
        auth.user_id,
        "create_compliance_export",
        "compliance_export",
        export.id,
        serde_json::json!({
            "export_type": export.export_type,
            "filters": export.filters
        }),
        None,
        None,
    ).await
    .map_err(|e| ApiError::Internal(format!("Failed to log audit event: {}", e)))?;

    info!("Created compliance export: {:?}", export.export_type);
    Ok(Json(ApiResponse::success(export)))
}

/// Get compliance export status
async fn get_compliance_export(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ComplianceExport>>, ApiError> {
    // TODO: Add admin role check
    let export = query_as!(ComplianceExport, "SELECT id, export_type, filters, status, file_path, created_by, created_at, completed_at FROM compliance_export WHERE id = $1", id)
        .fetch_optional(&state.index.get_pool())
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch compliance export: {}", e)))?
        .ok_or_else(|| ApiError::NotFound("Compliance export not found".to_string()))?;
    
    Ok(Json(ApiResponse::success(export)))
}

/// Get retention status summary
async fn get_retention_status_summary(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    // TODO: Add admin role check
    let compliance_service = ComplianceService::new(state.index.get_pool());
    
    let summary = compliance_service.get_retention_status_summary()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get retention status summary: {}", e)))?;
    
    Ok(Json(ApiResponse::success(summary)))
}

/// Get entries eligible for deletion
async fn get_deletable_entries(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
) -> Result<Json<ApiResponse<Vec<Uuid>>>, ApiError> {
    // TODO: Add admin role check
    let compliance_service = ComplianceService::new(state.index.get_pool());
    
    let entries = compliance_service.get_deletable_entries()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get deletable entries: {}", e)))?;
    
    Ok(Json(ApiResponse::success(entries)))
}

/// Get entries under legal hold
async fn get_legal_hold_entries(
    State(state): State<AppState>,
    auth: AuthContext, // Admin only
) -> Result<Json<ApiResponse<Vec<Uuid>>>, ApiError> {
    // TODO: Add admin role check
    let compliance_service = ComplianceService::new(state.index.get_pool());
    
    let entries = compliance_service.get_legal_hold_entries()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get legal hold entries: {}", e)))?;
    
    Ok(Json(ApiResponse::success(entries)))
}

pub fn create_compliance_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/admin/retention-policies", get(list_retention_policies).post(create_retention_policy))
        .route("/v1/admin/retention-policies/:id", get(get_retention_policy).put(update_retention_policy).delete(delete_retention_policy))
        .route("/v1/admin/legal-holds", get(list_legal_holds).post(create_legal_hold))
        .route("/v1/admin/legal-holds/:id/release", post(release_legal_hold))
        .route("/v1/admin/audit-logs", get(get_audit_logs))
        .route("/v1/admin/compliance-exports", post(create_compliance_export))
        .route("/v1/admin/compliance-exports/:id", get(get_compliance_export))
        .route("/v1/admin/retention-status", get(get_retention_status_summary))
        .route("/v1/admin/deletable-entries", get(get_deletable_entries))
        .route("/v1/admin/legal-hold-entries", get(get_legal_hold_entries))
}
