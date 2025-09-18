use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use blacklake_index::IndexClient;
use blacklake_storage::StorageClient;
use prometheus::{Encoder, TextEncoder, Registry, Counter, Histogram, Gauge};
use serde_json::json;
use std::sync::Arc;
use tokio::time::timeout;
use tracing::{error, info};

#[derive(Clone)]
pub struct HealthState {
    pub index: IndexClient,
    pub storage: StorageClient,
    pub metrics: Arc<Registry>,
}

// Prometheus metrics
lazy_static::lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: Counter = Counter::new(
        "http_requests_total",
        "Total number of HTTP requests"
    ).unwrap();
    
    pub static ref HTTP_REQUEST_DURATION: Histogram = Histogram::new(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    ).unwrap();
    
    pub static ref ACTIVE_CONNECTIONS: Gauge = Gauge::new(
        "active_connections",
        "Number of active connections"
    ).unwrap();
    
    pub static ref DATABASE_CONNECTIONS: Gauge = Gauge::new(
        "database_connections_active",
        "Number of active database connections"
    ).unwrap();
    
    pub static ref S3_OPERATIONS_TOTAL: Counter = Counter::new(
        "s3_operations_total",
        "Total number of S3 operations"
    ).unwrap();
    
    // Search metrics
    pub static ref SEARCH_REQUESTS_TOTAL: Counter = Counter::new(
        "search_requests_total",
        "Total number of search requests"
    ).unwrap();
    
    pub static ref SEARCH_REQUEST_DURATION: Histogram = Histogram::new(
        "search_request_duration_seconds",
        "Search request duration in seconds"
    ).unwrap();
    
    pub static ref SEARCH_RESULTS_COUNT: Histogram = Histogram::new(
        "search_results_count",
        "Number of results returned by search queries"
    ).unwrap();
    
    pub static ref SOLR_OPERATIONS_TOTAL: Counter = Counter::new(
        "solr_operations_total",
        "Total number of Solr operations"
    ).unwrap();
    
    pub static ref SOLR_INDEX_DOCUMENTS_TOTAL: Counter = Counter::new(
        "solr_index_documents_total",
        "Total number of documents indexed in Solr"
    ).unwrap();
    
    // Session metrics
    pub static ref SESSION_CREATIONS_TOTAL: Counter = Counter::new(
        "session_creations_total",
        "Total number of session creations"
    ).unwrap();
    
    pub static ref SESSION_DESTROYALS_TOTAL: Counter = Counter::new(
        "session_destroyals_total",
        "Total number of session destructions"
    ).unwrap();
    
    pub static ref ACTIVE_SESSIONS: Gauge = Gauge::new(
        "active_sessions",
        "Number of active sessions"
    ).unwrap();
    
    pub static ref CSRF_TOKEN_REQUESTS_TOTAL: Counter = Counter::new(
        "csrf_token_requests_total",
        "Total number of CSRF token requests"
    ).unwrap();
    
    pub static ref CSRF_TOKEN_VALIDATIONS_TOTAL: Counter = Counter::new(
        "csrf_token_validations_total",
        "Total number of CSRF token validations"
    ).unwrap();
    
    pub static ref CSRF_TOKEN_VALIDATION_FAILURES_TOTAL: Counter = Counter::new(
        "csrf_token_validation_failures_total",
        "Total number of CSRF token validation failures"
    ).unwrap();
    
    // Job metrics
    pub static ref JOB_ENQUEUED_TOTAL: Counter = Counter::new(
        "job_enqueued_total",
        "Total number of jobs enqueued"
    ).unwrap();
    
    pub static ref JOB_PROCESSED_TOTAL: Counter = Counter::new(
        "job_processed_total",
        "Total number of jobs processed"
    ).unwrap();
    
    pub static ref JOB_FAILED_TOTAL: Counter = Counter::new(
        "job_failed_total",
        "Total number of failed jobs"
    ).unwrap();
    
    pub static ref JOB_PROCESSING_DURATION: Histogram = Histogram::new(
        "job_processing_duration_seconds",
        "Job processing duration in seconds"
    ).unwrap();
    
    pub static ref QUEUE_SIZE: Gauge = Gauge::new(
        "queue_size",
        "Current size of job queues"
    ).unwrap();
}

pub async fn liveness_check() -> (StatusCode, Json<serde_json::Value>) {
    info!("Liveness check requested");
    
    let response = json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    });
    
    (StatusCode::OK, Json(response))
}

