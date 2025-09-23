use anyhow::Result;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};
use crate::compliance::{ComplianceService, LegalHoldStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceJob {
    pub id: Uuid,
    pub job_type: ComplianceJobType,
    pub status: ComplianceJobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceJobType {
    RetentionCheck,
    LegalHoldExpiry,
    ComplianceExport,
    AuditLogCleanup,
    RetentionPolicyApplication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub struct ComplianceJobProcessor {
    pool: PgPool,
    compliance_service: ComplianceService,
}

impl ComplianceJobProcessor {
    pub fn new(pool: PgPool) -> Self {
        let compliance_service = ComplianceService::new(pool.clone());
        Self {
            pool,
            compliance_service,
        }
    }

    /// Process all pending compliance jobs
    pub async fn process_pending_jobs(&self) -> Result<()> {
        let pending_jobs = sqlx::query_as!(
            ComplianceJob,
            "SELECT id, job_type, status, created_at, started_at, completed_at, error_message, metadata FROM compliance_job WHERE status = $1 ORDER BY created_at ASC",
            ComplianceJobStatus::Pending as _
        )
        .fetch_all(&self.pool)
        .await?;

        for job in pending_jobs {
            if let Err(e) = self.process_job(&job).await {
                error!("Failed to process compliance job {}: {}", job.id, e);
                
                // Mark job as failed
                sqlx::query!(
                    "UPDATE compliance_job SET status = $1, error_message = $2, completed_at = NOW() WHERE id = $3",
                    ComplianceJobStatus::Failed as _,
                    e.to_string(),
                    job.id
                )
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    /// Process a single compliance job
    async fn process_job(&self, job: &ComplianceJob) -> Result<()> {
        info!("Processing compliance job: {:?}", job.job_type);

        // Mark job as running
        sqlx::query!(
            "UPDATE compliance_job SET status = $1, started_at = NOW() WHERE id = $2",
            ComplianceJobStatus::Running as _,
            job.id
        )
        .execute(&self.pool)
        .await?;

        let result = match job.job_type {
            ComplianceJobType::RetentionCheck => self.process_retention_check().await,
            ComplianceJobType::LegalHoldExpiry => self.process_legal_hold_expiry().await,
            ComplianceJobType::ComplianceExport => self.process_compliance_export(&job.metadata).await,
            ComplianceJobType::AuditLogCleanup => self.process_audit_log_cleanup().await,
            ComplianceJobType::RetentionPolicyApplication => self.process_retention_policy_application(&job.metadata).await,
        };

        // Mark job as completed
        sqlx::query!(
            "UPDATE compliance_job SET status = $1, completed_at = NOW() WHERE id = $2",
            ComplianceJobStatus::Completed as _,
            job.id
        )
        .execute(&self.pool)
        .await?;

        result
    }

    /// Check for entries that are eligible for deletion due to expired retention
    async fn process_retention_check(&self) -> Result<()> {
        info!("Processing retention check");

        let deletable_entries = self.compliance_service.get_deletable_entries().await?;
        
        for entry_id in deletable_entries {
            // Check if entry can be deleted
            if self.compliance_service.can_delete_entry(entry_id).await? {
                // Log the deletion eligibility
                self.compliance_service.log_audit_event(
                    Uuid::new_v4(), // System user
                    "retention_expired",
                    "repo_entry",
                    entry_id,
                    serde_json::json!({
                        "action": "eligible_for_deletion",
                        "reason": "retention_period_expired"
                    }),
                    None,
                    None,
                ).await?;

                info!("Entry {} is eligible for deletion due to expired retention", entry_id);
            }
        }

        Ok(())
    }

    /// Check for expired legal holds
    async fn process_legal_hold_expiry(&self) -> Result<()> {
        info!("Processing legal hold expiry check");

        let expired_holds = sqlx::query!(
            "SELECT id, entry_id FROM legal_hold WHERE expires_at IS NOT NULL AND expires_at <= NOW() AND status = $1",
            LegalHoldStatus::Active as _
        )
        .fetch_all(&self.pool)
        .await?;

        for hold in expired_holds {
            // Mark legal hold as expired
            sqlx::query!(
                "UPDATE legal_hold SET status = $1 WHERE id = $2",
                LegalHoldStatus::Expired as _,
                hold.id
            )
            .execute(&self.pool)
            .await?;

            // Check if entry has other active legal holds
            let active_holds = sqlx::query!(
                "SELECT COUNT(*) as count FROM legal_hold WHERE entry_id = $1 AND status = $2",
                hold.entry_id,
                LegalHoldStatus::Active as _
            )
            .fetch_one(&self.pool)
            .await?;

            // If no active holds, remove legal hold flag from entry
            if active_holds.count.unwrap_or(0) == 0 {
                sqlx::query!(
                    "UPDATE repo_entry SET legal_hold = false WHERE id = $1",
                    hold.entry_id
                )
                .execute(&self.pool)
                .await?;
            }

            // Log the expiry
            self.compliance_service.log_audit_event(
                Uuid::new_v4(), // System user
                "legal_hold_expired",
                "legal_hold",
                hold.id,
                serde_json::json!({
                    "entry_id": hold.entry_id,
                    "action": "expired"
                }),
                None,
                None,
            ).await?;

            info!("Legal hold {} expired for entry {}", hold.id, hold.entry_id);
        }

        Ok(())
    }

    /// Process compliance export
    async fn process_compliance_export(&self, metadata: &serde_json::Value) -> Result<()> {
        info!("Processing compliance export");

        let export_id: Uuid = metadata.get("export_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| anyhow::anyhow!("Missing export_id in metadata"))?;

        // Get export details
        let export = sqlx::query!(
            "SELECT export_type, filters, created_by FROM compliance_export WHERE id = $1",
            export_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Export not found"))?;

        // Generate export file based on type
        let file_path = match export.export_type.as_str() {
            "audit_logs" => self.export_audit_logs(&export.filters, export.created_by).await?,
            "retention_status" => self.export_retention_status().await?,
            "legal_holds" => self.export_legal_holds(&export.filters).await?,
            "compliance_report" => self.export_compliance_report(&export.filters).await?,
            _ => return Err(anyhow::anyhow!("Unknown export type: {}", export.export_type)),
        };

        // Update export with file path
        sqlx::query!(
            "UPDATE compliance_export SET file_path = $1, status = $2 WHERE id = $3",
            file_path,
            "completed",
            export_id
        )
        .execute(&self.pool)
        .await?;

        info!("Compliance export {} completed: {}", export_id, file_path);
        Ok(())
    }

    /// Export audit logs to CSV
    async fn export_audit_logs(&self, filters: &serde_json::Value, created_by: Uuid) -> Result<String> {
        let start_date = filters.get("start_date")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        let end_date = filters.get("end_date")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let logs = self.compliance_service.get_audit_logs(
            None,
            None,
            None,
            start_date,
            end_date,
            None,
            None,
        ).await?;

        let file_path = format!("exports/audit_logs_{}.csv", Uuid::new_v4());
        
        // TODO: Implement CSV export
        // For now, just create a placeholder file
        std::fs::create_dir_all("exports")?;
        std::fs::write(&file_path, "audit_logs_export_placeholder")?;

        Ok(file_path)
    }

    /// Export retention status
    async fn export_retention_status(&self) -> Result<String> {
        let summary = self.compliance_service.get_retention_status_summary().await?;
        
        let file_path = format!("exports/retention_status_{}.json", Uuid::new_v4());
        
        std::fs::create_dir_all("exports")?;
        std::fs::write(&file_path, serde_json::to_string_pretty(&summary)?)?;

        Ok(file_path)
    }

    /// Export legal holds
    async fn export_legal_holds(&self, _filters: &serde_json::Value) -> Result<String> {
        let file_path = format!("exports/legal_holds_{}.csv", Uuid::new_v4());
        
        // TODO: Implement legal holds export
        std::fs::create_dir_all("exports")?;
        std::fs::write(&file_path, "legal_holds_export_placeholder")?;

        Ok(file_path)
    }

    /// Export comprehensive compliance report
    async fn export_compliance_report(&self, _filters: &serde_json::Value) -> Result<String> {
        let file_path = format!("exports/compliance_report_{}.pdf", Uuid::new_v4());
        
        // TODO: Implement PDF compliance report generation
        std::fs::create_dir_all("exports")?;
        std::fs::write(&file_path, "compliance_report_placeholder")?;

        Ok(file_path)
    }

    /// Clean up old audit logs
    async fn process_audit_log_cleanup(&self) -> Result<()> {
        info!("Processing audit log cleanup");

        // Delete audit logs older than 7 years
        let cutoff_date = Utc::now() - chrono::Duration::days(2555);
        
        let result = sqlx::query!(
            "DELETE FROM audit_log WHERE created_at < $1",
            cutoff_date
        )
        .execute(&self.pool)
        .await?;

        info!("Cleaned up {} old audit log entries", result.rows_affected());
        Ok(())
    }

    /// Apply retention policy to entries
    async fn process_retention_policy_application(&self, metadata: &serde_json::Value) -> Result<()> {
        info!("Processing retention policy application");

        let policy_id: Uuid = metadata.get("policy_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| anyhow::anyhow!("Missing policy_id in metadata"))?;

        let entry_ids: Vec<Uuid> = metadata.get("entry_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|s| Uuid::parse_str(s).ok())
                    .collect()
            })
            .unwrap_or_default();

        for entry_id in &entry_ids {
            self.compliance_service.apply_retention_policy(entry_id, policy_id).await?;
        }

        info!("Applied retention policy {} to {} entries", policy_id, entry_ids.len());
        Ok(())
    }

    /// Create a new compliance job
    pub async fn create_job(
        &self,
        job_type: ComplianceJobType,
        metadata: serde_json::Value,
    ) -> Result<ComplianceJob> {
        let job = sqlx::query_as!(
            ComplianceJob,
            "INSERT INTO compliance_job (job_type, status, metadata) VALUES ($1, $2, $3) RETURNING id, job_type, status, created_at, started_at, completed_at, error_message, metadata",
            job_type as _,
            ComplianceJobStatus::Pending as _,
            metadata
        )
        .fetch_one(&self.pool)
        .await?;

        info!("Created compliance job: {:?}", job.job_type);
        Ok(job)
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: Uuid) -> Result<Option<ComplianceJob>> {
        let job = sqlx::query_as!(
            ComplianceJob,
            "SELECT id, job_type, status, created_at, started_at, completed_at, error_message, metadata FROM compliance_job WHERE id = $1",
            job_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(job)
    }
}
