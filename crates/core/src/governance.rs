// Week 4: Governance & Safety Rails
// Core types and structures for branch protection, quotas, retention, and webhooks

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Branch protection rules for a repository reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtectedRef {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub ref_name: String,
    pub require_admin: bool,
    pub allow_fast_forward: bool,
    pub allow_delete: bool,
    pub required_checks: Vec<String>,
    pub required_reviewers: u32,
    pub require_schema_pass: bool,
}

/// Repository quota configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepoQuota {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub bytes_soft: u64,
    pub bytes_hard: u64,
}

/// Repository usage tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepoUsage {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub current_bytes: u64,
    pub last_calculated: chrono::DateTime<chrono::Utc>,
}

/// Retention policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetentionPolicy {
    pub tombstone_days: u32,
    pub hard_delete_days: u32,
    pub legal_hold: bool,
}

/// Repository retention configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepoRetention {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub retention_policy: RetentionPolicy,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Webhook {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<WebhookEvent>,
    pub active: bool,
}

/// Webhook event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    ArtifactCreated,
    ArtifactUpdated,
    ArtifactDeleted,
    CommitCreated,
    PolicyViolation,
    Test,
}

impl std::str::FromStr for WebhookEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "artifact_created" => Ok(WebhookEvent::ArtifactCreated),
            "artifact_updated" => Ok(WebhookEvent::ArtifactUpdated),
            "artifact_deleted" => Ok(WebhookEvent::ArtifactDeleted),
            "commit_created" => Ok(WebhookEvent::CommitCreated),
            "policy_violation" => Ok(WebhookEvent::PolicyViolation),
            "test" => Ok(WebhookEvent::Test),
            _ => Err(format!("Unknown webhook event: {}", s)),
        }
    }
}

impl std::fmt::Display for WebhookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookEvent::ArtifactCreated => write!(f, "artifact_created"),
            WebhookEvent::ArtifactUpdated => write!(f, "artifact_updated"),
            WebhookEvent::ArtifactDeleted => write!(f, "artifact_deleted"),
            WebhookEvent::CommitCreated => write!(f, "commit_created"),
            WebhookEvent::PolicyViolation => write!(f, "policy_violation"),
            WebhookEvent::Test => write!(f, "test"),
        }
    }
}

/// Webhook delivery attempt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub response_status: Option<u16>,
    pub response_body: Option<String>,
    pub attempts: u32,
    pub max_attempts: u32,
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Webhook dead letter entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookDead {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub failure_reason: String,
    pub attempts: u32,
}

/// Export job configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportManifest {
    pub ref_name: String,
    pub paths: Vec<String>,
    pub include_meta: bool,
    pub include_rdf: bool,
}

/// Export job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExportJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl std::str::FromStr for ExportJobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(ExportJobStatus::Pending),
            "processing" => Ok(ExportJobStatus::Processing),
            "completed" => Ok(ExportJobStatus::Completed),
            "failed" => Ok(ExportJobStatus::Failed),
            _ => Err(format!("Unknown export job status: {}", s)),
        }
    }
}

impl std::fmt::Display for ExportJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportJobStatus::Pending => write!(f, "pending"),
            ExportJobStatus::Processing => write!(f, "processing"),
            ExportJobStatus::Completed => write!(f, "completed"),
            ExportJobStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Export job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportJob {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub user_id: String,
    pub manifest: ExportManifest,
    pub status: ExportJobStatus,
    pub s3_key: Option<String>,
    pub download_url: Option<String>,
    pub error_message: Option<String>,
}

/// Check result for branch protection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pending,
    Success,
    Failure,
    Error,
}

/// Check result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckResult {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub ref_name: String,
    pub commit_id: Uuid,
    pub check_name: String,
    pub status: CheckStatus,
    pub details_url: Option<String>,
    pub output: Option<String>,
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyEvaluation {
    pub allowed: bool,
    pub reason: Option<String>,
    pub required_checks: Vec<String>,
    pub missing_reviewers: u32,
}