pub async fn readiness_check(
    State(state): State<HealthState>,
) -> (StatusCode, Json<serde_json::Value>) {
    info!("Readiness check requested");
    
    let mut checks = json!({
        "status": "ready",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "checks": {}
    });

    // Check database connectivity
    let db_check = timeout(
        std::time::Duration::from_secs(5),
        check_database(&state.index)
    ).await;

    match db_check {
        Ok(Ok(_)) => {
            checks["checks"]["database"] = json!({
                "status": "healthy",
                "message": "Database connection successful"
            });
        }
        Ok(Err(e)) => {
            error!("Database health check failed: {}", e);
            checks["checks"]["database"] = json!({
                "status": "unhealthy",
                "message": format!("Database error: {}", e)
            });
            checks["status"] = json!("not_ready");
        }
        Err(_) => {
            error!("Database health check timeout");
            checks["checks"]["database"] = json!({
                "status": "unhealthy",
                "message": "Database connection timeout"
            });
            checks["status"] = json!("not_ready");
        }
    }

    // Check S3 connectivity
    let s3_check = timeout(
        std::time::Duration::from_secs(5),
        check_storage(&state.storage)
    ).await;

    match s3_check {
        Ok(Ok(_)) => {
            checks["checks"]["storage"] = json!({
                "status": "healthy",
                "message": "S3 connection successful"
            });
        }
        Ok(Err(e)) => {
            error!("Storage health check failed: {}", e);
            checks["checks"]["storage"] = json!({
                "status": "unhealthy",
                "message": format!("Storage error: {}", e)
            });
            checks["status"] = json!("not_ready");
        }
        Err(_) => {
            error!("Storage health check timeout");
            checks["checks"]["storage"] = json!({
                "status": "unhealthy",
                "message": "Storage connection timeout"
            });
            checks["status"] = json!("not_ready");
        }
    }

    let status = if checks["status"] == "ready" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(checks))
}

pub async fn metrics(
    State(state): State<HealthState>,
) -> (StatusCode, String) {
    let encoder = TextEncoder::new();
    let metric_families = state.metrics.gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(e) => {
            error!("Failed to encode metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e))
        }
    }
}

async fn check_database(index: &IndexClient) -> Result<(), String> {
    // Simple query to check database connectivity
    match index.list_repos().await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Database query failed: {}", e)),
    }
}

async fn check_storage(storage: &StorageClient) -> Result<(), String> {
    // Check if bucket exists and is accessible
    match storage.ensure_bucket_exists().await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Storage check failed: {}", e)),
    }
}

pub fn create_metrics_registry() -> Registry {
    let registry = Registry::new();
    
    // Register HTTP metrics
    registry.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
    registry.register(Box::new(HTTP_REQUEST_DURATION.clone())).unwrap();
    registry.register(Box::new(ACTIVE_CONNECTIONS.clone())).unwrap();
    registry.register(Box::new(DATABASE_CONNECTIONS.clone())).unwrap();
    registry.register(Box::new(S3_OPERATIONS_TOTAL.clone())).unwrap();
    
    // Register search metrics
    registry.register(Box::new(SEARCH_REQUESTS_TOTAL.clone())).unwrap();
    registry.register(Box::new(SEARCH_REQUEST_DURATION.clone())).unwrap();
    registry.register(Box::new(SEARCH_RESULTS_COUNT.clone())).unwrap();
    registry.register(Box::new(SOLR_OPERATIONS_TOTAL.clone())).unwrap();
    registry.register(Box::new(SOLR_INDEX_DOCUMENTS_TOTAL.clone())).unwrap();
    
    // Register session metrics
    registry.register(Box::new(SESSION_CREATIONS_TOTAL.clone())).unwrap();
    registry.register(Box::new(SESSION_DESTROYALS_TOTAL.clone())).unwrap();
    registry.register(Box::new(ACTIVE_SESSIONS.clone())).unwrap();
    registry.register(Box::new(CSRF_TOKEN_REQUESTS_TOTAL.clone())).unwrap();
    registry.register(Box::new(CSRF_TOKEN_VALIDATIONS_TOTAL.clone())).unwrap();
    registry.register(Box::new(CSRF_TOKEN_VALIDATION_FAILURES_TOTAL.clone())).unwrap();
    
    // Register job metrics
    registry.register(Box::new(JOB_ENQUEUED_TOTAL.clone())).unwrap();
    registry.register(Box::new(JOB_PROCESSED_TOTAL.clone())).unwrap();
    registry.register(Box::new(JOB_FAILED_TOTAL.clone())).unwrap();
    registry.register(Box::new(JOB_PROCESSING_DURATION.clone())).unwrap();
    registry.register(Box::new(QUEUE_SIZE.clone())).unwrap();
    
    registry
}
