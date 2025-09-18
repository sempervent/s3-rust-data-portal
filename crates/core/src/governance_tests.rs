// Week 4: Governance & Safety Rails Tests
// Unit tests for governance features

#[cfg(test)]
mod governance_tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_quota_status_calculation() {
        let status = QuotaStatus::new(500_000_000, 1_000_000_000, 10_000_000_000);
        assert_eq!(status.current_bytes, 500_000_000);
        assert_eq!(status.soft_limit, 1_000_000_000);
        assert_eq!(status.hard_limit, 10_000_000_000);
        assert!(!status.soft_warning);
        assert!(!status.hard_exceeded);
        assert_eq!(status.usage_percentage, 5.0);
    }

    #[test]
    fn test_quota_status_soft_warning() {
        let status = QuotaStatus::new(1_500_000_000, 1_000_000_000, 10_000_000_000);
        assert!(status.soft_warning);
        assert!(!status.hard_exceeded);
        assert_eq!(status.usage_percentage, 15.0);
    }

    #[test]
    fn test_quota_status_hard_exceeded() {
        let status = QuotaStatus::new(11_000_000_000, 1_000_000_000, 10_000_000_000);
        assert!(status.soft_warning);
        assert!(status.hard_exceeded);
        assert_eq!(status.usage_percentage, 110.0);
    }

    #[test]
    fn test_webhook_signature_generation() {
        let secret = "test-secret";
        let payload = b"test-payload";
        let signature = WebhookSignature::generate(secret, payload);
        assert!(signature.starts_with("sha256="));
        assert_eq!(signature.len(), 71); // "sha256=" + 64 hex chars
    }

    #[test]
    fn test_webhook_signature_verification() {
        let secret = "test-secret";
        let payload = b"test-payload";
        let signature = WebhookSignature::generate(secret, payload);
        assert!(WebhookSignature::verify(secret, payload, &signature));
        assert!(!WebhookSignature::verify("wrong-secret", payload, &signature));
        assert!(!WebhookSignature::verify(secret, b"wrong-payload", &signature));
    }

    #[test]
    fn test_policy_evaluation_admin_required() {
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
    }

    #[test]
    fn test_policy_evaluation_checks_required() {
        let protected_ref = ProtectedRef {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            ref_name: "main".to_string(),
            require_admin: false,
            allow_fast_forward: true,
            allow_delete: false,
            required_checks: vec!["test-check".to_string()],
            required_reviewers: 0,
            require_schema_pass: false,
        };

        let evaluation = PolicyEngine::evaluate_branch_protection(
            &protected_ref,
            Uuid::new_v4(),
            "user123",
            false,
            &[], // no check results
        );

        assert!(!evaluation.allowed);
        assert_eq!(evaluation.required_checks, vec!["test-check"]);
    }

    #[test]
    fn test_policy_evaluation_success() {
        let protected_ref = ProtectedRef {
            id: Uuid::new_v4(),
            repo_id: Uuid::new_v4(),
            ref_name: "main".to_string(),
            require_admin: false,
            allow_fast_forward: true,
            allow_delete: false,
            required_checks: vec!["test-check".to_string()],
            required_reviewers: 0,
            require_schema_pass: false,
        };

        let commit_id = Uuid::new_v4();
        let check_results = vec![CheckResult {
            id: Uuid::new_v4(),
            repo_id: protected_ref.repo_id,
            ref_name: protected_ref.ref_name.clone(),
            commit_id,
            check_name: "test-check".to_string(),
            status: CheckStatus::Success,
            details_url: None,
            output: None,
        }];

        let evaluation = PolicyEngine::evaluate_branch_protection(
            &protected_ref,
            commit_id,
            "user123",
            false,
            &check_results,
        );

        assert!(evaluation.allowed);
        assert!(evaluation.reason.is_none());
        assert!(evaluation.required_checks.is_empty());
        assert_eq!(evaluation.missing_reviewers, 0);
    }

    #[test]
    fn test_retention_policy_serialization() {
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

    #[test]
    fn test_webhook_event_serialization() {
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

    #[test]
    fn test_export_job_status_serialization() {
        let statuses = vec![
            ExportJobStatus::Pending,
            ExportJobStatus::Running,
            ExportJobStatus::Completed,
            ExportJobStatus::Failed,
        ];

        for status in statuses {
            let json = serde_json::to_value(&status).unwrap();
            let deserialized: ExportJobStatus = serde_json::from_value(json).unwrap();
            assert_eq!(deserialized, status);
        }
    }

    #[test]
    fn test_check_status_serialization() {
        let statuses = vec![
            CheckStatus::Pending,
            CheckStatus::Success,
            CheckStatus::Failure,
            CheckStatus::Error,
        ];

        for status in statuses {
            let json = serde_json::to_value(&status).unwrap();
            let deserialized: CheckStatus = serde_json::from_value(json).unwrap();
            assert_eq!(deserialized, status);
        }
    }
}
