// BlackLake Solr Search API
// Week 6: Advanced search with Solr integration

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use blacklake_core::{
    AuthContext, SearchRequest, SearchResponse,
};
use blacklake_core::search::{SolrClient, SolrStatus};
use crate::{ApiError, ApiResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use crate::AppState;
use crate::health::{
    SEARCH_REQUESTS_TOTAL, SEARCH_REQUEST_DURATION, SEARCH_RESULTS_COUNT,
    SOLR_OPERATIONS_TOTAL, SOLR_INDEX_DOCUMENTS_TOTAL,
};

/// Solr search request
#[derive(Debug, Deserialize)]
pub struct SolrSearchRequest {
    pub q: Option<String>,
    pub fq: Option<Vec<String>>,
    pub sort: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub suggest: Option<String>,
    pub json_facet: Option<serde_json::Value>,
}

/// Solr search response
#[derive(Debug, Serialize)]
pub struct SolrSearchResponse {
    pub docs: Vec<serde_json::Value>,
    pub num_found: u32,
    pub facets: Option<serde_json::Value>,
    pub suggestions: Option<Vec<String>>,
}

/// Reindex request
#[derive(Debug, Deserialize)]
pub struct ReindexRequest {
    pub repo_id: Option<uuid::Uuid>,
    pub since_commit_id: Option<uuid::Uuid>,
    pub full_reindex: Option<bool>,
}

/// Reindex response
#[derive(Debug, Serialize)]
pub struct ReindexResponse {
    pub job_id: uuid::Uuid,
    pub message: String,
}

/// Search endpoint with Solr
async fn solr_search(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<SolrSearchResponse>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Auth("User or admin role required".to_string()));
    }

    // Build search query
    let search_query = SearchQuery {
        q: params.get("q").cloned().unwrap_or_else(|| "*:*".to_string()),
        fq: params.get("fq").map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        sort: params.get("sort").cloned(),
        limit: params.get("limit").and_then(|s| s.parse().ok()),
        offset: params.get("offset").and_then(|s| s.parse().ok()),
        json_facet: params.get("json.facet")
            .and_then(|s| serde_json::from_str(s).ok()),
    };

    // Execute search with metrics
    let start_time = Instant::now();
    SEARCH_REQUESTS_TOTAL.inc();
    SOLR_OPERATIONS_TOTAL.inc();
    
    let response = state.solr_client.search(&search_query).await
        .map_err(|e| ApiError::Internal(format!("Search failed: {}", e)))?;

    // Record search metrics
    let duration = start_time.elapsed();
    SEARCH_REQUEST_DURATION.observe(duration.as_secs_f64());
    SEARCH_RESULTS_COUNT.observe(response.num_found as f64);

    // Get suggestions if requested
    let suggestions = if let Some(suggest_query) = params.get("suggest") {
        state.solr_client.suggest(suggest_query, 5).await.ok()
    } else {
        None
    };

    // Log audit
    state.index.log_audit(
        &auth.sub,
        "solr_search",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "query": search_query,
            "results_count": response.num_found,
            "has_suggestions": suggestions.is_some()
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(SolrSearchResponse {
        docs: response.docs,
        num_found: response.num_found,
        facets: response.facets,
        suggestions,
    })))
}

/// Get search suggestions
async fn get_suggestions(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Auth("User or admin role required".to_string()));
    }

    let query = params.get("q")
        .ok_or_else(|| ApiError::InvalidRequest("Missing 'q' parameter".to_string()))?;
    
    let count = params.get("count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    SOLR_OPERATIONS_TOTAL.inc();
    
    let suggestions = state.solr_client.suggest(query, count).await
        .map_err(|e| ApiError::Internal(format!("Suggest failed: {}", e)))?;

    Ok(Json(ApiResponse::success(suggestions)))
}

/// Get Solr schema information
async fn get_schema(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<serde_json::Value>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Auth("Admin role required".to_string()));
    }

    let schema = state.solr_client.get_schema().await
        .map_err(|e| ApiError::Internal(format!("Schema retrieval failed: {}", e)))?;

    Ok(Json(ApiResponse::success(schema)))
}

/// Get Solr status
async fn get_status(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<SolrStatus>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Auth("Admin role required".to_string()));
    }

    let status = state.solr_client.get_status().await
        .map_err(|e| ApiError::Internal(format!("Status retrieval failed: {}", e)))?;

    Ok(Json(ApiResponse::success(status)))
}

/// Trigger reindex job
async fn trigger_reindex(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<ReindexRequest>,
) -> Result<Json<ApiResponse<ReindexResponse>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Auth("Admin role required".to_string()));
    }

    let job_id = uuid::Uuid::new_v4();

    // Enqueue reindex job using Apalis
    let reindex_job = blacklake_core::jobs::FullReindexJob {
        repo_id: payload.repo_id,
        repo_name: payload.repo_name,
        since_commit_id: payload.since_commit_id,
        batch_size: payload.batch_size.unwrap_or(100),
    };
    
    // Enqueue the job using the job manager
    if let Some(job_manager) = &state.job_manager {
        job_manager.enqueue_job(
            blacklake_core::jobs::JobType::FullReindex,
            blacklake_core::jobs::JobData::FullReindex(reindex_job)
        ).await
        .map_err(|e| ApiError::Internal(format!("Failed to enqueue reindex job: {}", e)))?;
        
        tracing::info!("Reindex job {} enqueued for repo {}", job_id, payload.repo_name);
    } else {
        return Err(ApiError::Internal("Job manager not available".to_string()));
    }
    state.index.log_audit(
        &auth.sub,
        "reindex_triggered",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "job_id": job_id,
            "repo_id": payload.repo_id,
            "since_commit_id": payload.since_commit_id,
            "full_reindex": payload.full_reindex
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(ReindexResponse {
        job_id,
        message: "Reindex job queued successfully".to_string(),
    })))
}

/// Create Solr search API routes
pub fn create_solr_search_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/search", get(solr_search))
        .route("/v1/search/suggest", get(get_suggestions))
        .route("/v1/search/schema", get(get_schema))
        .route("/v1/search/status", get(get_status))
        .route("/v1/search/reindex", post(trigger_reindex))
}

