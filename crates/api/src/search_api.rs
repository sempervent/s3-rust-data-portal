// BlackLake Search API
// Week 4: Search backend abstraction with provider toggle

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use blacklake_core::{
    ApiError, ApiResponse, AuthContext, SearchQuery, SearchResponse, SearchProvider,
    SearchBackend, SearchBackendFactory, SearchConfig, SearchHealth, SearchMetrics,
};
use blacklake_index::IndexClient;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Search backend state
pub struct SearchState {
    pub backend: Arc<dyn SearchBackend>,
    pub provider: SearchProvider,
    pub index: Arc<IndexClient>,
}

/// Search request
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: Option<String>,
    pub file_type: Option<Vec<String>>,
    pub org_lab: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub repo: Option<Vec<String>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Search configuration request
#[derive(Debug, Deserialize)]
pub struct SearchConfigRequest {
    pub provider: String,
    pub postgres_config: Option<serde_json::Value>,
    pub opensearch_config: Option<serde_json::Value>,
}

/// Search endpoint
async fn search(
    State(state): State<SearchState>,
    Path(repo): Path<String>,
    auth: AuthContext,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<SearchResponse>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    // Get repository
    let repo_info = state.index.get_repo_by_name(&repo).await?;

    // Build search query from parameters
    let search_query = SearchQuery {
        query: params.get("q").cloned(),
        file_type: params.get("file_type")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        org_lab: params.get("org_lab")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        tags: params.get("tags")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        size_min: params.get("size_min")
            .and_then(|s| s.parse().ok()),
        size_max: params.get("size_max")
            .and_then(|s| s.parse().ok()),
        date_from: params.get("date_from").cloned(),
        date_to: params.get("date_to").cloned(),
        repo: Some(vec![repo_info.name]),
        limit: params.get("limit")
            .and_then(|s| s.parse().ok()),
        offset: params.get("offset")
            .and_then(|s| s.parse().ok()),
        sort_by: params.get("sort_by").cloned(),
        sort_order: params.get("sort_order")
            .and_then(|s| match s.as_str() {
                "asc" => Some(blacklake_core::SortOrder::Asc),
                "desc" => Some(blacklake_core::SortOrder::Desc),
                _ => None,
            }),
    };

    // Execute search
    let response = state.backend.search(&search_query).await
        .map_err(|e| ApiError::InternalServerError(format!("Search failed: {}", e)))?;

    // Log audit
    state.index.log_audit(
        &auth.sub,
        "search",
        Some(&repo),
        None,
        None,
        Some(&serde_json::json!({
            "query": search_query,
            "provider": state.provider,
            "results_count": response.results.len(),
            "took_ms": response.took_ms
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(response)))
}

/// Global search endpoint
async fn global_search(
    State(state): State<SearchState>,
    auth: AuthContext,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<SearchResponse>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"user".to_string()) && !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    // Build search query from parameters
    let search_query = SearchQuery {
        query: params.get("q").cloned(),
        file_type: params.get("file_type")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        org_lab: params.get("org_lab")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        tags: params.get("tags")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        size_min: params.get("size_min")
            .and_then(|s| s.parse().ok()),
        size_max: params.get("size_max")
            .and_then(|s| s.parse().ok()),
        date_from: params.get("date_from").cloned(),
        date_to: params.get("date_to").cloned(),
        repo: params.get("repo")
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
        limit: params.get("limit")
            .and_then(|s| s.parse().ok()),
        offset: params.get("offset")
            .and_then(|s| s.parse().ok()),
        sort_by: params.get("sort_by").cloned(),
        sort_order: params.get("sort_order")
            .and_then(|s| match s.as_str() {
                "asc" => Some(blacklake_core::SortOrder::Asc),
                "desc" => Some(blacklake_core::SortOrder::Desc),
                _ => None,
            }),
    };

    // Execute search
    let response = state.backend.search(&search_query).await
        .map_err(|e| ApiError::InternalServerError(format!("Search failed: {}", e)))?;

    // Log audit
    state.index.log_audit(
        &auth.sub,
        "global_search",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "query": search_query,
            "provider": state.provider,
            "results_count": response.results.len(),
            "took_ms": response.took_ms
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(response)))
}

/// Get search health
async fn search_health(
    State(state): State<SearchState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<SearchHealth>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    let health = state.backend.health_check().await
        .map_err(|e| ApiError::InternalServerError(format!("Health check failed: {}", e)))?;

    Ok(Json(ApiResponse::success(health)))
}

/// Get search metrics
async fn search_metrics(
    State(state): State<SearchState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<SearchMetrics>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    let metrics = state.backend.get_metrics().await
        .map_err(|e| ApiError::InternalServerError(format!("Metrics retrieval failed: {}", e)))?;

    Ok(Json(ApiResponse::success(metrics)))
}

/// Update search configuration
async fn update_search_config(
    State(state): State<SearchState>,
    auth: AuthContext,
    Json(payload): Json<SearchConfigRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    // Parse new provider
    let new_provider = SearchProvider::from_str(&payload.provider)
        .map_err(|e| ApiError::BadRequest(format!("Invalid search provider: {}", e)))?;

    // Log audit
    state.index.log_audit(
        &auth.sub,
        "search_config_updated",
        None,
        None,
        None,
        Some(&serde_json::json!({
            "old_provider": state.provider,
            "new_provider": new_provider,
            "config": payload
        })),
        None,
    ).await?;

    // In a real implementation, this would update the search configuration
    // and restart the search backend with the new configuration
    // For now, we just log the change

    Ok(Json(ApiResponse::success(())))
}

/// Get search configuration
async fn get_search_config(
    State(state): State<SearchState>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<SearchConfigResponse>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    let config = SearchConfigResponse {
        provider: state.provider.clone(),
        available_providers: vec![
            SearchProvider::Postgres,
            SearchProvider::OpenSearch,
        ],
    };

    Ok(Json(ApiResponse::success(config)))
}

/// Search configuration response
#[derive(Debug, Serialize)]
struct SearchConfigResponse {
    pub provider: SearchProvider,
    pub available_providers: Vec<SearchProvider>,
}

/// Create search API routes
pub fn create_search_routes() -> Router<SearchState> {
    Router::new()
        .route("/repos/:repo/search", get(search))
        .route("/search", get(global_search))
        .route("/search/health", get(search_health))
        .route("/search/metrics", get(search_metrics))
        .route("/search/config", get(get_search_config))
        .route("/search/config", post(update_search_config))
}
