use anyhow::Result;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub retention_days: i32,
    pub legal_hold_override: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalHold {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub reason: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: LegalHoldStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegalHoldStatus {
    Active,
    Released,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceExport {
    pub id: Uuid,
    pub export_type: ExportType,
    pub filters: serde_json::Value,
    pub status: ExportStatus,
    pub file_path: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    AuditLogs,
    RetentionStatus,
    LegalHolds,
    ComplianceReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

pub struct ComplianceService {
    pool: PgPool,
}

impl ComplianceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new retention policy
    pub async fn create_retention_policy(
        &self,
        name: &str,
        description: Option<&str>,
        retention_days: i32,
        legal_hold_override: bool,
    ) -> Result<RetentionPolicy> {
        let policy = sqlx::query_as::<_, RetentionPolicy>(
            "INSERT INTO retention_policy (name, description, retention_days, legal_hold_override)
             VALUES ($1, $2, $3, $4) 
             RETURNING id, name, description, retention_days, legal_hold_override, created_at, updated_at"
        )
        .bind(name)
        .bind(description)
        .bind(retention_days)
        .bind(legal_hold_override)
        .fetch_one(&self.pool)
        .await?;

        info!("Created retention policy: {}", policy.name);
        Ok(policy)
    }

    /// Apply retention policy to an entry
    pub async fn apply_retention_policy(
        &self,
        entry_id: Uuid,
        policy_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE repo_entry SET retention_policy_id = $1, retention_until = NOW() + INTERVAL '1 day' * (SELECT retention_days FROM retention_policy WHERE id = $1) WHERE id = $2"
        )
        .bind(policy_id)
        .bind(entry_id)
        .execute(&self.pool)
        .await?;

        info!("Applied retention policy {} to entry {}", policy_id, entry_id);
        Ok(())
    }

    /// Create a legal hold on an entry
    pub async fn create_legal_hold(
        &self,
        entry_id: Uuid,
        reason: &str,
        created_by: Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<LegalHold> {
        let legal_hold = sqlx::query_as::<_, LegalHold>(
            "INSERT INTO legal_hold (entry_id, reason, created_by, expires_at, status)
             VALUES ($1, $2, $3, $4, $5) 
             RETURNING id, entry_id, reason, created_by, created_at, expires_at, status"
        )
        .bind(entry_id)
        .bind(reason)
        .bind(created_by)
        .bind(expires_at)
        .bind(LegalHoldStatus::Active as i32)
        .fetch_one(&self.pool)
        .await?;

        // Mark entry as under legal hold
        sqlx::query(
            "UPDATE repo_entry SET legal_hold = true WHERE id = $1"
        )
        .bind(entry_id)
        .execute(&self.pool)
        .await?;

        info!("Created legal hold for entry {}: {}", entry_id, reason);
        Ok(legal_hold)
    }

    /// Release a legal hold
    pub async fn release_legal_hold(
        &self,
        legal_hold_id: Uuid,
        released_by: Uuid,
    ) -> Result<()> {
        let legal_hold = sqlx::query_as::<_, LegalHold>(
            "SELECT id, entry_id, reason, created_by, created_at, expires_at, status FROM legal_hold WHERE id = $1"
        )
        .bind(legal_hold_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Legal hold not found"))?;

        if matches!(legal_hold.status, LegalHoldStatus::Released) {
            return Err(anyhow::anyhow!("Legal hold already released"));
        }

        // Update legal hold status
        sqlx::query(
            "UPDATE legal_hold SET status = $1 WHERE id = $2"
        )
        .bind(LegalHoldStatus::Released as i32)
        .bind(legal_hold_id)
        .execute(&self.pool)
        .await?;

        // Check if entry has other active legal holds
        let active_holds = sqlx::query(
            "SELECT COUNT(*) as count FROM legal_hold WHERE entry_id = $1 AND status = $2"
        )
        .bind(legal_hold.entry_id)
        .bind(LegalHoldStatus::Active as i32)
        .fetch_one(&self.pool)
        .await?;

        // If no active holds, remove legal hold flag from entry
        if active_holds.count.unwrap_or(0) == 0 {
            sqlx::query(
                "UPDATE repo_entry SET legal_hold = false WHERE id = $1"
            )
            .bind(legal_hold.entry_id)
            .execute(&self.pool)
            .await?;
        }

        info!("Released legal hold {} for entry {}", legal_hold_id, legal_hold.entry_id);
        Ok(())
    }

    /// Check if an entry can be deleted (not under legal hold and retention not expired)
    pub async fn can_delete_entry(&self, entry_id: Uuid) -> Result<bool> {
        let entry = sqlx::query(
            "SELECT legal_hold, retention_until FROM repo_entry WHERE id = $1"
        )
        .bind(entry_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Entry not found"))?;

        // Cannot delete if under legal hold
        if entry.legal_hold {
            return Ok(false);
        }

        // Cannot delete if retention period not expired
        if let Some(retention_until) = entry.retention_until {
            if retention_until > Utc::now() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Log an audit event
    pub async fn log_audit_event(
        &self,
        user_id: Uuid,
        action: &str,
        resource_type: &str,
        resource_id: Uuid,
        details: serde_json::Value,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO audit_log (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(user_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(details)
        .bind(ip_address)
        .bind(user_agent)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get audit logs with filtering
    pub async fn get_audit_logs(
        &self,
        user_id: Option<Uuid>,
        action: Option<&str>,
        resource_type: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<AuditLog>> {
        let mut query = "SELECT id, user_id, action, resource_type, resource_id, details, ip_address, user_agent, created_at FROM audit_log WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        if let Some(user_id) = user_id {
            param_count += 1;
            query.push_str(&format!(" AND user_id = ${}", param_count));
            params.push(Box::new(user_id));
        }

        if let Some(action) = action {
            param_count += 1;
            query.push_str(&format!(" AND action = ${}", param_count));
            params.push(Box::new(action));
        }

        if let Some(resource_type) = resource_type {
            param_count += 1;
            query.push_str(&format!(" AND resource_type = ${}", param_count));
            params.push(Box::new(resource_type));
        }

        if let Some(start_date) = start_date {
            param_count += 1;
            query.push_str(&format!(" AND created_at >= ${}", param_count));
            params.push(Box::new(start_date));
        }

        if let Some(end_date) = end_date {
            param_count += 1;
            query.push_str(&format!(" AND created_at <= ${}", param_count));
            params.push(Box::new(end_date));
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = limit {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
            params.push(Box::new(limit));
        }

        if let Some(offset) = offset {
            param_count += 1;
            query.push_str(&format!(" OFFSET ${}", param_count));
            params.push(Box::new(offset));
        }

        let logs = sqlx::query_as::<_, AuditLog>(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(logs)
    }

    /// Create a compliance export
    pub async fn create_compliance_export(
        &self,
        export_type: ExportType,
        filters: serde_json::Value,
        created_by: Uuid,
    ) -> Result<ComplianceExport> {
        let export = sqlx::query_as::<_, ComplianceExport>(
            "INSERT INTO compliance_export (export_type, filters, status, created_by) 
             VALUES ($1, $2, $3, $4) 
             RETURNING id, export_type, filters, status, file_path, created_by, created_at, completed_at"
        )
        .bind(export_type as i32)
        .bind(filters)
        .bind(ExportStatus::Pending as i32)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await?;

        info!("Created compliance export: {:?}", export.export_type);
        Ok(export)
    }

    /// Get entries that are eligible for deletion (retention expired, no legal hold)
    pub async fn get_deletable_entries(&self) -> Result<Vec<Uuid>> {
        let entries = sqlx::query(
            "SELECT id FROM repo_entry 
             WHERE legal_hold = false 
             AND retention_until IS NOT NULL 
             AND retention_until <= NOW()"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(entries.into_iter().map(|row| row.id).collect())
    }

    /// Get entries under legal hold
    pub async fn get_legal_hold_entries(&self) -> Result<Vec<Uuid>> {
        let entries = sqlx::query(
            "SELECT DISTINCT entry_id FROM legal_hold WHERE status = $1"
        )
        .bind(LegalHoldStatus::Active as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries.into_iter().map(|row| row.entry_id).collect())
    }

    /// Get retention status summary
    pub async fn get_retention_status_summary(&self) -> Result<serde_json::Value> {
        let stats = sqlx::query(
            "SELECT 
                COUNT(*) as total_entries,
                COUNT(CASE WHEN legal_hold = true THEN 1 END) as legal_hold_entries,
                COUNT(CASE WHEN retention_until IS NOT NULL AND retention_until <= NOW() THEN 1 END) as expired_retention,
                COUNT(CASE WHEN retention_until IS NOT NULL AND retention_until > NOW() THEN 1 END) as active_retention
             FROM repo_entry"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(serde_json::json!({
            "total_entries": stats.total_entries.unwrap_or(0),
            "legal_hold_entries": stats.legal_hold_entries.unwrap_or(0),
            "expired_retention": stats.expired_retention.unwrap_or(0),
            "active_retention": stats.active_retention.unwrap_or(0)
        }))
    }
}
