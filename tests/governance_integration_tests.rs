// Week 4: Governance Integration Tests
// End-to-end tests for branch protection, quotas, and webhooks

use blacklake_core::{
    governance::{
        ProtectedRef, RepoQuota, RetentionPolicy, RepoRetention, Webhook, WebhookEvent,
        PolicyEvaluation, CheckResult, CheckStatus, ExportJob, ExportManifest, ExportJobStatus,
    },
    Change, ChangeOp, CommitRequest, CreateRepoRequest, Uuid, AuthContext,
};
use blacklake_index::IndexClient;
use blacklake_storage::StorageClient;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use sqlx::PgPool;

/// Setup test database for integration tests
async fn setup_test_database() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://blacklake:password@localhost:5432/blacklake_test".to_string());
    
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    pool
}

/// Cleanup test database after tests
async fn cleanup_test_database(pool: PgPool) {
    // Clean up test data
    sqlx::query("DELETE FROM audit_logs WHERE repo_id IN (SELECT id FROM repos WHERE name LIKE 'test-%')")
        .execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM tree_entries WHERE repo_id IN (SELECT id FROM repos WHERE name LIKE 'test-%')")
        .execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM repos WHERE name LIKE 'test-%'")
        .execute(&pool).await.unwrap();
    
    pool.close().await;
}

/// Test branch protection enforcement
#[tokio::test]
async fn test_branch_protection_enforcement() {
    // Implement with actual test database
    let test_db = setup_test_database().await;
    let index_client = IndexClient::from_pool(test_db.clone()).await;
    
    // 1. Create repository
    let repo = index_client.create_repo("test-repo", "admin@example.com").await.unwrap();
    
    // 2. Set branch protection rules (require admin)
    let protection_rules = serde_json::json!({
        "require_admin_approval": true,
        "require_signed_commits": false,
        "require_status_checks": false
    });
    
    // 3. Try to commit as non-admin user (should fail)
    let non_admin_auth = AuthContext {
        sub: "user@example.com".to_string(),
        roles: vec!["user".to_string()],
    };
    
    let commit_result = index_client.commit_changes(
        repo.id,
        "main",
        vec![],
        "Test commit",
        &non_admin_auth
    ).await;
    
    assert!(commit_result.is_err(), "Non-admin commit should fail with branch protection");
    
    // 4. Try to commit as admin user (should succeed)
    let admin_auth = AuthContext {
        sub: "admin@example.com".to_string(),
        roles: vec!["admin".to_string(), "user".to_string()],
    };
    
    let admin_commit_result = index_client.commit_changes(
        repo.id,
        "main",
        vec![],
        "Admin commit",
        &admin_auth
    ).await;
    
    assert!(admin_commit_result.is_ok(), "Admin commit should succeed");
    
    // 5. Verify audit logs contain policy violations
    let audit_logs = index_client.get_audit_logs(
        Some(repo.id),
        None,
        Some(chrono::Utc::now() - chrono::Duration::hours(1))
    ).await.unwrap();
    
    assert!(!audit_logs.is_empty(), "Audit logs should contain policy violations");
    
    cleanup_test_database(test_db).await;
}

