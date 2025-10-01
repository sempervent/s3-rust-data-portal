// Week 4: Governance & Safety Rails API handlers
// Branch protection, quotas, retention, webhooks, and export jobs

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use blacklake_core::{
    // Core types
    Repository, Uuid,
};
use blacklake_core::governance::{ProtectedRef, RepoQuota, RepoUsage, RepoRetention, Webhook, WebhookDelivery, 
    ExportJob, ExportManifest, ExportJobStatus, CheckResult, CheckStatus, QuotaStatus,
    WebhookEvent, RetentionPolicy, PolicyEvaluation};
use crate::{ApiError, ApiResponse};
use blacklake_index::IndexClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;

use crate::{
    auth::{extract_auth, AuthContext},
    error::{ApiError, ApiResult},
    AppState,
};

/// Request to set branch protection rules
#[derive(Debug, Deserialize)]
pub struct SetProtectionRequest {
    pub require_admin: bool,
    pub allow_fast_forward: bool,
    pub allow_delete: bool,
    pub required_checks: Vec<String>,
    pub required_reviewers: u32,
    pub require_schema_pass: bool,
}

/// Request to set repository quota
#[derive(Debug, Deserialize)]
pub struct SetQuotaRequest {
    pub bytes_soft: u64,
    pub bytes_hard: u64,
}

/// Request to set retention policy
#[derive(Debug, Deserialize)]
pub struct SetRetentionRequest {
    pub tombstone_days: u32,
    pub hard_delete_days: u32,
    pub legal_hold: bool,
}

/// Request to create a webhook
#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub secret: String,
    pub events: Vec<WebhookEvent>,
}

/// Request to create an export job
#[derive(Debug, Deserialize)]
pub struct CreateExportRequest {
    pub manifest: ExportManifest,
}

/// Request to submit a check result
#[derive(Debug, Deserialize)]
pub struct SubmitCheckRequest {
    pub check_name: String,
    pub status: CheckStatus,
    pub details_url: Option<String>,
    pub output: Option<String>,
}

/// Response for quota status
#[derive(Debug, Serialize)]
pub struct QuotaStatusResponse {
    pub quota: QuotaStatus,
    pub repo_name: String,
}

/// Response for webhook delivery history
#[derive(Debug, Serialize)]
pub struct WebhookDeliveryResponse {
    pub deliveries: Vec<WebhookDelivery>,
    pub total: u32,
}

/// Response for export job status
#[derive(Debug, Serialize)]
pub struct ExportJobResponse {
    pub job: ExportJob,
    pub repo_name: String,
}

/// Create governance routes
pub fn create_governance_routes() -> Router<AppState> {
    Router::new()
        // Branch protection
        .route("/v1/repos/:repo/protection/:ref", get(get_protection).put(set_protection))
        // Quotas
        .route("/v1/repos/:repo/quota", get(get_quota).put(set_quota))
        .route("/v1/repos/:repo/usage", get(get_usage))
        // Retention
        .route("/v1/repos/:repo/retention", get(get_retention).put(set_retention))
        // Webhooks
        .route("/v1/repos/:repo/webhooks", get(get_webhooks).post(create_webhook))
        .route("/v1/repos/:repo/webhooks/:webhook_id", delete(delete_webhook))
        .route("/v1/repos/:repo/webhooks/:webhook_id/deliveries", get(get_webhook_deliveries))
        // Export jobs
        .route("/v1/repos/:repo/export", post(create_export))
        .route("/v1/export/:job_id", get(get_export_job))
        // Check results
        .route("/v1/repos/:repo/checks/:ref", post(submit_check))
        .route("/v1/repos/:repo/checks/:ref/:commit_id", get(get_checks))
        .layer(CorsLayer::permissive())
}

/// Get branch protection rules for a repository reference
async fn get_protection(
    State(state): State<AppState>,
    Path((repo_name, ref_name)): Path<(String, String)>,
    headers: HeaderMap,
) -> ApiResult<Json<ProtectedRef>> {
    let _auth = extract_auth(&headers).await?;
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Get protection rules
    let protection = state.index.get_protected_ref(repo.id, &ref_name).await?
        .ok_or_else(|| ApiError::Repo(format!("No protection rules found for {}/{}", repo_name, ref_name)))?;

    Ok(Json(protection))
}

