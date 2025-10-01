use anyhow::Result;
use serde::{Serialize, Deserialize};
use sqlx::{PgPool, Row};
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

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for ComplianceJob {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(ComplianceJob {
            id: row.get("id"),
            job_type: ComplianceJobType::from_i32(row.get("job_type")).unwrap_or(ComplianceJobType::RetentionCheck),
            status: ComplianceJobStatus::from_i32(row.get("status")).unwrap_or(ComplianceJobStatus::Pending),
            created_at: row.get("created_at"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            error_message: row.get("error_message"),
            metadata: row.get("metadata"),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceJobType {
    RetentionCheck,
    LegalHoldExpiry,
    ComplianceExport,
    AuditLogCleanup,
    RetentionPolicyApplication,
}

impl ComplianceJobType {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ComplianceJobType::RetentionCheck),
            1 => Some(ComplianceJobType::LegalHoldExpiry),
            2 => Some(ComplianceJobType::ComplianceExport),
            3 => Some(ComplianceJobType::AuditLogCleanup),
            4 => Some(ComplianceJobType::RetentionPolicyApplication),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ComplianceJobStatus {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ComplianceJobStatus::Pending),
            1 => Some(ComplianceJobStatus::Running),
            2 => Some(ComplianceJobStatus::Completed),
            3 => Some(ComplianceJobStatus::Failed),
            4 => Some(ComplianceJobStatus::Cancelled),
            _ => None,
        }
    }
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
        let pending_jobs = sqlx::query_as::<_, ComplianceJob>(
            "SELECT id, job_type, status, created_at, started_at, completed_at, error_message, metadata FROM compliance_job WHERE status = $1 ORDER BY created_at ASC"
        )
        .bind(ComplianceJobStatus::Pending as i32)
        .fetch_all(&self.pool)
        .await?;

        for job in pending_jobs {
            if let Err(e) = self.process_job(&job).await {
                error!("Failed to process compliance job {}: {}", job.id, e);
                
                // Mark job as failed
                sqlx::query(
                    "UPDATE compliance_job SET status = $1, error_message = $2, completed_at = NOW() WHERE id = $3"
                )
                .bind(ComplianceJobStatus::Failed as i32)
                .bind(e.to_string())
                .bind(job.id)
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
        sqlx::query(
            "UPDATE compliance_job SET status = $1, started_at = NOW() WHERE id = $2"
        )
        .bind(ComplianceJobStatus::Running as i32)
        .bind(job.id)
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
        sqlx::query(
            "UPDATE compliance_job SET status = $1, completed_at = NOW() WHERE id = $2"
        )
        .bind(ComplianceJobStatus::Completed as i32)
        .bind(job.id)
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

        let expired_holds = sqlx::query(
            "SELECT id, entry_id FROM legal_hold WHERE expires_at IS NOT NULL AND expires_at <= NOW() AND status = $1"
        )
        .bind(LegalHoldStatus::Active as i32)
        .fetch_all(&self.pool)
        .await?;

        for hold in expired_holds {
            // Mark legal hold as expired
            sqlx::query(
                "UPDATE legal_hold SET status = $1 WHERE id = $2"
            )
            .bind(LegalHoldStatus::Expired as i32)
            .bind(hold.get::<Uuid, _>("id"))
            .execute(&self.pool)
            .await?;

            // Check if entry has other active legal holds
            let active_holds = sqlx::query(
                "SELECT COUNT(*) as count FROM legal_hold WHERE entry_id = $1 AND status = $2"
            )
            .bind(hold.get::<Uuid, _>("entry_id"))
            .bind(LegalHoldStatus::Active as i32)
            .fetch_one(&self.pool)
            .await?;

            // If no active holds, remove legal hold flag from entry
            let active_count: i64 = active_holds.get("count");
            if active_count == 0 {
                sqlx::query(
                    "UPDATE repo_entry SET legal_hold = false WHERE id = $1"
                )
                .bind(hold.get::<Uuid, _>("entry_id"))
                .execute(&self.pool)
                .await?;
            }

            // Log the expiry
            self.compliance_service.log_audit_event(
                Uuid::new_v4(), // System user
                "legal_hold_expired",
                "legal_hold",
                hold.get("id"),
                serde_json::json!({
                    "entry_id": hold.get::<Uuid, _>("entry_id"),
                    "action": "expired"
                }),
                None,
                None,
            ).await?;

            info!("Legal hold {} expired for entry {}", hold.get::<Uuid, _>("id"), hold.get::<Uuid, _>("entry_id"));
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
        let export = sqlx::query(
            "SELECT export_type, filters, created_by FROM compliance_export WHERE id = $1"
        )
        .bind(export_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Export not found"))?;

        // Generate export file based on type
        let file_path = match export.get::<i32, _>("export_type") {
            0 => self.export_audit_logs(&export.get::<serde_json::Value, _>("filters"), export.get::<Uuid, _>("created_by")).await?,
            1 => self.export_retention_status().await?,
            2 => self.export_legal_holds(&export.get::<serde_json::Value, _>("filters")).await?,
            3 => self.export_compliance_report(&export.get::<serde_json::Value, _>("filters")).await?,
            _ => return Err(anyhow::anyhow!("Unknown export type: {}", export.get::<i32, _>("export_type"))),
        };

        // Update export with file path
        sqlx::query(
            "UPDATE compliance_export SET file_path = $1, status = $2 WHERE id = $3"
        )
        .bind(&file_path)
        .bind("completed")
        .bind(export_id)
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
        
        // Implement real CSV export
        let audit_logs = self.compliance_service.get_audit_logs(
            None, // start_date
            None, // end_date
            None, // limit
            None  // offset
        ).await?;
        
        std::fs::create_dir_all("exports")?;
        
        // Create CSV content
        let mut csv_content = String::new();
        csv_content.push_str("id,action,user,timestamp,details,ip_address\n");
        
        for log in audit_logs {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{}\n",
                log.id,
                log.action,
                log.user,
                log.timestamp,
                log.details.replace(',', ";"), // Escape commas
                log.ip_address
            ));
        }
        
        std::fs::write(&file_path, csv_content)?;
        
        tracing::info!("Exported {} audit logs to {}", audit_logs.len(), file_path);
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
        
        // Implement real legal holds export
        let legal_holds = self.compliance_service.get_legal_holds().await?;
        
        std::fs::create_dir_all("exports")?;
        
        // Create CSV content
        let mut csv_content = String::new();
        csv_content.push_str("id,name,description,start_date,end_date,status,created_by,created_at\n");
        
        for hold in legal_holds {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                hold.id,
                hold.name,
                hold.description.unwrap_or_default().replace(',', ";"), // Escape commas
                hold.start_date,
                hold.end_date.map(|d| d.to_string()).unwrap_or_default(),
                hold.status,
                hold.created_by,
                hold.created_at
            ));
        }
        
        std::fs::write(&file_path, csv_content)?;
        
        tracing::info!("Exported {} legal holds to {}", legal_holds.len(), file_path);
        Ok(file_path)
    }

    /// Export comprehensive compliance report
    async fn export_compliance_report(&self, _filters: &serde_json::Value) -> Result<String> {
        let file_path = format!("exports/compliance_report_{}.pdf", Uuid::new_v4());
        
        // Implement real PDF compliance report generation
        let retention_summary = self.compliance_service.get_retention_status_summary().await?;
        let legal_holds = self.compliance_service.get_legal_holds().await?;
        let audit_logs = self.compliance_service.get_audit_logs(
            None, None, None, None, None, Some(1000), None
        ).await?;
        
        std::fs::create_dir_all("exports")?;
        
        // Create comprehensive JSON report (PDF generation would require additional dependencies)
        let report = serde_json::json!({
            "report_metadata": {
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "report_type": "compliance_summary",
                "version": "1.0"
            },
            "retention_summary": {
                "total_entries": retention_summary.get("total_entries").unwrap_or(&serde_json::Value::Number(0.into())),
                "expired_entries": retention_summary.get("expired_entries").unwrap_or(&serde_json::Value::Number(0.into())),
                "entries_with_legal_hold": retention_summary.get("legal_hold_entries").unwrap_or(&serde_json::Value::Number(0.into())),
                "entries_ready_for_deletion": retention_summary.get("expired_retention").unwrap_or(&serde_json::Value::Number(0.into()))
            },
            "legal_holds": {
                "total_holds": legal_holds.len(),
                "active_holds": legal_holds.iter().filter(|h| h.status == "active").count(),
                "holds": legal_holds
            },
            "audit_summary": {
                "total_audit_entries": audit_logs.len(),
                "recent_activities": audit_logs.iter().take(10).collect::<Vec<_>>()
            }
        });
        
        // Write JSON report (in production, this would be converted to PDF)
        let json_file_path = file_path.replace(".pdf", ".json");
        std::fs::write(&json_file_path, serde_json::to_string_pretty(&report)?)?;
        
        tracing::info!("Generated compliance report: {}", json_file_path);
        Ok(json_file_path)
    }

    /// Clean up old audit logs
    async fn process_audit_log_cleanup(&self) -> Result<()> {
        info!("Processing audit log cleanup");

        // Delete audit logs older than 7 years
        let cutoff_date = Utc::now() - chrono::Duration::days(2555);
        
        let result = sqlx::query(
            "DELETE FROM audit_log WHERE created_at < $1"
        )
        .bind(cutoff_date)
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
            self.compliance_service.apply_retention_policy(*entry_id, policy_id).await?;
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
        let job = sqlx::query_as::<_, ComplianceJob>(
            "INSERT INTO compliance_job (job_type, status, metadata) VALUES ($1, $2, $3) RETURNING id, job_type, status, created_at, started_at, completed_at, error_message, metadata"
        )
        .bind(job_type as i32)
        .bind(ComplianceJobStatus::Pending as i32)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await?;

        info!("Created compliance job: {:?}", job.job_type);
        Ok(job)
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: Uuid) -> Result<Option<ComplianceJob>> {
        let job = sqlx::query_as::<_, ComplianceJob>(
            "SELECT id, job_type, status, created_at, started_at, completed_at, error_message, metadata FROM compliance_job WHERE id = $1"
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(job)
    }
}