/// Quota status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuotaStatus {
    pub current_bytes: u64,
    pub soft_limit: u64,
    pub hard_limit: u64,
    pub soft_warning: bool,
    pub hard_exceeded: bool,
    pub usage_percentage: f64,
}

impl QuotaStatus {
    pub fn new(current_bytes: u64, soft_limit: u64, hard_limit: u64) -> Self {
        let soft_warning = current_bytes > soft_limit;
        let hard_exceeded = current_bytes > hard_limit;
        let usage_percentage = if hard_limit > 0 {
            (current_bytes as f64 / hard_limit as f64) * 100.0
        } else {
            0.0
        };

        Self {
            current_bytes,
            soft_limit,
            hard_limit,
            soft_warning,
            hard_exceeded,
            usage_percentage,
        }
    }
}

/// Webhook payload for artifact events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactWebhookPayload {
    pub event: WebhookEvent,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub artifact_path: String,
    pub artifact_sha256: String,
    pub commit_id: Uuid,
    pub user_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Webhook payload for policy violation events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyViolationWebhookPayload {
    pub event: WebhookEvent,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub ref_name: String,
    pub policy_name: String,
    pub violation_reason: String,
    pub user_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Webhook payload for commit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitWebhookPayload {
    pub event: WebhookEvent,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub commit_id: Uuid,
    pub ref_name: String,
    pub user_id: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Generic webhook payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookPayload {
    pub event: WebhookEvent,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

/// Webhook delivery record (extended version)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookDeliveryExtended {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event: WebhookEvent,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: u32,
    pub last_attempt_at: Option<u64>,
    pub next_retry_at: Option<u64>,
    pub response_status: Option<u16>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Dead letter webhook record (extended version)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookDeadExtended {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event: WebhookEvent,
    pub payload: serde_json::Value,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub created_at: u64,
    pub moved_at: u64,
}

/// Webhook signature verification
pub struct WebhookSignature;

impl WebhookSignature {
    /// Generate HMAC-SHA256 signature for webhook payload
    pub fn generate(secret: &str, payload: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload);
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    }

    /// Verify HMAC-SHA256 signature for webhook payload
    pub fn verify(secret: &str, payload: &[u8], signature: &str) -> bool {
        let expected = Self::generate(secret, payload);
        expected == signature
    }
}

/// Policy evaluation engine
pub struct PolicyEngine;

impl PolicyEngine {
    /// Evaluate branch protection policy for a commit
    pub fn evaluate_branch_protection(
        protected_ref: &ProtectedRef,
        commit_id: Uuid,
        user_id: &str,
        is_admin: bool,
        check_results: &[CheckResult],
    ) -> PolicyEvaluation {
        let mut allowed = true;
        let mut reason = None;
        let mut required_checks = Vec::new();
        let mut missing_reviewers = 0;

        // Check admin requirement
        if protected_ref.require_admin && !is_admin {
            allowed = false;
            reason = Some("Admin access required".to_string());
        }

        // Check required checks
        for required_check in &protected_ref.required_checks {
            let check_passed = check_results
                .iter()
                .any(|result| {
                    result.check_name == *required_check
                        && result.commit_id == commit_id
                        && matches!(result.status, CheckStatus::Success)
                });

            if !check_passed {
                allowed = false;
                required_checks.push(required_check.clone());
            }
        }

        // Check required reviewers (simplified - in real implementation would check actual reviews)
        if protected_ref.required_reviewers > 0 {
            // This would need to be implemented with actual review tracking
            missing_reviewers = protected_ref.required_reviewers;
        }

        PolicyEvaluation {
            allowed,
            reason,
            required_checks,
            missing_reviewers,
        }
    }

    /// Check if fast-forward is allowed
    pub fn is_fast_forward_allowed(protected_ref: &ProtectedRef) -> bool {
        protected_ref.allow_fast_forward
    }

    /// Check if deletion is allowed
    pub fn is_deletion_allowed(protected_ref: &ProtectedRef) -> bool {
        protected_ref.allow_delete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