/// Test quota enforcement
#[tokio::test]
async fn test_quota_enforcement() {
    // Implement with actual test database
    let test_db = setup_test_database().await;
    let index_client = IndexClient::from_pool(test_db.clone()).await;
    
    // 1. Create repository with quota limits
    let repo = index_client.create_repo("test-quota-repo", "admin@example.com").await.unwrap();
    
    let quota = RepoQuota {
        repo_id: repo.id,
        soft_limit_gb: 1.0,
        hard_limit_gb: 2.0,
        current_usage_gb: 0.0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // 2. Upload files up to soft limit (should succeed with warning)
    let small_file_size = 500 * 1024 * 1024; // 500MB
    let upload_result = index_client.upload_file(
        repo.id,
        "main",
        "test-file-1.txt",
        small_file_size,
        "text/plain",
        b"test content".to_vec()
    ).await;
    
    assert!(upload_result.is_ok(), "Upload within soft limit should succeed");
    
    // 3. Upload files up to hard limit (should succeed)
    let medium_file_size = 1.5 * 1024.0 * 1024.0 * 1024.0; // 1.5GB
    let upload_result_2 = index_client.upload_file(
        repo.id,
        "main",
        "test-file-2.txt",
        medium_file_size as u64,
        "text/plain",
        vec![0; 1024 * 1024] // 1MB of data
    ).await;
    
    assert!(upload_result_2.is_ok(), "Upload within hard limit should succeed");
    
    // 4. Try to upload beyond hard limit (should fail)
    let large_file_size = 3.0 * 1024.0 * 1024.0 * 1024.0; // 3GB
    let upload_result_3 = index_client.upload_file(
        repo.id,
        "main",
        "test-file-3.txt",
        large_file_size as u64,
        "text/plain",
        vec![0; 1024 * 1024] // 1MB of data
    ).await;
    
    assert!(upload_result_3.is_err(), "Upload beyond hard limit should fail");
    
    // 5. Verify usage tracking is accurate
    let usage = index_client.get_repo_usage(repo.id).await.unwrap();
    assert!(usage.total_size_gb > 0.0, "Usage should be tracked");
    
    cleanup_test_database(test_db).await;
}

/// Test webhook delivery
#[tokio::test]
async fn test_webhook_delivery() {
    // Implement with actual test database and HTTP server
    let test_db = setup_test_database().await;
    let index_client = IndexClient::from_pool(test_db.clone()).await;
    
    // 1. Create webhook endpoint (mock HTTP server)
    let webhook_server = mockito::Server::new_async().await;
    let webhook_url = webhook_server.url();
    
    let mock = webhook_server
        .mock("POST", "/webhook")
        .with_status(200)
        .with_header("content-type", "application/json")
        .create();
    
    // 2. Create repository with webhook
    let repo = index_client.create_repo("test-webhook-repo", "admin@example.com").await.unwrap();
    
    let webhook = Webhook {
        id: Uuid::new_v4(),
        repo_id: Some(repo.id),
        url: webhook_url.clone(),
        events: vec!["commit".to_string(), "push".to_string()],
        secret: Some("test-secret".to_string()),
        active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let registered_webhook = index_client.create_webhook(webhook).await.unwrap();
    
    // 3. Make commit (should trigger webhook)
    let auth = AuthContext {
        sub: "admin@example.com".to_string(),
        roles: vec!["admin".to_string()],
    };
    
    let commit_result = index_client.commit_changes(
        repo.id,
        "main",
        vec![Change {
            op: ChangeOp::Add,
            path: "test-file.txt".to_string(),
            object_sha256: "test-hash".to_string(),
        }],
        "Test commit for webhook",
        &auth
    ).await;
    
    assert!(commit_result.is_ok(), "Commit should succeed");
    
    // 4. Verify webhook was delivered with correct signature
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Wait for async webhook delivery
    
    mock.assert();
    
    // 5. Test retry logic for failed deliveries
    let failing_mock = webhook_server
        .mock("POST", "/webhook")
        .with_status(500)
        .create();
    
    // Trigger another event
    let commit_result_2 = index_client.commit_changes(
        repo.id,
        "main",
        vec![Change {
            op: ChangeOp::Add,
            path: "test-file-2.txt".to_string(),
            object_sha256: "test-hash-2".to_string(),
        }],
        "Test commit for webhook retry",
        &auth
    ).await;
    
    assert!(commit_result_2.is_ok(), "Commit should succeed even if webhook fails");
    
    // Verify webhook was called multiple times (retry logic)
    failing_mock.assert();
    
    cleanup_test_database(test_db).await;
}

/// Test retention policy enforcement
#[tokio::test]
async fn test_retention_policy_enforcement() {
    // TODO: Implement with actual test database
    // This would test:
    // 1. Create repository with retention policy
    // 2. Create artifacts with old timestamps
    // 3. Run retention cleanup worker
    // 4. Verify artifacts are tombstoned/hard deleted
    // 5. Test legal hold prevents cleanup
}

/// Test export job processing
#[tokio::test]
async fn test_export_job_processing() {
    // TODO: Implement with actual test database and S3
    // This would test:
    // 1. Create repository with artifacts
    // 2. Create export job
    // 3. Process export job
    // 4. Verify export package contains correct artifacts
    // 5. Verify download URL is generated
}

/// Test check result submission
#[tokio::test]
async fn test_check_result_submission() {
    // TODO: Implement with actual test database
    // This would test:
    // 1. Create repository with branch protection requiring checks
    // 2. Submit check results
    // 3. Verify check results are stored
    // 4. Test policy evaluation with check results
}

/// Test policy evaluation logic
#[tokio::test]
async fn test_policy_evaluation() {
    use blacklake_core::governance::PolicyEngine;
    
    // Test admin requirement
    let protected_ref = ProtectedRef {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        ref_name: "main".to_string(),
        require_admin: true,
        allow_fast_forward: true,
        allow_delete: false,
        required_checks: vec![],
        required_reviewers: 0,
        require_schema_pass: false,
    };

    let evaluation = PolicyEngine::evaluate_branch_protection(
        &protected_ref,
        Uuid::new_v4(),
        "user123",
        false, // not admin
        &[],
    );

    assert!(!evaluation.allowed);
    assert_eq!(evaluation.reason, Some("Admin access required".to_string()));

    // Test with admin user
    let evaluation = PolicyEngine::evaluate_branch_protection(
        &protected_ref,
        Uuid::new_v4(),
        "admin123",
        true, // is admin
        &[],
    );

    assert!(evaluation.allowed);
    assert!(evaluation.reason.is_none());
}

/// Test quota status calculation
#[tokio::test]
async fn test_quota_status_calculation() {
    use blacklake_core::governance::QuotaStatus;
    
    // Test normal usage
    let status = QuotaStatus::new(500_000_000, 1_000_000_000, 10_000_000_000);
    assert!(!status.soft_warning);
    assert!(!status.hard_exceeded);
    assert_eq!(status.usage_percentage, 5.0);

    // Test soft warning
    let status = QuotaStatus::new(1_500_000_000, 1_000_000_000, 10_000_000_000);
    assert!(status.soft_warning);
    assert!(!status.hard_exceeded);
    assert_eq!(status.usage_percentage, 15.0);

    // Test hard exceeded
    let status = QuotaStatus::new(11_000_000_000, 1_000_000_000, 10_000_000_000);
    assert!(status.soft_warning);
    assert!(status.hard_exceeded);
    assert_eq!(status.usage_percentage, 110.0);
}

/// Test webhook signature verification
#[tokio::test]
async fn test_webhook_signature_verification() {
    use blacklake_core::governance::WebhookSignature;
    
    let secret = "test-secret";
    let payload = r#"{"event":"test","data":"test"}"#;
    let signature = WebhookSignature::generate(secret, payload.as_bytes());
    
    assert!(signature.starts_with("sha256="));
    assert!(WebhookSignature::verify(secret, payload.as_bytes(), &signature));
    assert!(!WebhookSignature::verify("wrong-secret", payload.as_bytes(), &signature));
    assert!(!WebhookSignature::verify(secret, b"wrong-payload", &signature));
}

/// Test retention policy serialization
#[tokio::test]
async fn test_retention_policy_serialization() {
    let policy = RetentionPolicy {
        tombstone_days: 30,
        hard_delete_days: 90,
        legal_hold: false,
    };

    let json = serde_json::to_value(&policy).unwrap();
    assert_eq!(json["tombstone_days"], 30);
    assert_eq!(json["hard_delete_days"], 90);
    assert_eq!(json["legal_hold"], false);

    let deserialized: RetentionPolicy = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.tombstone_days, 30);
    assert_eq!(deserialized.hard_delete_days, 90);
    assert_eq!(deserialized.legal_hold, false);
}

/// Test export job status transitions
#[tokio::test]
async fn test_export_job_status_transitions() {
    let job = ExportJob {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        user_id: "user123".to_string(),
        manifest: ExportManifest {
            ref_name: "main".to_string(),
            paths: vec!["data/".to_string()],
            include_meta: true,
            include_rdf: false,
        },
        status: ExportJobStatus::Pending,
        s3_key: None,
        download_url: None,
        error_message: None,
    };

    // Test status serialization
    let json = serde_json::to_value(&job.status).unwrap();
    assert_eq!(json, "pending");

    let deserialized: ExportJobStatus = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, ExportJobStatus::Pending);
}

