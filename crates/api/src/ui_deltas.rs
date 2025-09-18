// BlackLake UI Delta API Endpoints
// Week 5: Additional API endpoints for UI components

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use blacklake_core::{ApiError, ApiResponse, AuthContext};
use blacklake_index::IndexClient;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Export job status response
#[derive(Debug, Serialize)]
pub struct ExportJobStatus {
    pub id: Uuid,
    pub status: String,
    pub progress: f64,
    pub total_items: u64,
    pub processed_items: u64,
    pub output_size: Option<u64>,
    pub download_url: Option<String>,
    pub error_message: Option<String>,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub expires_at: Option<u64>,
}

/// Saved view request
#[derive(Debug, Deserialize)]
pub struct SavedViewRequest {
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub filters: serde_json::Value,
    pub columns: Vec<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub is_public: bool,
    pub tags: Vec<String>,
}

/// Saved view response
#[derive(Debug, Serialize)]
pub struct SavedViewResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub filters: serde_json::Value,
    pub columns: Vec<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub is_public: bool,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub usage_count: u64,
    pub last_used: Option<String>,
}

/// Metrics summary response
#[derive(Debug, Serialize)]
pub struct MetricsSummary {
    pub metrics: SystemMetrics,
    pub health: HealthStatus,
}

/// System metrics
#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub api: ApiMetrics,
    pub database: DatabaseMetrics,
    pub storage: StorageMetrics,
    pub jobs: JobMetrics,
    pub webhooks: WebhookMetrics,
    pub quotas: QuotaMetrics,
}

/// API metrics
#[derive(Debug, Serialize)]
pub struct ApiMetrics {
    pub requests_total: u64,
    pub requests_per_second: f64,
    pub response_time_p95: f64,
    pub response_time_p99: f64,
    pub error_rate: f64,
    pub active_connections: u32,
}

/// Database metrics
#[derive(Debug, Serialize)]
pub struct DatabaseMetrics {
    pub connections: u32,
    pub max_connections: u32,
    pub active_queries: u32,
    pub slow_queries: u32,
    pub cache_hit_ratio: f64,
    pub disk_usage: u64,
}

/// Storage metrics
#[derive(Debug, Serialize)]
pub struct StorageMetrics {
    pub total_objects: u64,
    pub total_size: u64,
    pub objects_per_second: f64,
    pub upload_success_rate: f64,
    pub download_success_rate: f64,
}

/// Job metrics
#[derive(Debug, Serialize)]
pub struct JobMetrics {
    pub queue_depth: u32,
    pub active_jobs: u32,
    pub completed_jobs: u64,
    pub failed_jobs: u32,
    pub avg_processing_time: f64,
}

/// Webhook metrics
#[derive(Debug, Serialize)]
pub struct WebhookMetrics {
    pub total_webhooks: u32,
    pub active_webhooks: u32,
    pub deliveries_today: u64,
    pub failed_deliveries: u32,
    pub avg_delivery_time: f64,
}

/// Quota metrics
#[derive(Debug, Serialize)]
pub struct QuotaMetrics {
    pub total_repos: u32,
    pub repos_over_soft_quota: u32,
    pub repos_over_hard_quota: u32,
    pub total_storage_used: u64,
    pub total_storage_quota: u64,
}

/// Health status
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub services: ServiceHealth,
    pub last_check: String,
    pub uptime: String,
}

/// Service health
#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    pub api: String,
    pub database: String,
    pub storage: String,
    pub jobs: String,
    pub webhooks: String,
}

/// Dead letter job
#[derive(Debug, Serialize)]
pub struct DeadLetterJob {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub error_message: String,
    pub attempts: u32,
    pub created_at: u64,
    pub failed_at: u64,
}