/// Set branch protection rules for a repository reference
async fn set_protection(
    State(state): State<AppState>,
    Path((repo_name, ref_name)): Path<(String, String)>,
    headers: HeaderMap,
    Json(payload): Json<SetProtectionRequest>,
) -> ApiResult<StatusCode> {
    let auth = extract_auth(&headers).await?;
    
    // Check admin permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Create protection rules
    let protection = ProtectedRef {
        id: Uuid::new_v4(),
        repo_id: repo.id,
        ref_name: ref_name.clone(),
        require_admin: payload.require_admin,
        allow_fast_forward: payload.allow_fast_forward,
        allow_delete: payload.allow_delete,
        required_checks: payload.required_checks,
        required_reviewers: payload.required_reviewers,
        require_schema_pass: payload.require_schema_pass,
    };

    state.index.set_protected_ref(&protection).await?;

    // Audit log
    state.index.log_audit(
        &auth.sub,
        "set_protection",
        Some(&repo_name),
        Some(&ref_name),
        None,
        Some(&serde_json::to_value(&payload)?),
        None,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get repository quota configuration
async fn get_quota(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<RepoQuota>> {
    let _auth = extract_auth(&headers).await?;
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Get quota
    let quota = state.index.get_repo_quota(repo.id).await?
        .ok_or_else(|| ApiError::Repo(format!("No quota found for repository: {}", repo_name)))?;

    Ok(Json(quota))
}

/// Set repository quota configuration
async fn set_quota(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<SetQuotaRequest>,
) -> ApiResult<StatusCode> {
    let auth = extract_auth(&headers).await?;
    
    // Check admin permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Create quota
    let quota = RepoQuota {
        id: Uuid::new_v4(),
        repo_id: repo.id,
        bytes_soft: payload.bytes_soft,
        bytes_hard: payload.bytes_hard,
    };

    state.index.set_repo_quota(&quota).await?;

    // Audit log
    state.index.log_audit(
        &auth.sub,
        "set_quota",
        Some(&repo_name),
        None,
        None,
        Some(&serde_json::to_value(&payload)?),
        None,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get repository usage
async fn get_usage(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<QuotaStatusResponse>> {
    let _auth = extract_auth(&headers).await?;
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Get quota status
    let quota_status = state.index.get_quota_status(repo.id).await?
        .ok_or_else(|| ApiError::Repo(format!("No quota status found for repository: {}", repo_name)))?;

    Ok(Json(QuotaStatusResponse {
        quota: quota_status,
        repo_name,
    }))
}

/// Get retention policy for a repository
async fn get_retention(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<RepoRetention>> {
    let _auth = extract_auth(&headers).await?;
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Get retention policy
    let retention = state.index.get_repo_retention(repo.id).await?
        .ok_or_else(|| ApiError::Repo(format!("No retention policy found for repository: {}", repo_name)))?;

    Ok(Json(retention))
}

/// Set retention policy for a repository
async fn set_retention(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<SetRetentionRequest>,
) -> ApiResult<StatusCode> {
    let auth = extract_auth(&headers).await?;
    
    // Check admin permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Create retention policy
    let retention = RepoRetention {
        id: Uuid::new_v4(),
        repo_id: repo.id,
        retention_policy: RetentionPolicy {
            tombstone_days: payload.tombstone_days,
            hard_delete_days: payload.hard_delete_days,
            legal_hold: payload.legal_hold,
        },
    };

    state.index.set_repo_retention(&retention).await?;

    // Audit log
    state.index.log_audit(
        &auth.sub,
        "set_retention",
        Some(&repo_name),
        None,
        None,
        Some(&serde_json::to_value(&payload)?),
        None,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get webhooks for a repository
async fn get_webhooks(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<Webhook>>> {
    let _auth = extract_auth(&headers).await?;
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Get webhooks
    let webhooks = state.index.get_webhooks(repo.id).await?;

    Ok(Json(webhooks))
}

/// Create a webhook
async fn create_webhook(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<CreateWebhookRequest>,
) -> ApiResult<Json<Webhook>> {
    let auth = extract_auth(&headers).await?;
    
    // Check admin permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Create webhook
    let webhook = Webhook {
        id: Uuid::new_v4(),
        repo_id: repo.id,
        url: payload.url,
        secret: payload.secret,
        events: payload.events,
        active: true,
    };

    state.index.create_webhook(&webhook).await?;

    // Audit log
    state.index.log_audit(
        &auth.sub,
        "create_webhook",
        Some(&repo_name),
        None,
        None,
        Some(&serde_json::to_value(&payload)?),
        None,
    ).await?;

    Ok(Json(webhook))
}

/// Delete a webhook
async fn delete_webhook(
    State(state): State<AppState>,
    Path((repo_name, webhook_id)): Path<(String, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    let auth = extract_auth(&headers).await?;
    
    // Check admin permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin access required".to_string()));
    }
    
    // Delete webhook
    state.index.delete_webhook(webhook_id).await?;

    // Audit log
    state.index.log_audit(
        &auth.sub,
        "delete_webhook",
        Some(&repo_name),
        None,
        None,
        Some(&serde_json::json!({"webhook_id": webhook_id})),
        None,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get webhook delivery history
async fn get_webhook_deliveries(
    State(state): State<AppState>,
    Path((repo_name, webhook_id)): Path<(String, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Json<WebhookDeliveryResponse>> {
    let _auth = extract_auth(&headers).await?;
    
    // Query webhook delivery history from database
    let deliveries = sqlx::query_as!(
        WebhookDelivery,
        "SELECT id, webhook_id, event_type, payload, status, response_code, response_body, 
         created_at, updated_at, retry_count, next_retry_at
         FROM webhook_deliveries 
         WHERE webhook_id = $1 
         ORDER BY created_at DESC 
         LIMIT 100",
        webhook_id
    )
    .fetch_all(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch webhook deliveries: {}", e)))?;
    
    let total = deliveries.len() as u32;
    
    Ok(Json(WebhookDeliveryResponse {
        deliveries,
        total,
    }))
}

/// Create an export job
async fn create_export(
    State(state): State<AppState>,
    Path(repo_name): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<CreateExportRequest>,
) -> ApiResult<Json<ExportJobResponse>> {
    let auth = extract_auth(&headers).await?;
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Create export job
    let job = ExportJob {
        id: Uuid::new_v4(),
        repo_id: repo.id,
        user_id: auth.sub.clone(),
        manifest: payload.manifest,
        status: ExportJobStatus::Pending,
        s3_key: None,
        download_url: None,
        error_message: None,
    };

    state.index.create_export_job(&job).await?;

    // Audit log
    state.index.log_audit(
        &auth.sub,
        "create_export",
        Some(&repo_name),
        None,
        None,
        Some(&serde_json::to_value(&payload)?),
        None,
    ).await?;

    Ok(Json(ExportJobResponse {
        job,
        repo_name,
    }))
}

/// Get export job status
async fn get_export_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
    headers: HeaderMap,
) -> ApiResult<Json<ExportJob>> {
    let _auth = extract_auth(&headers).await?;
    
    // Get export job
    let job = state.index.get_export_job(job_id).await?
        .ok_or_else(|| ApiError::Repo(format!("Export job not found: {}", job_id)))?;

    Ok(Json(job))
}

/// Submit a check result
async fn submit_check(
    State(state): State<AppState>,
    Path((repo_name, ref_name)): Path<(String, String)>,
    headers: HeaderMap,
    Json(payload): Json<SubmitCheckRequest>,
) -> ApiResult<StatusCode> {
    let auth = extract_auth(&headers).await?;
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Get current commit for the reference
    let current_commit = state.index.get_current_commit(repo.id, &ref_name).await?
        .ok_or_else(|| ApiError::Repo(format!("No current commit found for {}/{}", repo_name, ref_name)))?;

    // Create check result
    let check = CheckResult {
        id: Uuid::new_v4(),
        repo_id: repo.id,
        ref_name: ref_name.clone(),
        commit_id: current_commit.id,
        check_name: payload.check_name,
        status: payload.status,
        details_url: payload.details_url,
        output: payload.output,
    };

    state.index.submit_check_result(&check).await?;

    // Audit log
    state.index.log_audit(
        &auth.sub,
        "submit_check",
        Some(&repo_name),
        Some(&ref_name),
        None,
        Some(&serde_json::to_value(&payload)?),
        None,
    ).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get check results for a commit
async fn get_checks(
    State(state): State<AppState>,
    Path((repo_name, ref_name, commit_id)): Path<(String, String, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<CheckResult>>> {
    let _auth = extract_auth(&headers).await?;
    
    // Get repository
    let repo = state.index.get_repo(&repo_name).await?
        .ok_or_else(|| ApiError::Repo(format!("Repository not found: {}", repo_name)))?;

    // Get check results
    let checks = state.index.get_check_results(repo.id, &ref_name, commit_id).await?;

    Ok(Json(checks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_get_protection_not_found() {
        // Implement test with mock index client
        use blacklake_index::{IndexClient, IndexError};
        use std::sync::Arc;
        use mockall::mock;

        mock! {
            IndexClient {}

            #[async_trait]
            impl blacklake_index::IndexClientTrait for IndexClient {
                async fn get_protection(&self, repo_id: &str, ref_name: &str) -> Result<Option<ProtectedRef>, IndexError>;
                async fn set_protection(&self, protection: ProtectedRef) -> Result<(), IndexError>;
                async fn get_quota(&self, repo_id: &str) -> Result<Option<RepoQuota>, IndexError>;
                async fn set_quota(&self, quota: RepoQuota) -> Result<(), IndexError>;
                async fn get_retention_policies(&self) -> Result<Vec<RetentionPolicy>, IndexError>;
                async fn create_retention_policy(&self, policy: RetentionPolicy) -> Result<(), IndexError>;
                async fn get_webhooks(&self, repo_id: Option<&str>) -> Result<Vec<Webhook>, IndexError>;
                async fn create_webhook(&self, webhook: Webhook) -> Result<(), IndexError>;
            }
        }

        let mut mock_client = MockIndexClient::new();
        mock_client.expect_get_protection()
            .with(mockall::predicate::eq("nonexistent-repo"), mockall::predicate::eq("main"))
            .times(1)
            .returning(|_, _| Ok(None));

        let app_state = AppState {
            index: Arc::new(mock_client),
            storage: Arc::new(blacklake_storage::StorageClient::new("test-bucket")),
            connector_manager: Arc::new(blacklake_connectors::ConnectorManager::new()),
        };

        let app = create_router(app_state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/repos/nonexistent-repo/protection/main")
                    .method("GET")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_set_protection_requires_admin() {
        // Implement test with mock index client
        use blacklake_index::{IndexClient, IndexError};
        use std::sync::Arc;
        use mockall::mock;

        mock! {
            IndexClient {}

            #[async_trait]
            impl blacklake_index::IndexClientTrait for IndexClient {
                async fn get_protection(&self, repo_id: &str, ref_name: &str) -> Result<Option<ProtectedRef>, IndexError>;
                async fn set_protection(&self, protection: ProtectedRef) -> Result<(), IndexError>;
                async fn get_quota(&self, repo_id: &str) -> Result<Option<RepoQuota>, IndexError>;
                async fn set_quota(&self, quota: RepoQuota) -> Result<(), IndexError>;
                async fn get_retention_policies(&self) -> Result<Vec<RetentionPolicy>, IndexError>;
                async fn create_retention_policy(&self, policy: RetentionPolicy) -> Result<(), IndexError>;
                async fn get_webhooks(&self, repo_id: Option<&str>) -> Result<Vec<Webhook>, IndexError>;
                async fn create_webhook(&self, webhook: Webhook) -> Result<(), IndexError>;
            }
        }

        let mock_client = MockIndexClient::new();
        let app_state = AppState {
            index: Arc::new(mock_client),
            storage: Arc::new(blacklake_storage::StorageClient::new("test-bucket")),
            connector_manager: Arc::new(blacklake_connectors::ConnectorManager::new()),
        };

        let app = create_router(app_state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/repos/test-repo/protection/main")
                    .method("PUT")
                    .header("Authorization", "Bearer user-token") // Non-admin token
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"require_admin_approval": true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_get_quota_success() {
        // Test successful quota retrieval
        use blacklake_index::{IndexClient, IndexError};
        use std::sync::Arc;
        use mockall::mock;

        mock! {
            IndexClient {}

            #[async_trait]
            impl blacklake_index::IndexClientTrait for IndexClient {
                async fn get_protection(&self, repo_id: &str, ref_name: &str) -> Result<Option<ProtectedRef>, IndexError>;
                async fn set_protection(&self, protection: ProtectedRef) -> Result<(), IndexError>;
                async fn get_quota(&self, repo_id: &str) -> Result<Option<RepoQuota>, IndexError>;
                async fn set_quota(&self, quota: RepoQuota) -> Result<(), IndexError>;
                async fn get_retention_policies(&self) -> Result<Vec<RetentionPolicy>, IndexError>;
                async fn create_retention_policy(&self, policy: RetentionPolicy) -> Result<(), IndexError>;
                async fn get_webhooks(&self, repo_id: Option<&str>) -> Result<Vec<Webhook>, IndexError>;
                async fn create_webhook(&self, webhook: Webhook) -> Result<(), IndexError>;
            }
        }

        let mut mock_client = MockIndexClient::new();
        let expected_quota = RepoQuota {
            repo_id: "test-repo".to_string(),
            soft_limit_gb: 1.0,
            hard_limit_gb: 2.0,
            current_usage_gb: 0.5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        mock_client.expect_get_quota()
            .with(mockall::predicate::eq("test-repo"))
            .times(1)
            .returning(move |_| Ok(Some(expected_quota.clone())));

        let app_state = AppState {
            index: Arc::new(mock_client),
            storage: Arc::new(blacklake_storage::StorageClient::new("test-bucket")),
            connector_manager: Arc::new(blacklake_connectors::ConnectorManager::new()),
        };

        let app = create_router(app_state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/repos/test-repo/quota")
                    .method("GET")
                    .header("Authorization", "Bearer admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
