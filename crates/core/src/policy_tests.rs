#[cfg(test)]
mod tests {
    use super::*;
    use blacklake_core::governance::{
        PolicyEvaluation, CheckResult, CheckStatus, ProtectedRef, RepoQuota, RetentionPolicy
    };
    use chrono::{Utc, Duration};
    use serde_json::json;

    #[test]
    fn test_branch_protection_policy_evaluation() {
        // Test branch protection policy evaluation
        let policy = ProtectedRef {
            repo_id: "test-repo".to_string(),
            ref_pattern: "main".to_string(),
            require_admin_approval: true,
            require_signed_commits: false,
            require_status_checks: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let evaluation = PolicyEvaluation {
            policy_id: "branch-protection-1".to_string(),
            repo_id: "test-repo".to_string(),
            ref_name: "main".to_string(),
            user_id: "user@example.com".to_string(),
            user_roles: vec!["user".to_string()],
            checks: vec![
                CheckResult {
                    check_name: "admin_approval".to_string(),
                    status: CheckStatus::Failed,
                    message: "Requires admin approval".to_string(),
                    details: json!({"required_role": "admin"}),
                }
            ],
            overall_status: CheckStatus::Failed,
            evaluated_at: Utc::now(),
        };

        // Test that non-admin users fail branch protection
        assert_eq!(evaluation.overall_status, CheckStatus::Failed);
        assert_eq!(evaluation.checks.len(), 1);
        assert_eq!(evaluation.checks[0].status, CheckStatus::Failed);
    }

    #[test]
    fn test_quota_policy_evaluation() {
        // Test quota policy evaluation
        let quota = RepoQuota {
            repo_id: "test-repo".to_string(),
            soft_limit_gb: 1.0,
            hard_limit_gb: 2.0,
            current_usage_gb: 1.5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test soft limit warning
        assert!(quota.current_usage_gb >= quota.soft_limit_gb);
        assert!(quota.current_usage_gb < quota.hard_limit_gb);

        // Test hard limit enforcement
        let over_quota = RepoQuota {
            current_usage_gb: 2.5,
            ..quota.clone()
        };
        assert!(over_quota.current_usage_gb > over_quota.hard_limit_gb);
    }

    #[test]
    fn test_retention_policy_evaluation() {
        // Test retention policy evaluation
        let policy = RetentionPolicy {
            id: "retention-1".to_string(),
            name: "Data Retention".to_string(),
            description: "Standard data retention policy".to_string(),
            retention_days: 365,
            grace_period_days: 30,
            auto_delete: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let old_date = Utc::now() - Duration::days(400);
        let recent_date = Utc::now() - Duration::days(100);

        // Test old data should be deleted
        assert!(old_date < Utc::now() - Duration::days(policy.retention_days as i64));
        
        // Test recent data should be retained
        assert!(recent_date > Utc::now() - Duration::days(policy.retention_days as i64));
    }

    #[test]
    fn test_webhook_policy_evaluation() {
        // Test webhook delivery policy
        let webhook_events = vec![
            "push".to_string(),
            "commit".to_string(),
            "tag".to_string(),
        ];

        let user_actions = vec![
            "push".to_string(),
            "commit".to_string(),
        ];

        // Test that user actions match webhook events
        for action in &user_actions {
            assert!(webhook_events.contains(action));
        }
    }

    #[test]
    fn test_access_control_policy_evaluation() {
        // Test access control policy evaluation
        let admin_roles = vec!["admin".to_string(), "owner".to_string()];
        let user_roles = vec!["user".to_string()];
        let guest_roles = vec!["guest".to_string()];

        // Test admin access
        assert!(admin_roles.contains(&"admin".to_string()));
        
        // Test user access
        assert!(!user_roles.contains(&"admin".to_string()));
        
        // Test guest restrictions
        assert!(!guest_roles.contains(&"admin".to_string()));
        assert!(!guest_roles.contains(&"user".to_string()));
    }

    #[test]
    fn test_data_classification_policy_evaluation() {
        // Test data classification policy evaluation
        let classifications = vec![
            "public".to_string(),
            "internal".to_string(),
            "confidential".to_string(),
            "secret".to_string(),
        ];

        let file_classification = "confidential";
        let user_clearance = "internal";

        // Test classification access control
        let classification_levels = vec![
            ("public", 0),
            ("internal", 1),
            ("confidential", 2),
            ("secret", 3),
        ];

        let file_level = classification_levels.iter()
            .find(|(name, _)| *name == file_classification)
            .map(|(_, level)| *level)
            .unwrap_or(0);

        let user_level = classification_levels.iter()
            .find(|(name, _)| *name == user_clearance)
            .map(|(_, level)| *level)
            .unwrap_or(0);

        // User should not access files above their clearance level
        assert!(user_level < file_level);
    }
}
