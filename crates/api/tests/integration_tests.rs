// Integration tests for BlackLake API
// Week 6: Testing session management, search, and job functionality

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use blacklake_core::{
    AuthContext, AuthSession, CSRFToken, SearchQuery, SearchResponse,
    IndexEntryJob, SamplingJob, RdfEmissionJob, AntivirusScanJob, ExportJob,
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

// Mock test data
fn create_mock_auth_session() -> AuthSession {
    AuthSession::new(
        "test-user-123".to_string(),
        "test@example.com".to_string(),
        vec!["user".to_string(), "admin".to_string()],
        Some(json!({
            "iss": "http://keycloak:8080/realms/master",
            "aud": "blacklake",
            "exp": 1640995200
        })),
    )
}

fn create_mock_search_query() -> SearchQuery {
    SearchQuery {
        q: "test query".to_string(),
        fq: Some(vec!["file_type:csv".to_string()]),
        sort: Some("creation_dt desc".to_string()),
        limit: Some(20),
        offset: Some(0),
        json_facet: Some(json!({
            "file_type": {
                "type": "terms",
                "field": "file_type",
                "limit": 10
            }
        })),
    }
}

fn create_mock_index_job() -> IndexEntryJob {
    IndexEntryJob {
        repo_id: Uuid::new_v4(),
        repo_name: "test-repo".to_string(),
        ref_name: "main".to_string(),
        path: "data/test.csv".to_string(),
        commit_id: Uuid::new_v4(),
        object_sha256: "abc123def456".to_string(),
        metadata: json!({
            "title": "Test Dataset",
            "description": "A test dataset for integration testing",
            "tags": ["test", "csv", "integration"]
        }),
        operation: blacklake_core::IndexOperation::Index,
    }
}

fn create_mock_sampling_job() -> SamplingJob {
    SamplingJob {
        repo_id: Uuid::new_v4(),
        repo_name: "test-repo".to_string(),
        path: "data/test.csv".to_string(),
        commit_id: Uuid::new_v4(),
        object_sha256: "abc123def456".to_string(),
        file_type: "csv".to_string(),
    }
}

fn create_mock_rdf_job() -> RdfEmissionJob {
    RdfEmissionJob {
        repo_id: Uuid::new_v4(),
        repo_name: "test-repo".to_string(),
        path: "data/test.csv".to_string(),
        commit_id: Uuid::new_v4(),
        metadata: json!({
            "title": "Test Dataset",
            "creator": "Test User",
            "description": "A test dataset"
        }),
        formats: vec!["jsonld".to_string(), "turtle".to_string()],
    }
}

fn create_mock_antivirus_job() -> AntivirusScanJob {
    AntivirusScanJob {
        repo_id: Uuid::new_v4(),
        repo_name: "test-repo".to_string(),
        path: "data/test.csv".to_string(),
        object_sha256: "abc123def456".to_string(),
        file_size: 1024 * 1024, // 1MB
    }
}

fn create_mock_export_job() -> ExportJob {
    ExportJob {
        export_id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        repo_name: "test-repo".to_string(),
        manifest: json!({
            "include_metadata": true,
            "include_rdf": false,
            "filters": {
                "file_types": ["csv", "parquet"],
                "date_range": {
                    "from": "2024-01-01",
                    "to": "2024-12-31"
                }
            }
        }),
        include_metadata: true,
        include_rdf: false,
    }
}

#[tokio::test]
async fn test_session_creation() {
    let auth_session = create_mock_auth_session();
    
    assert_eq!(auth_session.sub, "test-user-123");
    assert_eq!(auth_session.email, "test@example.com");
    assert_eq!(auth_session.roles.len(), 2);
    assert!(auth_session.roles.contains(&"user".to_string()));
    assert!(auth_session.roles.contains(&"admin".to_string()));
    assert!(auth_session.oidc_token_metadata.is_some());
    assert!(!auth_session.csrf_token.as_str().is_empty());
}

#[tokio::test]
async fn test_csrf_token_generation() {
    let token1 = CSRFToken::new();
    let token2 = CSRFToken::new();
    
    assert_ne!(token1.as_str(), token2.as_str());
    assert!(!token1.as_str().is_empty());
    assert!(!token2.as_str().is_empty());
    
    // Tokens should be base64 encoded
    let decoded1 = base64::engine::general_purpose::STANDARD.decode(token1.as_str()).unwrap();
    let decoded2 = base64::engine::general_purpose::STANDARD.decode(token2.as_str()).unwrap();
    
    assert_eq!(decoded1.len(), 32); // 256-bit token
    assert_eq!(decoded2.len(), 32);
}

#[tokio::test]
async fn test_search_query_serialization() {
    let query = create_mock_search_query();
    
    let serialized = serde_json::to_string(&query).unwrap();
    let deserialized: SearchQuery = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(query.q, deserialized.q);
    assert_eq!(query.fq, deserialized.fq);
    assert_eq!(query.sort, deserialized.sort);
    assert_eq!(query.limit, deserialized.limit);
    assert_eq!(query.offset, deserialized.offset);
    assert!(query.json_facet.is_some());
    assert!(deserialized.json_facet.is_some());
}

#[tokio::test]
async fn test_index_job_serialization() {
    let job = create_mock_index_job();
    
    let serialized = serde_json::to_string(&job).unwrap();
    let deserialized: IndexEntryJob = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(job.repo_name, deserialized.repo_name);
    assert_eq!(job.path, deserialized.path);
    assert_eq!(job.object_sha256, deserialized.object_sha256);
    assert_eq!(job.operation, deserialized.operation);
}

#[tokio::test]
async fn test_sampling_job_serialization() {
    let job = create_mock_sampling_job();
    
    let serialized = serde_json::to_string(&job).unwrap();
    let deserialized: SamplingJob = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(job.repo_name, deserialized.repo_name);
    assert_eq!(job.path, deserialized.path);
    assert_eq!(job.file_type, deserialized.file_type);
}

#[tokio::test]
async fn test_rdf_job_serialization() {
    let job = create_mock_rdf_job();
    
    let serialized = serde_json::to_string(&job).unwrap();
    let deserialized: RdfEmissionJob = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(job.repo_name, deserialized.repo_name);
    assert_eq!(job.path, deserialized.path);
    assert_eq!(job.formats.len(), 2);
    assert!(job.formats.contains(&"jsonld".to_string()));
    assert!(job.formats.contains(&"turtle".to_string()));
}

#[tokio::test]
async fn test_antivirus_job_serialization() {
    let job = create_mock_antivirus_job();
    
    let serialized = serde_json::to_string(&job).unwrap();
    let deserialized: AntivirusScanJob = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(job.repo_name, deserialized.repo_name);
    assert_eq!(job.path, deserialized.path);
    assert_eq!(job.file_size, deserialized.file_size);
}

#[tokio::test]
async fn test_export_job_serialization() {
    let job = create_mock_export_job();
    
    let serialized = serde_json::to_string(&job).unwrap();
    let deserialized: ExportJob = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(job.repo_name, deserialized.repo_name);
    assert_eq!(job.include_metadata, deserialized.include_metadata);
    assert_eq!(job.include_rdf, deserialized.include_rdf);
}

#[tokio::test]
async fn test_job_trait_implementations() {
    // Test IndexEntryJob trait
    assert_eq!(IndexEntryJob::job_type(), "index_entry");
    assert_eq!(IndexEntryJob::max_attempts(), 5);
    
    // Test SamplingJob trait
    assert_eq!(SamplingJob::job_type(), "sampling");
    assert_eq!(SamplingJob::max_attempts(), 3);
    
    // Test RdfEmissionJob trait
    assert_eq!(RdfEmissionJob::job_type(), "rdf_emission");
    assert_eq!(RdfEmissionJob::max_attempts(), 3);
    
    // Test AntivirusScanJob trait
    assert_eq!(AntivirusScanJob::job_type(), "antivirus_scan");
    assert_eq!(AntivirusScanJob::max_attempts(), 2);
    
    // Test ExportJob trait
    assert_eq!(ExportJob::job_type(), "export");
    assert_eq!(ExportJob::max_attempts(), 1);
}

#[tokio::test]
async fn test_auth_context_creation() {
    let auth_context = AuthContext {
        sub: "test-user-123".to_string(),
        roles: vec!["user".to_string(), "admin".to_string()],
    };
    
    assert_eq!(auth_context.sub, "test-user-123");
    assert_eq!(auth_context.roles.len(), 2);
    assert!(auth_context.roles.contains(&"user".to_string()));
    assert!(auth_context.roles.contains(&"admin".to_string()));
}

#[tokio::test]
async fn test_search_response_creation() {
    let docs = vec![
        json!({
            "id": "doc1",
            "title": "Test Document 1",
            "file_type": "csv",
            "repo_name": "test-repo"
        }),
        json!({
            "id": "doc2",
            "title": "Test Document 2", 
            "file_type": "parquet",
            "repo_name": "test-repo"
        })
    ];
    
    let facets = json!({
        "file_type": {
            "buckets": [
                {"val": "csv", "count": 1},
                {"val": "parquet", "count": 1}
            ]
        }
    });
    
    let response = SearchResponse {
        docs: docs.clone(),
        num_found: 2,
        facets: Some(facets.clone()),
    };
    
    assert_eq!(response.docs.len(), 2);
    assert_eq!(response.num_found, 2);
    assert!(response.facets.is_some());
    
    let response_facets = response.facets.unwrap();
    assert!(response_facets.get("file_type").is_some());
}

#[tokio::test]
async fn test_job_error_handling() {
    use blacklake_core::jobs::JobError;
    
    let errors = vec![
        JobError::Processing("Test processing error".to_string()),
        JobError::NotFound("Job not found".to_string()),
        JobError::Timeout("Job timeout".to_string()),
        JobError::Storage("Storage error".to_string()),
        JobError::Serialization("Serialization error".to_string()),
    ];
    
    for error in errors {
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());
    }
}

