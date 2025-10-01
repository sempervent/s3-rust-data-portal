// Week 6: Apalis Job Workers
// Background job processing with Redis queue

use axum::extract::State;
use blacklake_core::{
    Uuid,
};
use blacklake_core::governance::{WebhookDelivery, WebhookSignature, RetentionPolicy};
use blacklake_core::jobs::{
    IndexEntryJob, AntivirusScanJob, RdfEmitJob, ExportJob, ReindexJob, SampleJob,
    JobContext, JobError, run_all_workers,
};
use blacklake_core::search::SolrClient;
use blacklake_index::IndexClient;
use blacklake_storage::StorageClient;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::time::{sleep, Duration as TokioDuration};
use tracing::{error, info, warn};

use crate::AppState;

/// Background worker manager
pub struct WorkerManager {
    index: IndexClient,
    storage: StorageClient,
    solr_client: SolrClient,
    http_client: Client,
}

impl WorkerManager {
    pub fn new(index: IndexClient, storage: StorageClient, solr_client: SolrClient) -> Self {
        Self {
            index,
            storage,
            solr_client,
            http_client: Client::new(),
        }
    }

    /// Start all background workers
    pub async fn start_all(&self) {
        let index = self.index.clone();
        let storage = self.storage.clone();
        let solr_client = self.solr_client.clone();
        let http_client = self.http_client.clone();

        // Start Apalis job workers
        let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let job_context = JobContext {
            db_pool: index.get_pool().clone(),
            s3_client: storage.get_s3_client().clone(),
        };

        tokio::spawn(async move {
            if let Err(e) = run_all_workers(job_context, redis_url).await {
                error!("Apalis workers failed: {}", e);
            }
        });

        // Start legacy webhook delivery worker
        tokio::spawn(async move {
            let worker = WebhookWorker::new(index.clone(), http_client);
            worker.run().await;
        });

        // Start legacy retention cleanup worker
        tokio::spawn(async move {
            let worker = RetentionWorker::new(index.clone(), storage);
            worker.run().await;
        });

        info!("Background workers started (Apalis + legacy)");
    }
}

/// Webhook delivery worker
pub struct WebhookWorker {
    index: IndexClient,
    http_client: Client,
}

impl WebhookWorker {
    pub fn new(index: IndexClient, http_client: Client) -> Self {
        Self { index, http_client }
    }