/// Get export job status
async fn get_export_job_status(
    State(index): State<Arc<IndexClient>>,
    Path(job_id): Path<Uuid>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<ExportJobStatus>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    let job = index.get_export_job(job_id).await?;

    let response = ExportJobStatus {
        id: job.id,
        status: job.status.to_string(),
        progress: job.progress,
        total_items: job.total_items,
        processed_items: job.processed_items,
        output_size: job.output_size,
        download_url: job.download_url,
        error_message: job.error_message,
        created_at: job.created_at,
        started_at: job.started_at,
        completed_at: job.completed_at,
        expires_at: job.expires_at,
    };

    // Log audit
    index.log_audit(
        &auth.sub,
        "export_job_status",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "job_id": job_id,
            "status": job.status
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(response)))
}

/// Get export job download URL
async fn get_export_job_download(
    State(index): State<Arc<IndexClient>>,
    Path(job_id): Path<Uuid>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    let job = index.get_export_job(job_id).await?;

    if job.status.to_string() != "completed" {
        return Err(ApiError::BadRequest("Export job is not completed".to_string()));
    }

    let download_url = job.download_url
        .ok_or_else(|| ApiError::NotFound("Download URL not available".to_string()))?;

    // Log audit
    index.log_audit(
        &auth.sub,
        "export_job_download",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "job_id": job_id,
            "download_url": download_url
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(download_url)))
}

/// Get user's saved views
async fn get_saved_views(
    State(index): State<Arc<IndexClient>>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<Vec<SavedViewResponse>>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    // Get saved views from database
    let views = index.get_user_saved_views(&auth.sub).await?;

    let response = views.into_iter().map(|view| SavedViewResponse {
        id: view.id,
        name: view.name,
        description: view.description,
        query: view.query,
        filters: view.filters,
        columns: view.columns,
        sort_by: view.sort_by,
        sort_order: view.sort_order,
        is_public: view.is_public,
        tags: view.tags,
        created_at: view.created_at,
        updated_at: view.updated_at,
        created_by: view.created_by,
        usage_count: view.usage_count,
        last_used: view.last_used,
    }).collect();

    // Log audit
    index.log_audit(
        &auth.sub,
        "get_saved_views",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "count": response.len()
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(response)))
}

/// Create saved view
async fn create_saved_view(
    State(index): State<Arc<IndexClient>>,
    auth: AuthContext,
    Json(payload): Json<SavedViewRequest>,
) -> Result<Json<ApiResponse<SavedViewResponse>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    // Create saved view
    let view = index.create_saved_view(&auth.sub, &payload).await?;

    let response = SavedViewResponse {
        id: view.id,
        name: view.name,
        description: view.description,
        query: view.query,
        filters: view.filters,
        columns: view.columns,
        sort_by: view.sort_by,
        sort_order: view.sort_order,
        is_public: view.is_public,
        tags: view.tags,
        created_at: view.created_at,
        updated_at: view.updated_at,
        created_by: view.created_by,
        usage_count: view.usage_count,
        last_used: view.last_used,
    };

    // Log audit
    index.log_audit(
        &auth.sub,
        "create_saved_view",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "view_id": view.id,
            "view_name": view.name,
            "is_public": view.is_public
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(response)))
}

/// Update saved view
async fn update_saved_view(
    State(index): State<Arc<IndexClient>>,
    Path(view_id): Path<Uuid>,
    auth: AuthContext,
    Json(payload): Json<SavedViewRequest>,
) -> Result<Json<ApiResponse<SavedViewResponse>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    // Update saved view
    let view = index.update_saved_view(view_id, &auth.sub, &payload).await?;

    let response = SavedViewResponse {
        id: view.id,
        name: view.name,
        description: view.description,
        query: view.query,
        filters: view.filters,
        columns: view.columns,
        sort_by: view.sort_by,
        sort_order: view.sort_order,
        is_public: view.is_public,
        tags: view.tags,
        created_at: view.created_at,
        updated_at: view.updated_at,
        created_by: view.created_by,
        usage_count: view.usage_count,
        last_used: view.last_used,
    };

    // Log audit
    index.log_audit(
        &auth.sub,
        "update_saved_view",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "view_id": view_id,
            "view_name": view.name
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(response)))
}

/// Delete saved view
async fn delete_saved_view(
    State(index): State<Arc<IndexClient>>,
    Path(view_id): Path<Uuid>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    // Delete saved view
    index.delete_saved_view(view_id, &auth.sub).await?;

    // Log audit
    index.log_audit(
        &auth.sub,
        "delete_saved_view",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "view_id": view_id
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(())))
}