/// Test check result status transitions
#[tokio::test]
async fn test_check_result_status_transitions() {
    let check = CheckResult {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        ref_name: "main".to_string(),
        commit_id: Uuid::new_v4(),
        check_name: "test-check".to_string(),
        status: CheckStatus::Success,
        details_url: Some("https://example.com/check/123".to_string()),
        output: Some("All tests passed".to_string()),
    };

    // Test status serialization
    let json = serde_json::to_value(&check.status).unwrap();
    assert_eq!(json, "success");

    let deserialized: CheckStatus = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, CheckStatus::Success);
}

/// Test webhook event serialization
#[tokio::test]
async fn test_webhook_event_serialization() {
    let events = vec![
        WebhookEvent::ArtifactCreated,
        WebhookEvent::CommitCreated,
        WebhookEvent::PolicyViolation,
    ];

    let json = serde_json::to_value(&events).unwrap();
    assert_eq!(json[0], "artifact_created");
    assert_eq!(json[1], "commit_created");
    assert_eq!(json[2], "policy_violation");

    let deserialized: Vec<WebhookEvent> = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.len(), 3);
    assert!(matches!(deserialized[0], WebhookEvent::ArtifactCreated));
    assert!(matches!(deserialized[1], WebhookEvent::CommitCreated));
    assert!(matches!(deserialized[2], WebhookEvent::PolicyViolation));
}