#[tokio::test]
async fn test_session_error_handling() {
    use blacklake_core::sessions::SessionError;
    
    let errors = vec![
        SessionError::StoreError("Redis connection failed".to_string()),
        SessionError::Unauthorized,
        SessionError::CsrfMismatch,
        SessionError::ConfigurationError("Missing SESSION_SECRET".to_string()),
        SessionError::InternalError("Database error".to_string()),
    ];
    
    for error in errors {
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());
    }
}

#[tokio::test]
async fn test_solr_error_handling() {
    use blacklake_core::search::SolrError;
    
    let errors = vec![
        SolrError::Api("Solr server error".to_string()),
        SolrError::InvalidUrl("Invalid URL format".to_string()),
        SolrError::Other(anyhow::anyhow!("Generic error")),
    ];
    
    for error in errors {
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());
    }
}

// Mock integration test for API endpoints
// Note: These would require a full test setup with database, Redis, and Solr
// For now, we'll test the data structures and serialization

#[tokio::test]
async fn test_session_login_request() {
    let request = json!({
        "oidc_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
    });
    
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(!serialized.is_empty());
    assert!(serialized.contains("oidc_token"));
}

#[tokio::test]
async fn test_search_request_parameters() {
    let params = json!({
        "q": "machine learning",
        "fq": ["file_type:csv", "org_lab:ornl"],
        "sort": "creation_dt desc",
        "limit": 20,
        "offset": 0,
        "json.facet": {
            "file_type": {
                "type": "terms",
                "field": "file_type",
                "limit": 10
            }
        }
    });
    
    let serialized = serde_json::to_string(&params).unwrap();
    assert!(!serialized.is_empty());
    assert!(serialized.contains("machine learning"));
    assert!(serialized.contains("file_type:csv"));
}

#[tokio::test]
async fn test_reindex_request() {
    let request = json!({
        "repo_id": "123e4567-e89b-12d3-a456-426614174000",
        "since_commit_id": "123e4567-e89b-12d3-a456-426614174001",
        "full_reindex": true
    });
    
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(!serialized.is_empty());
    assert!(serialized.contains("repo_id"));
    assert!(serialized.contains("full_reindex"));
}
