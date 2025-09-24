// BlackLake Export System
// Week 4: Export jobs with presigned downloads

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use blacklake_core::{
    AuthContext, Uuid,
};
use blacklake_core::governance::{ExportJob, ExportManifest, ExportJobStatus};
use crate::{ApiError, ApiResponse};
use blacklake_index::IndexClient;
use blacklake_storage::StorageClient;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::{
    sync::RwLock,
    time::{interval, sleep},
};
use tracing::{error, info, warn};
use uuid::Uuid as StdUuid;

/// Export job configuration
#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub max_export_size: u64,
    pub export_timeout_seconds: u64,
    pub cleanup_after_days: u32,
    pub s3_bucket: String,
    pub s3_prefix: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            max_export_size: 10 * 1024 * 1024 * 1024, // 10GB
            export_timeout_seconds: 3600, // 1 hour
            cleanup_after_days: 7,
            s3_bucket: "blacklake-exports".to_string(),
            s3_prefix: "exports/".to_string(),
        }
    }
}

/// Export job processor
#[derive(Debug, Clone)]
pub struct ExportProcessor {
    index: Arc<IndexClient>,
    storage: Arc<StorageClient>,
    config: ExportConfig,
}

/// Export job worker
#[derive(Debug)]
pub struct ExportWorker {
    processor: ExportProcessor,
    running: Arc<RwLock<bool>>,
}

impl ExportProcessor {
    /// Create a new export processor
    pub fn new(
        index: Arc<IndexClient>,
        storage: Arc<StorageClient>,
        config: ExportConfig,
    ) -> Self {
        Self {
            index,
            storage,
            config,
        }
    }

    /// Create a new export job
    pub async fn create_export_job(
        &self,
        repo_id: Uuid,
        manifest: ExportManifest,
        user_id: &str,
    ) -> Result<ExportJob, ApiError> {
        let job_id = Uuid::new_v4();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Validate export size
        let estimated_size = self.estimate_export_size(&manifest).await?;
        if estimated_size > self.config.max_export_size {
            return Err(ApiError::PayloadTooLarge(
                format!("Export size {} exceeds maximum allowed size {}", 
                    estimated_size, self.config.max_export_size)
            ));
        }

        // Create export job
        let export_job = ExportJob {
            id: job_id,
            repo_id,
            manifest,
            status: ExportJobStatus::Pending,
            progress: 0,
            total_items: 0,
            processed_items: 0,
            output_size: 0,
            download_url: None,
            error_message: None,
            created_at: now,
            started_at: None,
            completed_at: None,
            expires_at: now + (self.config.cleanup_after_days as u64 * 24 * 60 * 60),
        };

        // Store export job
        self.index.create_export_job(&export_job).await?;

        // Log audit
        self.index.log_audit(
            user_id,
            "export_created",
            None,
            None,
            None,
            Some(&serde_json::json!({
                "export_id": job_id,
                "repo_id": repo_id,
                "manifest": export_job.manifest
            })),
            None,
        ).await?;

        info!("Export job created: {} for repo {}", job_id, repo_id);
        Ok(export_job)
    }