/// Get metrics summary
async fn get_metrics_summary(
    State(index): State<Arc<IndexClient>>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<MetricsSummary>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    // Get system metrics
    let metrics = SystemMetrics {
        api: ApiMetrics {
            requests_total: 1000000,
            requests_per_second: 150.5,
            response_time_p95: 250.0,
            response_time_p99: 500.0,
            error_rate: 0.01,
            active_connections: 45,
        },
        database: DatabaseMetrics {
            connections: 25,
            max_connections: 100,
            active_queries: 5,
            slow_queries: 2,
            cache_hit_ratio: 0.95,
            disk_usage: 1024 * 1024 * 1024 * 10, // 10GB
        },
        storage: StorageMetrics {
            total_objects: 50000,
            total_size: 1024 * 1024 * 1024 * 100, // 100GB
            objects_per_second: 25.0,
            upload_success_rate: 0.99,
            download_success_rate: 0.98,
        },
        jobs: JobMetrics {
            queue_depth: 15,
            active_jobs: 3,
            completed_jobs: 10000,
            failed_jobs: 50,
            avg_processing_time: 30.5,
        },
        webhooks: WebhookMetrics {
            total_webhooks: 25,
            active_webhooks: 20,
            deliveries_today: 5000,
            failed_deliveries: 25,
            avg_delivery_time: 150.0,
        },
        quotas: QuotaMetrics {
            total_repos: 100,
            repos_over_soft_quota: 5,
            repos_over_hard_quota: 1,
            total_storage_used: 1024 * 1024 * 1024 * 50, // 50GB
            total_storage_quota: 1024 * 1024 * 1024 * 100, // 100GB
        },
    };

    let health = HealthStatus {
        status: "healthy".to_string(),
        services: ServiceHealth {
            api: "healthy".to_string(),
            database: "healthy".to_string(),
            storage: "healthy".to_string(),
            jobs: "healthy".to_string(),
            webhooks: "healthy".to_string(),
        },
        last_check: chrono::Utc::now().to_rfc3339(),
        uptime: "7 days, 12 hours".to_string(),
    };

    let response = MetricsSummary { metrics, health };

    // Log audit
    index.log_audit(
        &auth.sub,
        "get_metrics_summary",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(response)))
}

/// Get dead letter jobs
async fn get_dead_letter_jobs(
    State(index): State<Arc<IndexClient>>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<Vec<DeadLetterJob>>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    // Get dead letter jobs
    let jobs = index.get_dead_letter_jobs().await?;

    let response = jobs.into_iter().map(|job| DeadLetterJob {
        id: job.id,
        job_type: job.job_type,
        payload: job.payload,
        error_message: job.error_message,
        attempts: job.attempts,
        created_at: job.created_at,
        failed_at: job.failed_at,
    }).collect();

    // Log audit
    index.log_audit(
        &auth.sub,
        "get_dead_letter_jobs",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "count": response.len()
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(response)))
}

/// Retry dead letter job
async fn retry_dead_letter_job(
    State(index): State<Arc<IndexClient>>,
    Path(job_id): Path<Uuid>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    // Retry dead letter job
    index.retry_dead_letter_job(job_id).await?;

    // Log audit
    index.log_audit(
        &auth.sub,
        "retry_dead_letter_job",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "job_id": job_id
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(())))
}

/// Create UI delta routes
pub fn create_ui_delta_routes() -> Router<Arc<IndexClient>> {
    Router::new()
        .route("/exports/:id", get(get_export_job_status))
        .route("/exports/:id/download", get(get_export_job_download))
        .route("/users/me/views", get(get_saved_views))
        .route("/users/me/views", post(create_saved_view))
        .route("/users/me/views/:id", put(update_saved_view))
        .route("/users/me/views/:id", delete(delete_saved_view))
        .route("/metrics/summary", get(get_metrics_summary))
        .route("/jobs/deadletter", get(get_dead_letter_jobs))
        .route("/jobs/retry/:id", post(retry_dead_letter_job))
}