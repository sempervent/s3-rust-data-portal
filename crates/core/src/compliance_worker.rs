use anyhow::Result;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn};
use crate::compliance_jobs::{ComplianceJobProcessor, ComplianceJobType};

pub struct ComplianceWorker {
    pool: PgPool,
    processor: ComplianceJobProcessor,
    interval: Duration,
}

impl ComplianceWorker {
    pub fn new(pool: PgPool, interval: Duration) -> Self {
        let processor = ComplianceJobProcessor::new(pool.clone());
        Self {
            pool,
            processor,
            interval,
        }
    }

    /// Start the compliance worker
    pub async fn start(&self) -> Result<()> {
        info!("Starting compliance worker with interval: {:?}", self.interval);

        loop {
            if let Err(e) = self.run_cycle().await {
                error!("Compliance worker cycle failed: {}", e);
            }

            sleep(self.interval).await;
        }
    }

    /// Run a single compliance worker cycle
    async fn run_cycle(&self) -> Result<()> {
        info!("Running compliance worker cycle");

        // Process pending jobs
        if let Err(e) = self.processor.process_pending_jobs().await {
            error!("Failed to process pending compliance jobs: {}", e);
        }

        // Schedule periodic jobs
        self.schedule_periodic_jobs().await?;

        Ok(())
    }

    /// Schedule periodic compliance jobs
    async fn schedule_periodic_jobs(&self) -> Result<()> {
        // Check if we need to schedule a retention check (daily)
        let last_retention_check = sqlx::query(
            "SELECT MAX(created_at) as last_check FROM compliance_job WHERE job_type = $1 AND status = $2"
        )
        .bind(ComplianceJobType::RetentionCheck as i32)
        .bind("completed")
        .fetch_optional(&self.pool)
        .await?;

        let should_schedule_retention_check = match last_retention_check {
            Some(Some(last_check)) => {
                let days_since_last_check = chrono::Utc::now().signed_duration_since(last_check).num_days();
                days_since_last_check >= 1
            }
            _ => true, // Never run before
        };

        if should_schedule_retention_check {
            self.processor.create_job(
                ComplianceJobType::RetentionCheck,
                serde_json::json!({}),
            ).await?;
            info!("Scheduled retention check job");
        }

        // Check if we need to schedule a legal hold expiry check (daily)
        let last_legal_hold_check = sqlx::query(
            "SELECT MAX(created_at) as last_check FROM compliance_job WHERE job_type = $1 AND status = $2"
        )
        .bind(ComplianceJobType::LegalHoldExpiry as i32)
        .bind("completed")
        .fetch_optional(&self.pool)
        .await?;

        let should_schedule_legal_hold_check = match last_legal_hold_check {
            Some(Some(last_check)) => {
                let days_since_last_check = chrono::Utc::now().signed_duration_since(last_check).num_days();
                days_since_last_check >= 1
            }
            _ => true, // Never run before
        };

        if should_schedule_legal_hold_check {
            self.processor.create_job(
                ComplianceJobType::LegalHoldExpiry,
                serde_json::json!({}),
            ).await?;
            info!("Scheduled legal hold expiry check job");
        }

        // Check if we need to schedule audit log cleanup (weekly)
        let last_audit_cleanup = sqlx::query(
            "SELECT MAX(created_at) as last_check FROM compliance_job WHERE job_type = $1 AND status = $2"
        )
        .bind(ComplianceJobType::AuditLogCleanup as i32)
        .bind("completed")
        .fetch_optional(&self.pool)
        .await?;

        let should_schedule_audit_cleanup = match last_audit_cleanup {
            Some(Some(last_check)) => {
                let days_since_last_check = chrono::Utc::now().signed_duration_since(last_check).num_days();
                days_since_last_check >= 7
            }
            _ => true, // Never run before
        };

        if should_schedule_audit_cleanup {
            self.processor.create_job(
                ComplianceJobType::AuditLogCleanup,
                serde_json::json!({}),
            ).await?;
            info!("Scheduled audit log cleanup job");
        }

        Ok(())
    }

    /// Get compliance worker statistics
    pub async fn get_stats(&self) -> Result<serde_json::Value> {
        let stats = sqlx::query(
            "SELECT 
                COUNT(*) as total_jobs,
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_jobs,
                COUNT(CASE WHEN status = 'running' THEN 1 END) as running_jobs,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_jobs,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_jobs
             FROM compliance_job"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(serde_json::json!({
            "total_jobs": stats.total_jobs.unwrap_or(0),
            "pending_jobs": stats.pending_jobs.unwrap_or(0),
            "running_jobs": stats.running_jobs.unwrap_or(0),
            "completed_jobs": stats.completed_jobs.unwrap_or(0),
            "failed_jobs": stats.failed_jobs.unwrap_or(0)
        }))
    }

    /// Get recent compliance job history
    pub async fn get_recent_jobs(&self, limit: Option<i64>) -> Result<Vec<serde_json::Value>> {
        let limit = limit.unwrap_or(50);
        
        let jobs = sqlx::query(
            "SELECT id, job_type, status, created_at, started_at, completed_at, error_message, metadata
             FROM compliance_job 
             ORDER BY created_at DESC 
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let job_values: Vec<serde_json::Value> = jobs.into_iter().map(|job| {
            serde_json::json!({
                "id": job.id,
                "job_type": job.job_type,
                "status": job.status,
                "created_at": job.created_at,
                "started_at": job.started_at,
                "completed_at": job.completed_at,
                "error_message": job.error_message,
                "metadata": job.metadata
            })
        }).collect();

        Ok(job_values)
    }
}

/// Start the compliance worker in a background task
pub async fn start_compliance_worker(pool: PgPool) -> Result<()> {
    let worker = ComplianceWorker::new(pool, Duration::from_secs(300)); // Run every 5 minutes
    
    tokio::spawn(async move {
        if let Err(e) = worker.start().await {
            error!("Compliance worker failed: {}", e);
        }
    });

    Ok(())
}