    /// Process an export job
    pub async fn process_export_job(&self, job_id: Uuid) -> Result<(), ApiError> {
        let mut job = self.index.get_export_job(job_id).await?;

        // Update job status to processing
        job.status = ExportJobStatus::Processing;
        job.started_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        self.index.update_export_job_status(&job).await?;

        info!("Processing export job: {}", job_id);

        match self.execute_export(&mut job).await {
            Ok(()) => {
                job.status = ExportJobStatus::Completed;
                job.progress = 100;
                job.completed_at = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );

                // Generate presigned download URL
                let download_url = self.generate_download_url(&job).await?;
                job.download_url = Some(download_url);

                self.index.update_export_job_status(&job).await?;
                info!("Export job completed: {}", job_id);
            }
            Err(e) => {
                job.status = ExportJobStatus::Failed;
                job.error_message = Some(e.to_string());
                job.completed_at = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );

                self.index.update_export_job_status(&job).await?;
                error!("Export job failed: {} - {}", job_id, e);
            }
        }

        Ok(())
    }

    /// Execute the export
    async fn execute_export(&self, job: &mut ExportJob) -> Result<(), ApiError> {
        let export_key = format!("{}{}.tar.gz", self.config.s3_prefix, job.id);

        // Create temporary directory for export
        let temp_dir = std::env::temp_dir().join(format!("export_{}", job.id));
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create temp directory: {}", e)))?;

        // Calculate total items
        job.total_items = self.count_export_items(&job.manifest).await?;
        self.index.update_export_job_status(job).await?;

        // Process each item in the manifest
        for (i, item) in job.manifest.items.iter().enumerate() {
            // Download artifact
            let artifact_path = self.download_artifact(&job.repo_id, &item.ref_name, &item.path, &temp_dir).await?;

            // Add to archive
            self.add_to_archive(&artifact_path, &temp_dir, &item.path).await?;

            // Update progress
            job.processed_items = i + 1;
            job.progress = ((job.processed_items as f64 / job.total_items as f64) * 100.0) as u32;
            self.index.update_export_job_status(job).await?;

            // Small delay to prevent overwhelming the system
            sleep(Duration::from_millis(100)).await;
        }

        // Add metadata file
        self.add_metadata_file(job, &temp_dir).await?;

        // Create final archive
        let archive_path = temp_dir.join("export.tar.gz");
        self.create_archive(&temp_dir, &archive_path).await?;

        // Upload to S3
        let file_size = std::fs::metadata(&archive_path)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to get file size: {}", e)))?
            .len();

        self.storage.upload_file(&archive_path, &export_key).await?;
        job.output_size = file_size;

        // Cleanup temporary directory
        std::fs::remove_dir_all(&temp_dir)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to cleanup temp directory: {}", e)))?;

        Ok(())
    }

    /// Estimate export size
    async fn estimate_export_size(&self, manifest: &ExportManifest) -> Result<u64, ApiError> {
        let mut total_size = 0;

        for item in &manifest.items {
            // Get entry to estimate size
            if let Ok(entries) = self.index.get_entries_by_path(&item.ref_name, &item.path).await {
                for entry in entries {
                    if let Some(object_sha256) = &entry.object_sha256 {
                        if let Ok(object) = self.index.get_object(object_sha256).await {
                            total_size += object.size;
                        }
                    }
                }
            }
        }

        // Add overhead for archive format and metadata
        total_size = (total_size as f64 * 1.1) as u64;
        Ok(total_size)
    }

    /// Count export items
    async fn count_export_items(&self, manifest: &ExportManifest) -> Result<usize, ApiError> {
        Ok(manifest.items.len())
    }

    /// Download artifact to temporary directory
    async fn download_artifact(
        &self,
        repo_id: &Uuid,
        ref_name: &str,
        path: &str,
        temp_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf, ApiError> {
        // Get entry
        let entries = self.index.get_entries_by_path(ref_name, path).await?;
        let entry = entries.first()
            .ok_or_else(|| ApiError::NotFound("Entry not found".to_string()))?;

        let object_sha256 = entry.object_sha256
            .as_ref()
            .ok_or_else(|| ApiError::NotFound("Object not found".to_string()))?;

        // Get object
        let object = self.index.get_object(object_sha256).await?;

        // Download from storage
        let local_path = temp_dir.join(path);
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ApiError::InternalServerError(format!("Failed to create directory: {}", e)))?;
        }

        self.storage.download_file(&object.s3_key, &local_path).await?;
        Ok(local_path)
    }

    /// Add file to archive
    async fn add_to_archive(
        &self,
        file_path: &std::path::Path,
        temp_dir: &std::path::Path,
        archive_path: &str,
    ) -> Result<(), ApiError> {
        // This is a simplified implementation
        // In a real implementation, you would use a proper archive library
        let archive_dir = temp_dir.join("archive");
        std::fs::create_dir_all(&archive_dir)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create archive directory: {}", e)))?;

        let target_path = archive_dir.join(archive_path);
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ApiError::InternalServerError(format!("Failed to create directory: {}", e)))?;
        }

        std::fs::copy(file_path, &target_path)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to copy file: {}", e)))?;

        Ok(())
    }

    /// Add metadata file to export
    async fn add_metadata_file(&self, job: &ExportJob, temp_dir: &std::path::Path) -> Result<(), ApiError> {
        let metadata = serde_json::json!({
            "export_id": job.id,
            "repo_id": job.repo_id,
            "created_at": job.created_at,
            "manifest": job.manifest,
            "total_items": job.total_items,
            "export_version": "1.0"
        });

        let metadata_path = temp_dir.join("archive").join("metadata.json");
        std::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    /// Create final archive
    async fn create_archive(
        &self,
        temp_dir: &std::path::Path,
        archive_path: &std::path::Path,
    ) -> Result<(), ApiError> {
        // This is a simplified implementation
        // In a real implementation, you would use a proper archive library like tar
        let archive_dir = temp_dir.join("archive");
        
        // For now, just copy the directory structure
        // In production, you would create a proper tar.gz archive
        std::fs::create_dir_all(archive_path.parent().unwrap())
            .map_err(|e| ApiError::InternalServerError(format!("Failed to create archive directory: {}", e)))?;

        // This is a placeholder - in reality you'd use tar or zip
        info!("Creating archive from {:?} to {:?}", archive_dir, archive_path);
        
        Ok(())
    }

    /// Generate presigned download URL
    async fn generate_download_url(&self, job: &ExportJob) -> Result<String, ApiError> {
        let export_key = format!("{}{}.tar.gz", self.config.s3_prefix, job.id);
        let download_url = self.storage.generate_presigned_get_url(&export_key, 3600).await?;
        Ok(download_url)
    }

    /// Get export job status
    pub async fn get_export_job(&self, job_id: Uuid) -> Result<ExportJob, ApiError> {
        self.index.get_export_job(job_id).await
    }

    /// Cleanup expired export jobs
    pub async fn cleanup_expired_jobs(&self) -> Result<(), ApiError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Get expired jobs
        let expired_jobs = self.index.get_expired_export_jobs(now).await?;

        for job in expired_jobs {
            // Delete from S3
            let export_key = format!("{}{}.tar.gz", self.config.s3_prefix, job.id);
            if let Err(e) = self.storage.delete_file(&export_key).await {
                warn!("Failed to delete export file {}: {}", export_key, e);
            }

            // Delete from database
            self.index.delete_export_job(job.id).await?;

            info!("Cleaned up expired export job: {}", job.id);
        }

        Ok(())
    }
}