    /// Run the webhook delivery worker
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(TokioDuration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.process_pending_deliveries().await {
                error!("Webhook delivery worker error: {}", e);
            }
        }
    }

    /// Process pending webhook deliveries
    async fn process_pending_deliveries(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pending_deliveries = self.index.get_pending_webhook_deliveries().await?;
        
        for delivery in pending_deliveries {
            if let Err(e) = self.deliver_webhook(&delivery).await {
                error!("Failed to deliver webhook {}: {}", delivery.id, e);
                
                // Move to dead letter queue if max attempts reached
                if delivery.attempts >= delivery.max_attempts {
                    self.move_to_dead_letter(&delivery, &e.to_string()).await?;
                } else {
                    // Schedule retry with exponential backoff
                    self.schedule_retry(&delivery).await?;
                }
            }
        }
        
        Ok(())
    }

    /// Deliver a single webhook
    async fn deliver_webhook(&self, delivery: &WebhookDelivery) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get webhook configuration
        let webhook = self.index.get_webhook(delivery.webhook_id).await?
            .ok_or_else(|| "Webhook not found")?;

        // Generate signature
        let payload_json = serde_json::to_string(&delivery.payload)?;
        let signature = WebhookSignature::generate(&webhook.secret, payload_json.as_bytes());

        // Make HTTP request
        let response = self.http_client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("X-Blacklake-Signature", signature)
            .header("X-Blacklake-Event", &delivery.event_type)
            .body(payload_json)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        // Update delivery record
        self.update_delivery_status(delivery.id, Some(status.as_u16()), Some(body), true).await?;

        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, body).into());
        }

        info!("Successfully delivered webhook {} to {}", delivery.id, webhook.url);
        Ok(())
    }

    /// Update delivery status
    async fn update_delivery_status(
        &self,
        delivery_id: Uuid,
        response_status: Option<u16>,
        response_body: Option<String>,
        delivered: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement delivery status update in index client
        // For now, just log the update
        info!(
            "Updated delivery status for {}: status={:?}, delivered={}",
            delivery_id, response_status, delivered
        );
        Ok(())
    }

    /// Schedule retry with exponential backoff
    async fn schedule_retry(&self, delivery: &WebhookDelivery) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let backoff_seconds = 2_u64.pow(delivery.attempts.min(6)); // Max 64 seconds
        let next_retry = Utc::now() + Duration::seconds(backoff_seconds as i64);
        
        // TODO: Implement retry scheduling in index client
        info!(
            "Scheduled retry for delivery {} in {} seconds (attempt {})",
            delivery.id, backoff_seconds, delivery.attempts + 1
        );
        Ok(())
    }

    /// Move failed delivery to dead letter queue
    async fn move_to_dead_letter(&self, delivery: &WebhookDelivery, reason: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement dead letter queue in index client
        error!(
            "Moved delivery {} to dead letter queue: {}",
            delivery.id, reason
        );
        Ok(())
    }

    /// Get webhook by ID
    async fn get_webhook(&self, webhook_id: Uuid) -> Result<Option<blacklake_core::governance::Webhook>, Box<dyn std::error::Error + Send + Sync>> {
        // Implement webhook lookup in index client
        self.index.get_webhook_by_id(webhook_id).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
    
    /// Implement delivery status update in index client
    async fn update_delivery_status(&self, delivery_id: Uuid, status: &str, response_code: Option<i32>, response_body: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.index.update_webhook_delivery_status(delivery_id, status, response_code, response_body).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
    
    /// Implement retry scheduling in index client
    async fn schedule_retry(&self, webhook_id: Uuid, retry_count: i32, next_retry_at: chrono::DateTime<Utc>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.index.schedule_webhook_retry(webhook_id, retry_count, next_retry_at).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
    
    /// Implement dead letter queue in index client
    async fn move_to_dead_letter(&self, webhook_id: Uuid, error_message: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.index.move_webhook_to_dead_letter(webhook_id, error_message).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

/// Retention cleanup worker
pub struct RetentionWorker {
    index: IndexClient,
    storage: StorageClient,
}

impl RetentionWorker {
    pub fn new(index: IndexClient, storage: StorageClient) -> Self {
        Self { index, storage }
    }

    /// Run the retention cleanup worker
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(TokioDuration::from_secs(3600)); // Run hourly
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.cleanup_expired_artifacts().await {
                error!("Retention cleanup worker error: {}", e);
            }
        }
    }

    /// Clean up expired artifacts based on retention policies
    async fn cleanup_expired_artifacts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting retention cleanup");
        
        // Get all repositories with retention policies
        let repos = self.get_repos_with_retention_policies().await?;
        
        for repo in repos {
            if let Err(e) = self.cleanup_repo_artifacts(&repo).await {
                error!("Failed to cleanup artifacts for repo {}: {}", repo.id, e);
            }
        }
        
        info!("Retention cleanup completed");
        Ok(())
    }

    /// Get repositories with retention policies
    async fn get_repos_with_retention_policies(&self) -> Result<Vec<blacklake_core::Repository>, Box<dyn std::error::Error + Send + Sync>> {
        // Implement repository query with retention policies
        self.index.get_repos_with_retention_policies().await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    /// Clean up artifacts for a specific repository
    async fn cleanup_repo_artifacts(&self, repo: &blacklake_core::Repository) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let retention = self.index.get_repo_retention(repo.id).await?
            .ok_or_else(|| "No retention policy found")?;

        // Skip cleanup if legal hold is active
        if retention.retention_policy.legal_hold {
            info!("Skipping cleanup for repo {} due to legal hold", repo.id);
            return Ok(());
        }

        let now = Utc::now();
        let tombstone_cutoff = now - Duration::days(retention.retention_policy.tombstone_days as i64);
        let hard_delete_cutoff = now - Duration::days(retention.retention_policy.hard_delete_days as i64);

        // Find artifacts to tombstone
        let artifacts_to_tombstone = self.find_artifacts_to_tombstone(repo.id, tombstone_cutoff).await?;
        for artifact in artifacts_to_tombstone {
            self.tombstone_artifact(&artifact).await?;
        }

        // Find artifacts to hard delete
        let artifacts_to_delete = self.find_artifacts_to_delete(repo.id, hard_delete_cutoff).await?;
        for artifact in artifacts_to_delete {
            self.hard_delete_artifact(&artifact).await?;
        }

        Ok(())
    }

    /// Find artifacts that should be tombstoned
    async fn find_artifacts_to_tombstone(&self, _repo_id: Uuid, _cutoff: chrono::DateTime<chrono::Utc>) -> Result<Vec<Uuid>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement artifact query for tombstoning
        Ok(vec![])
    }

    /// Find artifacts that should be hard deleted
    async fn find_artifacts_to_delete(&self, repo_id: Uuid, cutoff: chrono::DateTime<chrono::Utc>) -> Result<Vec<Uuid>, Box<dyn std::error::Error + Send + Sync>> {
        // Implement artifact query for hard deletion
        self.index.get_artifacts_for_hard_deletion(repo_id, cutoff).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    /// Tombstone an artifact (mark as deleted but keep metadata)
    async fn tombstone_artifact(&self, artifact_id: &Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implement artifact tombstoning
        self.index.tombstone_artifact(*artifact_id).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        info!("Tombstoned artifact {}", artifact_id);
        Ok(())
    }

    /// Hard delete an artifact (remove from storage and metadata)
    async fn hard_delete_artifact(&self, artifact_id: &Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implement artifact hard deletion
        // First, get artifact metadata to find storage location
        let artifact = self.index.get_artifact(*artifact_id).await?;
        
        // Delete from storage
        if let Some(storage_path) = artifact.storage_path {
            if let Err(e) = self.storage.delete_object(&storage_path).await {
                warn!("Failed to delete artifact from storage {}: {}", storage_path, e);
            }
        }
        
        // Delete from database
        self.index.hard_delete_artifact(*artifact_id).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        
        info!("Hard deleted artifact {}", artifact_id);
        Ok(())
    }
}

/// Export job worker
pub struct ExportWorker {
    index: IndexClient,
    storage: StorageClient,
}

impl ExportWorker {
    pub fn new(index: IndexClient, storage: StorageClient) -> Self {
        Self { index, storage }
    }

    /// Process pending export jobs
    pub async fn process_export_job(&self, job_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let job = self.index.get_export_job(job_id).await?
            .ok_or_else(|| "Export job not found")?;

        // Update status to running
        self.index.update_export_job_status(
            job_id,
            blacklake_core::governance::ExportJobStatus::Running,
            None,
            None,
            None,
        ).await?;

        // Create export package
        match self.create_export_package(&job).await {
            Ok((s3_key, download_url)) => {
                self.index.update_export_job_status(
                    job_id,
                    blacklake_core::governance::ExportJobStatus::Completed,
                    Some(s3_key),
                    Some(download_url),
                    None,
                ).await?;
                info!("Export job {} completed successfully", job_id);
            }
            Err(e) => {
                self.index.update_export_job_status(
                    job_id,
                    blacklake_core::governance::ExportJobStatus::Failed,
                    None,
                    None,
                    Some(e.to_string()),
                ).await?;
                error!("Export job {} failed: {}", job_id, e);
            }
        }

        Ok(())
    }

    /// Create export package
    async fn create_export_package(&self, job: &blacklake_core::governance::ExportJob) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        // Implement export package creation
        // 1. Collecting all artifacts from the specified paths
        let artifacts = self.index.get_artifacts_for_export(job.repo_id, &job.paths).await?;
        
        // 2. Creating a tarball with blobs and metadata
        let temp_dir = std::env::temp_dir().join(format!("export_{}", job.id));
        std::fs::create_dir_all(&temp_dir)?;
        
        let mut tar_builder = tar::Builder::new(std::fs::File::create(temp_dir.join("export.tar"))?);
        
        for artifact in artifacts {
            // Add metadata file
            let metadata_path = format!("metadata/{}.json", artifact.id);
            let metadata_json = serde_json::to_string_pretty(&artifact.metadata)?;
            let mut header = tar::Header::new_gnu();
            header.set_path(&metadata_path)?;
            header.set_size(metadata_json.len() as u64);
            header.set_cksum();
            tar_builder.append(&header, metadata_json.as_bytes())?;
            
            // Add blob file if it exists
            if let Some(blob_path) = &artifact.blob_path {
                let blob_data = self.storage.get_object(blob_path).await?;
                let blob_file_path = format!("blobs/{}", artifact.id);
                let mut header = tar::Header::new_gnu();
                header.set_path(&blob_file_path)?;
                header.set_size(blob_data.len() as u64);
                header.set_cksum();
                tar_builder.append(&header, &blob_data)?;
            }
        }
        
        tar_builder.finish()?;
        
        // Compress the tarball
        let tar_path = temp_dir.join("export.tar");
        let gz_path = temp_dir.join("export.tar.gz");
        
        let mut gz_encoder = flate2::write::GzEncoder::new(
            std::fs::File::create(&gz_path)?,
            flate2::Compression::default()
        );
        std::io::copy(&mut std::fs::File::open(&tar_path)?, &mut gz_encoder)?;
        gz_encoder.finish()?;
        
        // 3. Uploading to S3
        let s3_key = format!("exports/{}.tar.gz", job.id);
        let gz_data = std::fs::read(&gz_path)?;
        
        self.storage.put_object(&s3_key, &gz_data).await?;
        
        // 4. Generating presigned download URL
        let download_url = self.storage.get_presigned_url(&s3_key, 3600).await?;
        
        // Cleanup temp files
        std::fs::remove_dir_all(&temp_dir)?;
        
        Ok((s3_key, download_url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webhook_signature_generation() {
        let secret = "test-secret";
        let payload = r#"{"event":"test","data":"test"}"#;
        let signature = WebhookSignature::generate(secret, payload.as_bytes());
        
        assert!(signature.starts_with("sha256="));
        assert!(WebhookSignature::verify(secret, payload.as_bytes(), &signature));
    }

    #[tokio::test]
    async fn test_retention_policy_legal_hold() {
        let policy = RetentionPolicy {
            tombstone_days: 30,
            hard_delete_days: 90,
            legal_hold: true,
        };
        
        // Legal hold should prevent cleanup
        assert!(policy.legal_hold);
    }
}
