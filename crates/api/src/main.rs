use axum::{
    extract::{Path, Query, State, Request},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router, middleware,
};
use blacklake_core::{
    AuthContext, CanonicalMeta, Change, ChangeOp, CommitRequest, CommitResponse, CreateRepoRequest,
    CreateRepoResponse, generate_subject_iri, JwtClaims, MetadataSchema, project_to_index,
    RdfFormat, SearchRequest, SearchResponse, TreeResponse, TreeEntry, UploadInitRequest, 
    UploadInitResponse, canonical_to_dc_jsonld, canonical_to_turtle, validate_repo_name,
    normalize_path, validate_meta, validate_content_type, validate_file_size,
};
use blacklake_index::{IndexClient, IndexError};
use blacklake_storage::{StorageClient, StorageError};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse},
};
use tracing::{info, warn, instrument, Span};
use uuid::Uuid;

// Import new modules
mod auth;
mod health;
mod rate_limit;

use auth::{AuthLayer, auth_middleware, request_id_middleware, create_auth_layer};
use health::{HealthState, liveness_check, readiness_check, metrics, create_metrics_registry};
use rate_limit::{RateLimitState, rate_limit_middleware, create_rate_limit_config, start_rate_limit_cleanup};

#[derive(Clone)]
struct AppState {
    index: IndexClient,
    storage: StorageClient,
    auth_layer: AuthLayer,
    rate_limit_state: RateLimitState,
    health_state: HealthState,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Repository error: {0}")]
    Repo(String),
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Index error: {0}")]
    Index(#[from] IndexError),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Repo(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Storage(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            ApiError::Index(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            ApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
            "timestamp": Utc::now()
        }));

        (status, body).into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration from environment
    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("APP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .map_err(|e| anyhow::anyhow!("Invalid APP_PORT: {}", e))?;

    // Initialize clients
    let index = IndexClient::from_env().await?;
    let storage = StorageClient::from_env().await?;
    
    // Initialize auth layer
    let auth_layer = create_auth_layer()?;
    
    // Initialize rate limiting
    let rate_limit_config = create_rate_limit_config();
    let rate_limit_state = RateLimitState::new(rate_limit_config);
    
    // Initialize metrics
    let metrics_registry = create_metrics_registry();
    let health_state = HealthState {
        index: index.clone(),
        storage: storage.clone(),
        metrics: Arc::new(metrics_registry),
    };

    let state = AppState { 
        index, 
        storage, 
        auth_layer,
        rate_limit_state,
        health_state,
    };

    // Build the application
    let app = Router::new()
        // Health endpoints (no auth required)
        .route("/live", get(liveness_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(metrics))
        // API endpoints
        .route("/v1/repos", post(create_repo).get(list_repos))
        .route("/v1/repos/:repo/upload-init", post(upload_init))
        .route("/v1/repos/:repo/commit", post(commit))
        .route("/v1/repos/:repo/blob/:ref/*path", get(get_blob))
        .route("/v1/repos/:repo/tree/:ref", get(get_tree))
        .route("/v1/repos/:repo/search", get(search))
        .route("/v1/repos/:repo/rdf/:ref/*path", get(get_rdf))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn_with_state(
                    state.rate_limit_state.clone(),
                    rate_limit_middleware,
                ))
                .layer(middleware::from_fn_with_state(
                    state.auth_layer.clone(),
                    auth_middleware,
                ))
                .layer(middleware::from_fn(request_id_middleware))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                )
                .layer(
                    CorsLayer::new()
                        .allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap())
                        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE])
                        .allow_headers(Any)
                        .allow_credentials(true)
                ),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    info!("Server listening on {}:{}", host, port);

    // Setup graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        info!("Received shutdown signal");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    Ok(())
}

// Request ID middleware
async fn request_id_middleware(
    mut request: Request,
    next: middleware::Next,
) -> axum::response::Response {
    let request_id = Uuid::new_v4().to_string();
    
    // Add request ID to headers
    request.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );
    
    // Create a tracing span with request ID
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri()
    );
    
    let _enter = span.enter();
    
    next.run(request).await
}

// Authentication middleware
async fn extract_auth(headers: &HeaderMap) -> ApiResult<AuthContext> {
    // TODO: Implement proper JWT verification with OIDC
    // TODO: Add JWKS key rotation and caching
    // TODO: Implement rate limiting per user
    // TODO: Add request timeout and circuit breaker patterns
    // For now, return a mock auth context
    Ok(AuthContext {
        sub: "user@example.com".to_string(),
        roles: vec!["user".to_string()],
    })
}