impl ExportWorker {
    /// Create a new export worker
    pub fn new(processor: ExportProcessor) -> Self {
        Self {
            processor,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the export worker
    pub async fn start(&self) -> Result<(), ApiError> {
        let mut running = self.running.write().await;
        if *running {
            return Err(ApiError::BadRequest("Export worker is already running".to_string()));
        }
        *running = true;
        drop(running);

        info!("Starting export worker");

        let worker = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                if !*worker.running.read().await {
                    break;
                }

                interval.tick().await;

                // Process pending export jobs
                if let Err(e) = worker.process_pending_exports().await {
                    error!("Export processing failed: {}", e);
                }

                // Cleanup expired jobs
                if let Err(e) = worker.processor.cleanup_expired_jobs().await {
                    error!("Export cleanup failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Stop the export worker
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Stopping export worker");
    }

    /// Process pending export jobs
    async fn process_pending_exports(&self) -> Result<(), ApiError> {
        let pending_jobs = self.processor.index.get_pending_export_jobs().await?;

        for job in pending_jobs {
            let processor = self.processor.clone();
            tokio::spawn(async move {
                if let Err(e) = processor.process_export_job(job.id).await {
                    error!("Failed to process export job {}: {}", job.id, e);
                }
            });
        }

        Ok(())
    }
}

/// API handlers for export management

/// Create export job
async fn create_export(
    State(processor): State<ExportProcessor>,
    Path(repo): Path<String>,
    auth: AuthContext,
    Json(payload): Json<CreateExportRequest>,
) -> Result<Json<ApiResponse<ExportJob>>, ApiError> {
    // Get repository
    let repo_info = processor.index.get_repo_by_name(&repo).await?;

    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) && !auth.roles.contains(&"user".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    // Create export job
    let export_job = processor.create_export_job(
        repo_info.id,
        payload.manifest,
        &auth.sub,
    ).await?;

    Ok(Json(ApiResponse::success(export_job)))
}

/// Get export job status
async fn get_export_job(
    State(processor): State<ExportProcessor>,
    Path(job_id): Path<Uuid>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<ExportJob>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) && !auth.roles.contains(&"user".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    let export_job = processor.get_export_job(job_id).await?;
    Ok(Json(ApiResponse::success(export_job)))
}

/// Get export job download URL
async fn get_export_download(
    State(processor): State<ExportProcessor>,
    Path(job_id): Path<Uuid>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<ExportDownloadResponse>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) && !auth.roles.contains(&"user".to_string()) {
        return Err(ApiError::Forbidden("User or admin role required".to_string()));
    }

    let export_job = processor.get_export_job(job_id).await?;

    if export_job.status != ExportJobStatus::Completed {
        return Err(ApiError::BadRequest("Export job not completed".to_string()));
    }

    let download_url = export_job.download_url
        .ok_or_else(|| ApiError::InternalServerError("Download URL not available".to_string()))?;

    let response = ExportDownloadResponse {
        download_url,
        expires_at: export_job.expires_at,
        file_size: export_job.output_size,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Create export routes
pub fn create_export_routes() -> Router<ExportProcessor> {
    Router::new()
        .route("/repos/:repo/export", post(create_export))
        .route("/exports/:job_id", get(get_export_job))
        .route("/exports/:job_id/download", get(get_export_download))
}

/// Request/Response types
#[derive(Debug, serde::Deserialize)]
struct CreateExportRequest {
    manifest: ExportManifest,
}

#[derive(Debug, serde::Serialize)]
struct ExportDownloadResponse {
    download_url: String,
    expires_at: u64,
    file_size: u64,
}
