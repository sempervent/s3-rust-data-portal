use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use blacklake_core::{AuthContext, EntryMetaIndex, project_to_index};
use blacklake_index::IndexClient;
use blacklake_storage::StorageClient;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequest {
    pub job_type: JobType,
    pub repo_id: Uuid,
    pub commit_id: Uuid,
    pub path: String,
    pub sha256: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    AntivirusScan,
    CsvSample,
    ParquetSample,
    OnnxSniff,
    MetadataIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    pub job_id: Uuid,
    pub job_type: JobType,
    pub status: JobState,
    pub created_at: SystemTime,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub error: Option<String>,
    pub progress: u8, // 0-100
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct JobWorker {
    pub id: Uuid,
    pub job_type: JobType,
    pub status: JobState,
    pub started_at: SystemTime,
}

#[derive(Debug, Clone)]
pub struct JobQueue {
    pub jobs: Arc<RwLock<HashMap<Uuid, JobStatus>>>,
    pub workers: Arc<RwLock<HashMap<Uuid, JobWorker>>>,
    pub sender: mpsc::UnboundedSender<JobRequest>,
}

impl JobQueue {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<JobRequest>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        let queue = Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            workers: Arc::new(RwLock::new(HashMap::new())),
            sender,
        };
        
        (queue, receiver)
    }

    pub async fn submit_job(&self, job_request: JobRequest) -> Result<Uuid, JobError> {
        let job_id = Uuid::new_v4();
        
        let job_status = JobStatus {
            job_id,
            job_type: job_request.job_type.clone(),
            status: JobState::Pending,
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            error: None,
            progress: 0,
            result: None,
        };

        // Store job status
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id, job_status);
        }

        // Send to queue
        self.sender.send(job_request)
            .map_err(|_| JobError::QueueFull)?;

        info!("Job {} submitted for processing", job_id);
        Ok(job_id)
    }

    pub async fn get_job_status(&self, job_id: Uuid) -> Option<JobStatus> {
        let jobs = self.jobs.read().await;
        jobs.get(&job_id).cloned()
    }

    pub async fn get_worker_health(&self) -> WorkerHealth {
        let jobs = self.jobs.read().await;
        let workers = self.workers.read().await;

        let total_jobs = jobs.len();
        let pending_jobs = jobs.values().filter(|j| matches!(j.status, JobState::Pending)).count();
        let running_jobs = jobs.values().filter(|j| matches!(j.status, JobState::Running)).count();
        let completed_jobs = jobs.values().filter(|j| matches!(j.status, JobState::Completed)).count();
        let failed_jobs = jobs.values().filter(|j| matches!(j.status, JobState::Failed)).count();

        WorkerHealth {
            total_jobs,
            pending_jobs,
            running_jobs,
            completed_jobs,
            failed_jobs,
            active_workers: workers.len(),
            workers: workers.values().cloned().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHealth {
    pub total_jobs: usize,
    pub pending_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub active_workers: usize,
    pub workers: Vec<JobWorker>,
}

#[derive(Debug, thiserror::Error)]
pub enum JobError {
    #[error("Job queue is full")]
    QueueFull,
    #[error("Job not found: {0}")]
    JobNotFound(Uuid),
    #[error("Worker error: {0}")]
    WorkerError(String),
}

impl axum::response::IntoResponse for JobError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            JobError::QueueFull => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            JobError::JobNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            JobError::WorkerError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub struct JobProcessor {
    pub index: IndexClient,
    pub storage: StorageClient,
    pub queue: JobQueue,
}

impl JobProcessor {
    pub fn new(index: IndexClient, storage: StorageClient, queue: JobQueue) -> Self {
        Self { index, storage, queue }
    }

    pub async fn start_workers(&self, mut receiver: mpsc::UnboundedReceiver<JobRequest>) {
        let mut worker_handles = Vec::new();

        // Start antivirus worker
        let antivirus_handle = self.start_worker(JobType::AntivirusScan, receiver.clone()).await;
        worker_handles.push(antivirus_handle);

        // Start CSV sampling worker
        let csv_handle = self.start_worker(JobType::CsvSample, receiver.clone()).await;
        worker_handles.push(csv_handle);

        // Start Parquet sampling worker
        let parquet_handle = self.start_worker(JobType::ParquetSample, receiver.clone()).await;
        worker_handles.push(parquet_handle);

        // Start ONNX sniffing worker
        let onnx_handle = self.start_worker(JobType::OnnxSniff, receiver.clone()).await;
        worker_handles.push(onnx_handle);

        // Start metadata indexing worker
        let metadata_handle = self.start_worker(JobType::MetadataIndex, receiver.clone()).await;
        worker_handles.push(metadata_handle);

        // Wait for all workers
        for handle in worker_handles {
            if let Err(e) = handle.await {
                error!("Worker failed: {}", e);
            }
        }
    }

    async fn start_worker(
        &self,
        job_type: JobType,
        mut receiver: mpsc::UnboundedReceiver<JobRequest>,
    ) -> tokio::task::JoinHandle<()> {
        let index = self.index.clone();
        let storage = self.storage.clone();
        let queue = self.queue.clone();

        tokio::spawn(async move {
            let worker_id = Uuid::new_v4();
            
            // Register worker
            {
                let mut workers = queue.workers.write().await;
                workers.insert(worker_id, JobWorker {
                    id: worker_id,
                    job_type: job_type.clone(),
                    status: JobState::Running,
                    started_at: SystemTime::now(),
                });
            }

            info!("Worker {} started for job type {:?}", worker_id, job_type);

            while let Some(job_request) = receiver.recv().await {
                if job_request.job_type != job_type {
                    // Forward to appropriate worker (simplified)
                    continue;
                }

                // Update job status to running
                {
                    let mut jobs = queue.jobs.write().await;
                    if let Some(job) = jobs.get_mut(&job_request.job_id) {
                        job.status = JobState::Running;
                        job.started_at = Some(SystemTime::now());
                        job.progress = 10;
                    }
                }

                // Process job
                let result = match job_type {
                    JobType::AntivirusScan => {
                        Self::process_antivirus_scan(&index, &storage, &job_request).await
                    }
                    JobType::CsvSample => {
                        Self::process_csv_sample(&index, &storage, &job_request).await
                    }
                    JobType::ParquetSample => {
                        Self::process_parquet_sample(&index, &storage, &job_request).await
                    }
                    JobType::OnnxSniff => {
                        Self::process_onnx_sniff(&index, &storage, &job_request).await
                    }
                    JobType::MetadataIndex => {
                        Self::process_metadata_index(&index, &storage, &job_request).await
                    }
                };

                // Update job status
                {
                    let mut jobs = queue.jobs.write().await;
                    if let Some(job) = jobs.get_mut(&job_request.job_id) {
                        match result {
                            Ok(result_data) => {
                                job.status = JobState::Completed;
                                job.completed_at = Some(SystemTime::now());
                                job.progress = 100;
                                job.result = Some(result_data);
                            }
                            Err(e) => {
                                job.status = JobState::Failed;
                                job.completed_at = Some(SystemTime::now());
                                job.error = Some(e.to_string());
                            }
                        }
                    }
                }
            }

            // Unregister worker
            {
                let mut workers = queue.workers.write().await;
                workers.remove(&worker_id);
            }

            info!("Worker {} stopped", worker_id);
        })
    }

    async fn process_antivirus_scan(
        _index: &IndexClient,
        _storage: &StorageClient,
        _job_request: &JobRequest,
    ) -> Result<serde_json::Value, JobError> {
        // TODO: Implement actual antivirus scanning
        // For now, just simulate the process
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        Ok(serde_json::json!({
            "scan_result": "clean",
            "scan_engine": "clamav",
            "scan_time": SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
        }))
    }

    async fn process_csv_sample(
        _index: &IndexClient,
        _storage: &StorageClient,
        _job_request: &JobRequest,
    ) -> Result<serde_json::Value, JobError> {
        // TODO: Implement CSV sampling
        // For now, just simulate the process
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        Ok(serde_json::json!({
            "sample_rows": 10,
            "columns": ["col1", "col2", "col3"],
            "sample_data": [
                ["value1", "value2", "value3"],
                ["value4", "value5", "value6"]
            ]
        }))
    }

    async fn process_parquet_sample(
        _index: &IndexClient,
        _storage: &StorageClient,
        _job_request: &JobRequest,
    ) -> Result<serde_json::Value, JobError> {
        // TODO: Implement Parquet sampling
        // For now, just simulate the process
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        Ok(serde_json::json!({
            "schema": {
                "fields": [
                    {"name": "field1", "type": "string"},
                    {"name": "field2", "type": "int64"}
                ]
            },
            "row_count": 1000,
            "file_size": 1024
        }))
    }

    async fn process_onnx_sniff(
        _index: &IndexClient,
        _storage: &StorageClient,
        _job_request: &JobRequest,
    ) -> Result<serde_json::Value, JobError> {
        // TODO: Implement ONNX model sniffing
        // For now, just simulate the process
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        Ok(serde_json::json!({
            "model_type": "onnx",
            "opset_version": 11,
            "producer_name": "pytorch",
            "producer_version": "1.9.0",
            "input_shapes": [["batch_size", 3, 224, 224]],
            "output_shapes": [["batch_size", 1000]]
        }))
    }

    async fn process_metadata_index(
        index: &IndexClient,
        _storage: &StorageClient,
        job_request: &JobRequest,
    ) -> Result<serde_json::Value, JobError> {
        // Process metadata indexing
        let index_row = project_to_index(job_request.commit_id, &job_request.path, &job_request.metadata);
        
        index.upsert_entry_meta_index(&index_row).await
            .map_err(|e| JobError::WorkerError(format!("Failed to index metadata: {}", e)))?;

        Ok(serde_json::json!({
            "indexed_fields": [
                "creation_dt", "creator", "file_name", "file_type", "file_size",
                "org_lab", "description", "data_source", "data_collection_method", "version"
            ],
            "path": job_request.path
        }))
    }
}

// API handlers
pub async fn submit_job(
    State(queue): State<JobQueue>,
    headers: HeaderMap,
    Json(payload): Json<JobRequest>,
) -> Result<Json<serde_json::Value>, JobError> {
    let _auth = extract_auth(&headers).await?;
    
    let job_id = queue.submit_job(payload).await?;
    
    Ok(Json(serde_json::json!({
        "job_id": job_id,
        "status": "submitted"
    })))
}

pub async fn get_job_status(
    State(queue): State<JobQueue>,
    Path(job_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<JobStatus>, JobError> {
    let _auth = extract_auth(&headers).await?;
    
    let job_status = queue.get_job_status(job_id).await
        .ok_or(JobError::JobNotFound(job_id))?;
    
    Ok(Json(job_status))
}

pub async fn get_worker_health(
    State(queue): State<JobQueue>,
    headers: HeaderMap,
) -> Result<Json<WorkerHealth>, JobError> {
    let _auth = extract_auth(&headers).await?;
    
    let health = queue.get_worker_health().await;
    Ok(Json(health))
}

// Helper function to extract auth (simplified)
async fn extract_auth(_headers: &HeaderMap) -> Result<AuthContext, JobError> {
    // TODO: Implement proper auth extraction
    Ok(AuthContext {
        user_id: "system".to_string(),
        groups: vec![],
        scope: "admin".to_string(),
    })
}