// Repository endpoints

async fn create_repo(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateRepoRequest>,
) -> ApiResult<Json<CreateRepoResponse>> {
    let auth = extract_auth(&headers).await?;
    
    // Validate repository name
    validate_repo_name(&payload.name)
        .map_err(|e| ApiError::InvalidRequest(format!("Invalid repository name: {}", e)))?;

    // TODO: Implement repository name collision detection with retry logic
    // TODO: Add repository size limits and quotas
    // TODO: Implement audit logging for repository creation

    let repo = state
        .index
        .create_repo(&payload.name, &auth.user_id)
        .await?;

    Ok(Json(CreateRepoResponse {
        id: repo.id,
        name: repo.name,
        created_at: repo.created_at,
    }))
}

async fn list_repos(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<CreateRepoResponse>>> {
    let _auth = extract_auth(&headers).await?;

    let repos = state.index.list_repos().await?;

    let response: Vec<CreateRepoResponse> = repos
        .into_iter()
        .map(|repo| CreateRepoResponse {
            id: repo.id,
            name: repo.name,
            created_at: repo.created_at,
        })
        .collect();

    Ok(Json(response))
}

// Upload endpoints

async fn upload_init(
    State(state): State<AppState>,
    Path(repo): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<UploadInitRequest>,
) -> ApiResult<Json<UploadInitResponse>> {
    let auth = extract_auth(&headers).await?;
    
    // Validate path
    let normalized_path = normalize_path(&payload.path)
        .map_err(|e| ApiError::InvalidRequest(format!("Invalid path: {}", e)))?;
    
    // Validate file size
    validate_file_size(payload.size, None)
        .map_err(|e| ApiError::InvalidRequest(format!("Invalid file size: {}", e)))?;
    
    // Validate content type
    if let Some(ref content_type) = payload.media_type {
        validate_content_type(content_type)
            .map_err(|e| ApiError::InvalidRequest(format!("Invalid content type: {}", e)))?;
    }

    // TODO: Implement virus scanning for uploaded files
    // TODO: Implement upload quotas and rate limiting per user

    // Get repository
    let repo_info = state.index.get_repo_by_name(&repo).await?;

    // Generate SHA256 hash (in real implementation, this would be computed from file content)
    let sha256 = blacklake_core::hash_bytes(&format!("{}{}", payload.path, payload.size).as_bytes());
    let s3_key = blacklake_storage::StorageClient::content_address_key(&sha256);

    // Generate presigned URL
    let upload_url = state
        .storage
        .presign_put(
            &s3_key,
            payload.size,
            &payload.media_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            Duration::hours(1),
        )
        .await?;

    // Store object metadata
    state
        .index
        .upsert_object(
            &sha256,
            payload.size as i64,
            payload.media_type.as_deref(),
            &s3_key,
        )
        .await?;

    Ok(Json(UploadInitResponse {
        upload_url: upload_url.to_string(),
        sha256,
        s3_key,
        expires_at: Utc::now() + Duration::hours(1),
    }))
}

// Commit endpoints

async fn commit(
    State(state): State<AppState>,
    Path(repo): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(payload): Json<CommitRequest>,
) -> ApiResult<Json<CommitResponse>> {
    let auth = extract_auth(&headers).await?;

    // TODO: Add commit message validation and sanitization
    // TODO: Implement atomic commit operations with proper rollback
    // TODO: Add commit size limits and validation
    // TODO: Implement branch protection rules and merge policies

    // Check for RDF emission flag
    let emit_rdf = params.get("emit_rdf")
        .map(|v| v == "true")
        .unwrap_or(false);

    // Get repository
    let repo_info = state.index.get_repo_by_name(&repo).await?;

    // Validate metadata against schema
    for change in &payload.changes {
        // Validate path
        let _normalized_path = normalize_path(&change.path)
            .map_err(|e| ApiError::InvalidRequest(format!("Invalid path '{}': {}", change.path, e)))?;
        
        // Validate metadata
        validate_meta(&change.meta, Some("1.0"))
            .map_err(|e| ApiError::InvalidRequest(format!("Invalid metadata for path '{}': {}", change.path, e)))?;
    }

    // Get current commit for the reference
    let current_commit = state.index.get_ref(repo_info.id, &payload.r#ref).await.ok();

    // Create new commit
    let commit = state
        .index
        .create_commit(
            repo_info.id,
            current_commit.as_ref().map(|r| r.commit_id),
            &auth.sub,
            payload.message.as_deref(),
            payload.expected_parent,
        )
        .await?;

    // Bind entries to commit
    state
        .index
        .bind_entries(commit.id, &payload.changes)
        .await?;

    // Process metadata indexing and RDF generation for each change
    for change in &payload.changes {
        if change.op == ChangeOp::Add || change.op == ChangeOp::Modify {
            // Update metadata index
            let index_row = project_to_index(commit.id, &change.path, &change.meta);
            state
                .index
                .upsert_entry_meta_index(&index_row)
                .await?;

            // Generate RDF if requested
            if emit_rdf {
                if let Ok(canonical_meta) = serde_json::from_value::<CanonicalMeta>(change.meta.clone()) {
                    let subject_iri = generate_subject_iri(&repo, &payload.r#ref, &change.path);
                    
                    // Generate JSON-LD
                    let jsonld = canonical_to_dc_jsonld(&subject_iri, &canonical_meta);
                    let jsonld_text = serde_json::to_string_pretty(&jsonld)?;
                    let jsonld_sha256 = blacklake_core::hash_bytes(jsonld_text.as_bytes());
                    
                    // Store JSON-LD
                    state
                        .index
                        .store_artifact_rdf(
                            commit.id,
                            &change.path,
                            &RdfFormat::Jsonld,
                            &jsonld_text,
                            &jsonld_sha256,
                        )
                        .await?;

                    // Generate and store Turtle
                    if let Ok(turtle_text) = canonical_to_turtle(&subject_iri, &canonical_meta) {
                        let turtle_sha256 = blacklake_core::hash_bytes(turtle_text.as_bytes());
                        
                        state
                            .index
                            .store_artifact_rdf(
                                commit.id,
                                &change.path,
                                &RdfFormat::Turtle,
                                &turtle_text,
                                &turtle_sha256,
                            )
                            .await?;
                    }
                }
            }
        }
    }

    // Update reference
    state
        .index
        .set_ref(
            repo_info.id,
            &payload.r#ref,
            blacklake_core::ReferenceKind::Branch,
            commit.id,
        )
        .await?;

    // Log audit
    state
        .index
        .append_audit_log(
            &auth.sub,
            "commit",
            Some(&repo),
            Some(&payload.r#ref),
            None,
            Some(json!({"changes": payload.changes.len()})),
            Some(json!({"commit_id": commit.id})),
        )
        .await?;

    Ok(Json(CommitResponse {
        commit_id: commit.id,
        parent_id: commit.parent_id,
        created_at: commit.created_at,
    }))
}

// Blob endpoints

async fn get_blob(
    State(state): State<AppState>,
    Path((repo, r#ref, path)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    let _auth = extract_auth(&headers).await?;

    // Get repository
    let repo_info = state.index.get_repo_by_name(&repo).await?;

    // Get reference
    let ref_info = state.index.get_ref(repo_info.id, &r#ref).await?;

    // Get tree entries for the commit
    let entries = state
        .index
        .get_tree_entries(ref_info.commit_id, Some(&path))
        .await?;

    if entries.is_empty() {
        return Err(ApiError::Repo(format!("Path not found: {}", path)));
    }

    let entry = &entries[0];
    if let Some(sha256) = &entry.object_sha256 {
        // Generate presigned URL for download
        let s3_key = blacklake_storage::StorageClient::content_address_key(sha256);
        let download_url = state
            .storage
            .presign_get(&s3_key, Duration::hours(1))
            .await?;

        // Log audit
        state
            .index
            .append_audit_log(
                &_auth.sub,
                "blob_access",
                Some(&repo),
                Some(&r#ref),
                Some(&path),
                None,
                Some(json!({"sha256": sha256})),
            )
            .await?;

        Ok(Json(json!({
            "download_url": download_url.to_string(),
            "sha256": sha256,
            "path": path,
            "meta": entry.meta
        })))
    } else {
        Err(ApiError::Repo(format!("No object found for path: {}", path)))
    }
}

// Tree endpoints

async fn get_tree(
    State(state): State<AppState>,
    Path((repo, r#ref)): Path<(String, String)>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> ApiResult<Json<TreeResponse>> {
    let _auth = extract_auth(&headers).await?;

    // Get repository
    let repo_info = state.index.get_repo_by_name(&repo).await?;

    // Get reference
    let ref_info = state.index.get_ref(repo_info.id, &r#ref).await?;

    // Get path prefix from query params
    let path_prefix = params.get("p");

    // Get tree entries
    let entries = state
        .index
        .get_tree_entries(ref_info.commit_id, path_prefix.map(|s| s.as_str()))
        .await?;

    let tree_entries: Vec<TreeEntry> = entries
        .into_iter()
        .map(|entry| TreeEntry {
            path: entry.path,
            is_dir: entry.is_dir,
            size: None, // TODO: get from object metadata
            media_type: None, // TODO: get from object metadata
            meta: entry.meta,
        })
        .collect();

    Ok(Json(TreeResponse {
        entries: tree_entries,
    }))
}

// Search endpoints

async fn search(
    State(state): State<AppState>,
    Path(repo): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> ApiResult<Json<SearchResponse>> {
    let _auth = extract_auth(&headers).await?;

    // Get repository
    let repo_info = state.index.get_repo_by_name(&repo).await?;

    // Parse search parameters
    let mut filters = HashMap::new();
    for (key, value) in params {
        if key != "sort" && key != "limit" && key != "offset" {
            filters.insert(key, serde_json::Value::String(value));
        }
    }

    let sort = params.get("sort").map(|s| s.as_str());
    let limit = params.get("limit").and_then(|s| s.parse().ok());
    let offset = params.get("offset").and_then(|s| s.parse().ok());

    // Search entries
    let (entries, total) = state
        .index
        .search_entries(repo_info.id, &filters, sort, limit, offset)
        .await?;

    // TODO: Convert entries to SearchEntry format
    let search_entries = vec![];

    Ok(Json(SearchResponse {
        entries: search_entries,
        total,
    }))
}

// RDF endpoints

async fn get_rdf(
    State(state): State<AppState>,
    Path((repo, r#ref, path)): Path<(String, String, String)>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> ApiResult<axum::response::Response> {
    let _auth = extract_auth(&headers).await?;

    // Get format parameter (default to turtle)
    let format_str = params.get("format").map(|s| s.as_str()).unwrap_or("turtle");
    let format = match format_str {
        "turtle" => RdfFormat::Turtle,
        "jsonld" => RdfFormat::Jsonld,
        _ => return Err(ApiError::InvalidRequest("Invalid format. Use 'turtle' or 'jsonld'".to_string())),
    };

    // Get repository
    let repo_info = state.index.get_repo_by_name(&repo).await?;

    // Get reference
    let ref_info = state.index.get_ref(repo_info.id, &r#ref).await?;

    // Try to get stored RDF first
    if let Some(rdf) = state
        .index
        .get_artifact_rdf(ref_info.commit_id, &path, &format)
        .await?
    {
        let content_type = match format {
            RdfFormat::Turtle => "text/turtle",
            RdfFormat::Jsonld => "application/ld+json",
        };

        return Ok(axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", content_type)
            .body(rdf.graph.into())
            .unwrap());
    }

    // Check if auto_rdf feature is enabled
    let features = state.index.get_repo_features(repo_info.id).await?;
    let auto_rdf = features.get("auto_rdf")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if auto_rdf {
        // Get entry metadata and generate RDF on the fly
        let entries = state
            .index
            .get_tree_entries(ref_info.commit_id, Some(&path))
            .await?;

        if let Some(entry) = entries.first() {
            if let Ok(canonical_meta) = serde_json::from_value::<CanonicalMeta>(entry.meta.clone()) {
                let subject_iri = generate_subject_iri(&repo, &r#ref, &path);
                
                let rdf_text = match format {
                    RdfFormat::Turtle => canonical_to_turtle(&subject_iri, &canonical_meta)?,
                    RdfFormat::Jsonld => {
                        let jsonld = canonical_to_dc_jsonld(&subject_iri, &canonical_meta);
                        serde_json::to_string_pretty(&jsonld)?
                    }
                };

                let rdf_sha256 = blacklake_core::hash_bytes(rdf_text.as_bytes());
                
                // Store the generated RDF
                state
                    .index
                    .store_artifact_rdf(
                        ref_info.commit_id,
                        &path,
                        &format,
                        &rdf_text,
                        &rdf_sha256,
                    )
                    .await?;

                let content_type = match format {
                    RdfFormat::Turtle => "text/turtle",
                    RdfFormat::Jsonld => "application/ld+json",
                };

                return Ok(axum::response::Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", content_type)
                    .body(rdf_text.into())
                    .unwrap());
            }
        }
    }

    Err(ApiError::Repo(format!("RDF not found for path: {}", path)))
}

// Helper functions

fn validate_metadata(meta: &Value, schema: &MetadataSchema) -> bool {
    // TODO: Implement proper JSON Schema validation
    // For now, just check if it's an object
    meta.is_object()
}