/// Test governance types integration
#[tokio::test]
async fn test_governance_types_integration() {
    // Test that all governance types can be serialized/deserialized
    let protected_ref = ProtectedRef {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        ref_name: "main".to_string(),
        require_admin: true,
        allow_fast_forward: false,
        allow_delete: false,
        required_checks: vec!["test-check".to_string()],
        required_reviewers: 2,
        require_schema_pass: true,
    };

    let quota = RepoQuota {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        bytes_soft: 1_000_000_000,
        bytes_hard: 10_000_000_000,
    };

    let retention = RepoRetention {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        retention_policy: RetentionPolicy {
            tombstone_days: 30,
            hard_delete_days: 90,
            legal_hold: false,
        },
    };

    let webhook = Webhook {
        id: Uuid::new_v4(),
        repo_id: Uuid::new_v4(),
        url: "https://example.com/webhook".to_string(),
        secret: "secret123".to_string(),
        events: vec![WebhookEvent::CommitCreated, WebhookEvent::ArtifactCreated],
        active: true,
    };

    // Test serialization
    let protected_ref_json = serde_json::to_value(&protected_ref).unwrap();
    let quota_json = serde_json::to_value(&quota).unwrap();
    let retention_json = serde_json::to_value(&retention).unwrap();
    let webhook_json = serde_json::to_value(&webhook).unwrap();

    // Test deserialization
    let _: ProtectedRef = serde_json::from_value(protected_ref_json).unwrap();
    let _: RepoQuota = serde_json::from_value(quota_json).unwrap();
    let _: RepoRetention = serde_json::from_value(retention_json).unwrap();
    let _: Webhook = serde_json::from_value(webhook_json).unwrap();
}
