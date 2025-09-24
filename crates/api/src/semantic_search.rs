// Semantic search API endpoints
// Week 8: AI-assisted metadata and search

use axum::{
    extract::{Query, State, Json},
    http::StatusCode,
    response::Json as AxumJson,
    routing::{get, post},
    Router,
};
use blacklake_core::{
    AuthContext,
};
use blacklake_core::embeddings::{EmbeddingService, MockEmbeddingService, SemanticSearchRequest, SemanticSearchResponse, EmbeddingRequest, EmbeddingResponse, SuggestedTags};
use crate::{ApiError, ApiResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::AppState;

/// Semantic search query parameters
#[derive(Debug, Deserialize)]
pub struct SemanticSearchQuery {
    pub q: String,
    pub limit: Option<usize>,
    pub threshold: Option<f32>,
    pub repo: Option<String>,
    pub classification: Option<String>,
}

/// Semantic search response
#[derive(Debug, Serialize)]
pub struct SemanticSearchApiResponse {
    pub results: Vec<SemanticSearchResult>,
    pub total: usize,
    pub processing_time_ms: u64,
    pub query: String,
}

/// Semantic search result
#[derive(Debug, Serialize)]
pub struct SemanticSearchResult {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub similarity_score: f32,
    pub source_type: String,
    pub repo_name: Option<String>,
    pub path: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<u64>,
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Vec<String>,
    pub classification: Option<String>,
}

/// Generate embedding request
#[derive(Debug, Deserialize)]
pub struct GenerateEmbeddingRequest {
    pub text: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Generate embedding response
#[derive(Debug, Serialize)]
pub struct GenerateEmbeddingResponse {
    pub embedding: Vec<f32>,
    pub model: String,
    pub dimensions: usize,
    pub processing_time_ms: u64,
}

/// Extract suggested tags request
#[derive(Debug, Deserialize)]
pub struct ExtractTagsRequest {
    pub text: String,
}

/// Extract suggested tags response
#[derive(Debug, Serialize)]
pub struct ExtractTagsResponse {
    pub entity_tags: Vec<String>,
    pub keyword_tags: Vec<String>,
    pub confidence_scores: HashMap<String, f32>,
    pub processing_time_ms: u64,
}

/// Semantic search endpoint
async fn semantic_search(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(params): Query<SemanticSearchQuery>,
) -> Result<AxumJson<ApiResponse<SemanticSearchApiResponse>>, ApiError> {
    // Check search access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_access(
        &auth.sub,
        "search",
        "search",
        HashMap::new(),
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Create embedding service (in production, this would be injected)
    let embedding_service = Arc::new(MockEmbeddingService::new());
    
    // Build search request
    let mut filters = HashMap::new();
    if let Some(repo) = params.repo {
        filters.insert("repo_name".to_string(), serde_json::Value::String(repo));
    }
    if let Some(classification) = params.classification {
        filters.insert("classification".to_string(), serde_json::Value::String(classification));
    }
    
    let search_request = SemanticSearchRequest {
        query: params.q.clone(),
        limit: params.limit,
        threshold: params.threshold,
        filters: if filters.is_empty() { None } else { Some(filters) },
    };
    
    // Perform semantic search
    let search_response = embedding_service.semantic_search(&search_request).await
        .map_err(|e| ApiError::Internal(format!("Semantic search failed: {}", e)))?;
    
    // Convert results to API format
    let results: Vec<SemanticSearchResult> = search_response.results.into_iter().map(|result| {
        SemanticSearchResult {
            id: result.id,
            title: result.title,
            description: result.description,
            url: result.url,
            similarity_score: result.similarity_score,
            source_type: result.source_type,
            repo_name: result.metadata.get("repo_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            path: result.metadata.get("path").and_then(|v| v.as_str()).map(|s| s.to_string()),
            content_type: result.metadata.get("content_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
            size: result.metadata.get("size").and_then(|v| v.as_u64()),
            modified_at: result.metadata.get("modified_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            tags: result.metadata.get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            classification: result.metadata.get("classification").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }).collect();
    
    let response = SemanticSearchApiResponse {
        results,
        total: search_response.total,
        processing_time_ms: search_response.processing_time_ms,
        query: params.q,
    };
    
    Ok(AxumJson(ApiResponse::success(response)))
}

/// Generate embedding endpoint
async fn generate_embedding(
    State(_state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<GenerateEmbeddingRequest>,
) -> Result<AxumJson<ApiResponse<GenerateEmbeddingResponse>>, ApiError> {
    // Check admin access for embedding generation
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "embeddings",
        &_state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Create embedding service
    let embedding_service = Arc::new(MockEmbeddingService::new());
    
    // Generate embedding
    let request = EmbeddingRequest {
        text: payload.text,
        metadata: payload.metadata,
    };
    
    let response = embedding_service.generate_embedding(&request).await
        .map_err(|e| ApiError::Internal(format!("Embedding generation failed: {}", e)))?;
    
    let api_response = GenerateEmbeddingResponse {
        embedding: response.embedding,
        model: response.model,
        dimensions: response.dimensions,
        processing_time_ms: response.processing_time_ms,
    };
    
    Ok(AxumJson(ApiResponse::success(api_response)))
}

/// Extract suggested tags endpoint
async fn extract_suggested_tags(
    State(_state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<ExtractTagsRequest>,
) -> Result<AxumJson<ApiResponse<ExtractTagsResponse>>, ApiError> {
    // Check admin access for tag extraction
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "tags",
        &_state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Create embedding service
    let embedding_service = Arc::new(MockEmbeddingService::new());
    
    // Extract suggested tags
    let tags = embedding_service.extract_suggested_tags(&payload.text).await
        .map_err(|e| ApiError::Internal(format!("Tag extraction failed: {}", e)))?;
    
    let response = ExtractTagsResponse {
        entity_tags: tags.entity_tags,
        keyword_tags: tags.keyword_tags,
        confidence_scores: tags.confidence_scores,
        processing_time_ms: tags.processing_time_ms,
    };
    
    Ok(AxumJson(ApiResponse::success(response)))
}

/// Get embedding model info
async fn get_embedding_model_info(
    State(_state): State<AppState>,
    auth: AuthContext,
) -> Result<AxumJson<ApiResponse<serde_json::Value>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "embeddings",
        &_state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Create embedding service
    let embedding_service = Arc::new(MockEmbeddingService::new());
    let config = embedding_service.get_model_config();
    
    let model_info = serde_json::json!({
        "model_name": config.model_name,
        "dimensions": config.dimensions,
        "max_text_length": config.max_text_length,
        "batch_size": config.batch_size,
        "status": "active"
    });
    
    Ok(AxumJson(ApiResponse::success(model_info)))
}

/// Create semantic search routes
pub fn create_semantic_search_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/search/semantic", get(semantic_search))
        .route("/v1/embeddings/generate", post(generate_embedding))
        .route("/v1/embeddings/suggested-tags", post(extract_suggested_tags))
        .route("/v1/embeddings/model-info", get(get_embedding_model_info))
}
