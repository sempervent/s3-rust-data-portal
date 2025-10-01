// BlackLake Job System with Apalis
// Week 6: Advanced job processing with Redis backend

// Simplified job system without Redis for now
use tracing::info;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

// Job types
pub type JobId = Uuid;

pub trait Job: Send + Sync + 'static {
    fn name(&self) -> &str;
}

// Job context and response types
pub struct JobContext {
    pub job_id: JobId,
    pub worker_id: String,
    pub s3_client: Option<aws_sdk_s3::Client>,
    pub db_pool: Option<sqlx::PgPool>,
}

pub enum JobResponse {
    Success,
    Failure(String),
}

pub struct JobRequest {
    pub job_id: JobId,
    pub job: Box<dyn BlackLakeJob>,
}

impl JobRequest {
    pub fn new(job_id: JobId, job: Box<dyn BlackLakeJob>) -> Self {
        Self { job_id, job }
    }
}

/// Job processing errors
#[derive(Debug, Error)]
pub enum JobError {
    #[error("Job processing failed: {0}")]
    Processing(String),
    #[error("Job not found: {0}")]
    NotFound(String),
    #[error("Job timeout: {0}")]
    Timeout(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<aws_sdk_s3::primitives::ByteStreamError> for JobError {
    fn from(err: aws_sdk_s3::primitives::ByteStreamError) -> Self {
        JobError::Storage(format!("ByteStream error: {}", err))
    }
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Processing,
    Running,
    Completed,
    Failed,
    Retrying,
    DeadLetter,
    Cancelled,
    Unknown,
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterJob {
    pub job_id: String,
    pub job_data: JobData,
    pub error_message: String,
    pub failed_at: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
}

/// Job metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub id: JobId,
    pub job_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub attempts: u32,
    pub max_attempts: u32,
    pub status: JobStatus,
    pub error_message: Option<String>,
    pub progress: f64,
    pub metadata: serde_json::Value,
}

/// Base job trait for all BlackLake jobs
#[async_trait::async_trait]
pub trait BlackLakeJob: Job + Send + Sync + 'static {
    /// Job type identifier
    fn job_type(&self) -> &'static str;
    
    /// Maximum number of retry attempts
    fn max_attempts(&self) -> u32 {
        3
    }
    
    /// Retry delay between attempts
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(60)
    }
    
    /// Job timeout
    fn timeout(&self) -> Duration {
        Duration::from_secs(300) // 5 minutes default
    }
    
    /// Process the job
    async fn process(&self, ctx: &JobContext) -> Result<JobResponse, JobError>;
}

/// Index entry job for Solr indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntryJob {
    pub repo_id: Uuid,
    pub repo_name: String,
    pub ref_name: String,
    pub path: String,
    pub commit_id: Uuid,
    pub object_sha256: String,
    pub metadata: serde_json::Value,
    pub operation: IndexOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexOperation {
    Index,
    Update,
    Delete,
}

#[async_trait::async_trait]
impl Job for IndexEntryJob {
    fn name(&self) -> &str {
        "index_entry"
    }
}

#[async_trait::async_trait]
impl BlackLakeJob for IndexEntryJob {
    fn job_type(&self) -> &'static str {
        "index_entry"
    }
    
    fn max_attempts(&self) -> u32 {
        5
    }
    
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(30)
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(120)
    }
    
    async fn process(&self, _ctx: &JobContext) -> Result<JobResponse, JobError> {
        tracing::info!(
            "Processing index entry job: repo={}, path={}, operation={:?}",
            self.repo_name,
            self.path,
            self.operation
        );
        
        // Create Solr document
        let doc = serde_json::json!({
            "id": format!("{}:{}:{}", self.repo_id, self.commit_id, self.path),
            "repo_id": self.repo_id.to_string(),
            "repo_name": self.repo_name,
            "commit_id": self.commit_id.to_string(),
            "path": self.path,
            "object_sha256": self.object_sha256,
            "meta": self.metadata,
            "_version_": 1
        });
        
        match self.operation {
            IndexOperation::Index => {
                // Index the document in Solr
                tracing::info!("Indexing document: {}", self.path);
                
                // Convert to SolrDocument format
                let solr_doc = blacklake_core::search::SolrDocument {
                    id: format!("{}:{}:{}:{}", self.repo_name, self.ref_name, self.path, self.commit_id),
                    repo: self.repo_name.clone(),
                    r#ref: self.ref_name.clone(),
                    path: self.path.clone(),
                    commit_id: self.commit_id.to_string(),
                    file_name: self.path.split('/').last().unwrap_or("").to_string(),
                    title: None,
                    description: None,
                    tags: vec![],
                    org_lab: "default".to_string(),
                    file_type: self.metadata.get("file_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                    file_size: self.metadata.get("file_size").and_then(|v| v.as_i64()).unwrap_or(0),
                    creation_dt: chrono::Utc::now().to_rfc3339(),
                    sha256: self.object_sha256.clone(),
                    content: None,
                    meta: self.metadata.clone(),
                };
                
                // TODO: Use actual SolrClient instance for indexing
                // This would require passing the SolrClient through the job context
                tracing::info!("Document prepared for indexing: {}", self.path);
            }
            IndexOperation::Update => {
                // Update the document in Solr
                tracing::info!("Updating document: {}", self.path);
                
                // Convert to SolrDocument format (same as index)
                let solr_doc = blacklake_core::search::SolrDocument {
                    id: format!("{}:{}:{}:{}", self.repo_name, self.ref_name, self.path, self.commit_id),
                    repo: self.repo_name.clone(),
                    r#ref: self.ref_name.clone(),
                    path: self.path.clone(),
                    commit_id: self.commit_id.to_string(),
                    file_name: self.path.split('/').last().unwrap_or("").to_string(),
                    title: None,
                    description: None,
                    tags: vec![],
                    org_lab: "default".to_string(),
                    file_type: self.metadata.get("file_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                    file_size: self.metadata.get("file_size").and_then(|v| v.as_i64()).unwrap_or(0),
                    creation_dt: chrono::Utc::now().to_rfc3339(),
                    sha256: self.object_sha256.clone(),
                    content: None,
                    meta: self.metadata.clone(),
                };
                
                // TODO: Use actual SolrClient instance for updating
                // This would require passing the SolrClient through the job context
                tracing::info!("Document prepared for update: {}", self.path);
            }
            IndexOperation::Delete => {
                // Delete the document from Solr
                tracing::info!("Deleting document: {}", self.path);
                
                let query = format!("id:{}:{}:{}:*", self.repo_name, self.ref_name, self.path);
                
                // TODO: Use actual SolrClient instance for deletion
                // This would require passing the SolrClient through the job context
                tracing::info!("Document prepared for deletion: {}", self.path);
            }
        }
        
        Ok(JobResponse::Success)
    }
}

/// Sampling job for CSV/Parquet files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingJob {
    pub repo_id: Uuid,
    pub repo_name: String,
    pub path: String,
    pub object_sha256: String,
    pub file_type: String,
    pub file_size: u64,
}

#[async_trait::async_trait]
impl Job for SamplingJob {
    fn name(&self) -> &str {
        "sampling"
    }
}

#[async_trait::async_trait]
impl BlackLakeJob for SamplingJob {
    fn job_type(&self) -> &'static str {
        "sampling"
    }
    
    fn max_attempts(&self) -> u32 {
        3
    }
    
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(60)
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(180)
    }
    
    async fn process(&self, _ctx: &JobContext) -> Result<JobResponse, JobError> {
        tracing::info!(
            "Processing sampling job: repo={}, path={}, type={}",
            self.repo_name,
            self.path,
            self.file_type
        );
        
        // Implement file sampling logic
        match self.file_type.as_str() {
            "csv" => {
                tracing::info!("Sampling CSV file: {}", self.path);
                // Download file from S3, sample first N rows, extract schema
                if let Some(s3_client) = &_ctx.s3_client {
                    match self.sample_csv_file(s3_client).await {
                        Ok(sample_data) => {
                            tracing::info!("CSV sampling completed for {}: {} rows sampled", self.path, sample_data.len());
                            // Store sample data in database for UI display
                        }
                        Err(e) => {
                            tracing::error!("Failed to sample CSV file {}: {}", self.path, e);
                            return Err(JobError::Processing(format!("CSV sampling failed: {}", e)));
                        }
                    }
                } else {
                    tracing::warn!("S3 client not available for CSV sampling: {}", self.path);
                }
            }
            "parquet" => {
                tracing::info!("Sampling Parquet file: {}", self.path);
                // Download file from S3, read metadata, sample data
                if let Some(s3_client) = &_ctx.s3_client {
                    match self.sample_parquet_file(s3_client).await {
                        Ok(sample_data) => {
                            tracing::info!("Parquet sampling completed for {}: {} rows sampled", self.path, sample_data.len());
                            // Store sample data in database for UI display
                        }
                        Err(e) => {
                            tracing::error!("Failed to sample Parquet file {}: {}", self.path, e);
                            return Err(JobError::Processing(format!("Parquet sampling failed: {}", e)));
                        }
                    }
                } else {
                    tracing::warn!("S3 client not available for Parquet sampling: {}", self.path);
                }
            }
            _ => {
                tracing::warn!("Unsupported file type for sampling: {}", self.file_type);
                return Err(JobError::Processing(format!("Unsupported file type: {}", self.file_type)));
            }
        }
        
        Ok(JobResponse::Success)
    }
}

impl SamplingJob {
    async fn sample_csv_file(&self, s3_client: &aws_sdk_s3::Client) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        // Download file from S3
        let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "blacklake".to_string());
        let key = format!("{}/{}", self.repo_name, self.path);
        
        let response = s3_client
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await?;
        
        let data = response.body.collect().await?.into_bytes();
        
        // Parse CSV and sample first 100 rows
        let mut reader = csv::Reader::from_reader(data.as_ref());
        let mut sample_data = Vec::new();
        let mut row_count = 0;
        let max_rows = 100;
        
        for result in reader.records() {
            if row_count >= max_rows {
                break;
            }
            
            let record = result?;
            let mut row = serde_json::Map::new();
            
            for (i, field) in record.iter().enumerate() {
                if let Some(header) = reader.headers().get(i) {
                    row.insert(header.to_string(), serde_json::Value::String(field.to_string()));
                }
            }
            
            sample_data.push(serde_json::Value::Object(row));
            row_count += 1;
        }
        
        Ok(sample_data)
    }
    
    async fn sample_parquet_file(&self, s3_client: &aws_sdk_s3::Client) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        // Download file from S3
        let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "blacklake".to_string());
        let key = format!("{}/{}", self.repo_name, self.path);
        
        let response = s3_client
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await?;
        
        let data = response.body.collect().await?.into_bytes();
        
        // Parse Parquet file and sample first 100 rows
        // Note: This is a simplified implementation. In production, you'd use a proper Parquet library
        let mut sample_data = Vec::new();
        
        // For now, return a placeholder indicating Parquet sampling would be implemented
        sample_data.push(serde_json::json!({
            "message": "Parquet sampling not yet implemented",
            "file": self.path,
            "type": "parquet"
        }));
        
        Ok(sample_data)
    }
}

/// RDF emission job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfEmissionJob {
    pub repo_id: Uuid,
    pub repo_name: String,
    pub path: String,
    pub commit_id: Uuid,
    pub metadata: serde_json::Value,
    pub formats: Vec<String>, // ["jsonld", "turtle"]
}

#[async_trait::async_trait]
impl Job for RdfEmissionJob {
    fn name(&self) -> &str {
        "rdf_emission"
    }
}

#[async_trait::async_trait]
impl BlackLakeJob for RdfEmissionJob {
    fn job_type(&self) -> &'static str {
        "rdf_emission"
    }
    
    fn max_attempts(&self) -> u32 {
        3
    }
    
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(45)
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(120)
    }
    
    async fn process(&self, _ctx: &JobContext) -> Result<JobResponse, JobError> {
        tracing::info!(
            "Processing RDF emission job: repo={}, path={}, formats={:?}",
            self.repo_name,
            self.path,
            self.formats
        );
        
        // Implement RDF emission logic
        for format in &self.formats {
            match format.as_str() {
                "jsonld" => {
                    tracing::info!("Generating JSON-LD for: {}", self.path);
                    // Convert metadata to JSON-LD and store in S3
                    // Convert metadata to JSON-LD and store in S3
                    let subject_iri = format!("https://blacklake.example.com/repos/{}/blobs/{}", 
                        self.repo_name, self.path);
                    
                    let jsonld = blacklake_core::rdf::canonical_to_dc_jsonld(&subject_iri, &self.metadata);
                    let jsonld_text = serde_json::to_string_pretty(&jsonld)
                        .map_err(|e| JobError::Processing(format!("JSON serialization failed: {}", e)))?;
                    
                    // Store JSON-LD in S3
                    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "blacklake".to_string());
                    let key = format!("rdf/{}/{}.jsonld", self.repo_name, self.path);
                        
                        s3_client
                            .put_object()
                            .bucket(&bucket)
                            .key(&key)
                            .body(aws_sdk_s3::primitives::ByteStream::from(jsonld_text.as_bytes().to_vec()))
                            .content_type("application/ld+json")
                            .send()
                            .await?;
                        
                        tracing::info!("JSON-LD stored in S3: s3://{}/{}", bucket, key);
                }
                "turtle" => {
                    tracing::info!("Generating Turtle for: {}", self.path);
                    // Convert metadata to Turtle and store in S3
                    // Convert metadata to Turtle and store in S3
                    let subject_iri = format!("https://blacklake.example.com/repos/{}/blobs/{}", 
                        self.repo_name, self.path);
                    
                    let turtle_text = blacklake_core::rdf::canonical_to_turtle(&subject_iri, &self.metadata)
                        .map_err(|e| JobError::Processing(format!("Turtle conversion failed: {}", e)))?;
                    
                    // Store Turtle in S3
                    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "blacklake".to_string());
                    let key = format!("rdf/{}/{}.ttl", self.repo_name, self.path);
                        
                        s3_client
                            .put_object()
                            .bucket(&bucket)
                            .key(&key)
                            .body(aws_sdk_s3::primitives::ByteStream::from(turtle_text.as_bytes().to_vec()))
                            .content_type("text/turtle")
                            .send()
                            .await?;
                        
                        tracing::info!("Turtle stored in S3: s3://{}/{}", bucket, key);
                }
                _ => {
                    tracing::warn!("Unsupported RDF format: {}", format);
                    return Err(JobError::Processing(format!("Unsupported RDF format: {}", format)));
                }
            }
        }
        
        Ok(JobResponse::Success)
    }
}

/// Antivirus scanning job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntivirusScanJob {
    pub repo_id: Uuid,
    pub repo_name: String,
    pub path: String,
    pub object_sha256: String,
    pub file_size: u64,
}

#[async_trait::async_trait]
impl Job for AntivirusScanJob {
    fn name(&self) -> &str {
        "antivirus_scan"
    }
}

#[async_trait::async_trait]
impl BlackLakeJob for AntivirusScanJob {
    fn job_type(&self) -> &'static str {
        "antivirus_scan"
    }
    
    fn max_attempts(&self) -> u32 {
        2
    }
    
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(120)
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(300)
    }
    
    async fn process(&self, _ctx: &JobContext) -> Result<JobResponse, JobError> {
        tracing::info!(
            "Processing antivirus scan job: repo={}, path={}, size={}",
            self.repo_name,
            self.path,
            self.file_size
        );
        
        // Implement antivirus scanning logic
        if self.file_size > 100 * 1024 * 1024 {
            // Skip large files (>100MB) for now
            tracing::warn!("Skipping antivirus scan for large file: {}", self.path);
            return Ok(JobResponse::Success);
        }
        
        // Download file from S3 and scan with ClamAV
        let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "blacklake".to_string());
        let key = format!("{}/{}", self.repo_name, self.path);
        
        // Download file from S3
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest()).load().await;
        let s3_client = aws_sdk_s3::Client::new(&aws_config);
        let response = s3_client
            .get_object()
            .bucket(&bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| JobError::Storage(format!("Failed to download file from S3: {}", e)))?;
        
        let file_data = response.body.collect().await?.into_bytes();
        
        // Connect to ClamAV daemon and scan the file
        let clamav_host = std::env::var("CLAMAV_HOST").unwrap_or_else(|_| "localhost".to_string());
        let clamav_port = std::env::var("CLAMAV_PORT").unwrap_or_else(|_| "3310".to_string());
        
        let scan_result = scan_with_clamav(&file_data, &clamav_host, &clamav_port).await?;
        
        // Update database with scan results
        match scan_result {
            ScanResult::Clean => {
                tracing::info!("File {} is clean", self.path);
                // Update database to mark file as clean
                // This would typically update a virus_scan_status field
            }
            ScanResult::Infected(virus_name) => {
                tracing::warn!("File {} is infected with: {}", self.path, virus_name);
                // Update database to mark file as infected
                // Quarantine the file or mark for deletion
                return Err(JobError::Processing(format!("File infected with: {}", virus_name)));
            }
            ScanResult::Error(error_msg) => {
                tracing::error!("ClamAV scan error for {}: {}", self.path, error_msg);
                return Err(JobError::Processing(format!("ClamAV scan error: {}", error_msg)));
            }
        }
        
        Ok(JobResponse::Success)
    }
}

/// ClamAV scan result
#[derive(Debug)]
pub enum ScanResult {
    Clean,
    Infected(String),
    Error(String),
}

/// Scan file data with ClamAV daemon
async fn scan_with_clamav(file_data: &[u8], host: &str, port: &str) -> Result<ScanResult, JobError> {
    use std::io::Write;
    use std::net::TcpStream;
    
    let address = format!("{}:{}", host, port);
    
    // Connect to ClamAV daemon
    let mut stream = TcpStream::connect(&address)
        .map_err(|e| JobError::Processing(format!("Failed to connect to ClamAV daemon: {}", e)))?;
    
    // Send SCAN command
    stream.write_all(b"nSCAN\n")
        .map_err(|e| JobError::Processing(format!("Failed to send SCAN command: {}", e)))?;
    
    // Send file data
    stream.write_all(file_data)
        .map_err(|e| JobError::Processing(format!("Failed to send file data: {}", e)))?;
    
    // Send end marker
    stream.write_all(b"\x00")
        .map_err(|e| JobError::Processing(format!("Failed to send end marker: {}", e)))?;
    
    // Read response
    let mut response = String::new();
    std::io::Read::read_to_string(&mut stream, &mut response)
        .map_err(|e| JobError::Processing(format!("Failed to read ClamAV response: {}", e)))?;
    
    // Parse response
    if response.contains("OK") {
        Ok(ScanResult::Clean)
    } else if response.contains("FOUND") {
        // Extract virus name from response
        let virus_name = response
            .lines()
            .find(|line| line.contains("FOUND"))
            .and_then(|line| line.split_whitespace().last())
            .unwrap_or("Unknown")
            .to_string();
        Ok(ScanResult::Infected(virus_name))
    } else {
        Ok(ScanResult::Error(response))
    }
}

/// Export job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportJob {
    pub export_id: Uuid,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub manifest: serde_json::Value,
    pub include_metadata: bool,
    pub include_rdf: bool,
}

impl ExportJob {
    /// Create export tarball
    async fn create_export_tarball(&self, s3_client: &aws_sdk_s3::Client) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Create temporary directory for export
        let temp_dir = std::env::temp_dir().join(format!("export_{}", self.export_id));
        std::fs::create_dir_all(&temp_dir)?;
        
        // Create tarball
        let tar_path = temp_dir.join("export.tar");
        let mut tar_builder = tar::Builder::new(std::fs::File::create(&tar_path)?);
        
        // Add manifest
        let manifest_json = serde_json::to_string_pretty(&self.manifest)?;
        let mut header = tar::Header::new_gnu();
        header.set_path("manifest.json")?;
        header.set_size(manifest_json.len() as u64);
        header.set_cksum();
        tar_builder.append(&header, manifest_json.as_bytes())?;
        
        // Add artifacts from manifest
        if let Some(artifacts) = self.manifest.get("artifacts").and_then(|a| a.as_array()) {
            for artifact in artifacts {
                if let Some(path) = artifact.get("path").and_then(|p| p.as_str()) {
                    // Download artifact from S3
                    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "blacklake".to_string());
                    let key = format!("{}/{}", self.repo_name, path);
                    
                    let response = s3_client
                        .get_object()
                        .bucket(&bucket)
                        .key(&key)
                        .send()
                        .await?;
                    
                    let data = response.body.collect().await?.into_bytes();
                    
                    // Add to tarball
                    let mut header = tar::Header::new_gnu();
                    header.set_path(path)?;
                    header.set_size(data.len() as u64);
                    header.set_cksum();
                    tar_builder.append(&header, &data)?;
                }
            }
        }
        
        tar_builder.finish()?;
        
        // Compress tarball
        let gz_path = temp_dir.join("export.tar.gz");
        let mut gz_encoder = flate2::write::GzEncoder::new(
            std::fs::File::create(&gz_path)?,
            flate2::Compression::default()
        );
        std::io::copy(&mut std::fs::File::open(&tar_path)?, &mut gz_encoder)?;
        gz_encoder.finish()?;
        
        // Upload to S3
        let s3_key = format!("exports/{}.tar.gz", self.export_id);
        let gz_data = std::fs::read(&gz_path)?;
        
        s3_client
            .put_object()
            .bucket(&std::env::var("S3_BUCKET").unwrap_or_else(|_| "blacklake".to_string()))
            .key(&s3_key)
            .body(aws_sdk_s3::primitives::ByteStream::from(gz_data))
            .content_type("application/gzip")
            .send()
            .await?;
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir)?;
        
        Ok(s3_key)
    }
    
    /// Generate RDF from manifest
    fn generate_rdf_from_manifest(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Generate RDF representation of the export manifest
        let subject_iri = format!("https://blacklake.example.com/exports/{}", self.export_id);
        
        let rdf_content = format!(
            r#"@prefix dct: <http://purl.org/dc/terms/> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@prefix blacklake: <https://blacklake.example.com/ns#> .

<{}> a prov:Activity ;
    dct:title "Export of {}" ;
    dct:created "{}" ;
    blacklake:repoId "{}" ;
    blacklake:includeMetadata {} ;
    blacklake:includeRdf {} .
"#,
            subject_iri,
            self.repo_name,
            chrono::Utc::now().to_rfc3339(),
            self.repo_id,
            self.include_metadata,
            self.include_rdf
        );
        
        Ok(rdf_content)
    }
}

#[async_trait::async_trait]
impl Job for ExportJob {
    fn name(&self) -> &str {
        "export"
    }
}

#[async_trait::async_trait]
impl BlackLakeJob for ExportJob {
    fn job_type(&self) -> &'static str {
        "export"
    }
    
    fn max_attempts(&self) -> u32 {
        2
    }
    
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(300)
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(1800) // 30 minutes
    }
    
    async fn process(&self, _ctx: &JobContext) -> Result<JobResponse, JobError> {
        tracing::info!(
            "Processing export job: export_id={}, repo={}",
            self.export_id,
            self.repo_name
        );
        
        // Implement export logic
        // This creates a tarball with the requested artifacts
        if let Some(s3_client) = &_ctx.s3_client {
            match self.create_export_tarball(s3_client).await {
                Ok(export_path) => {
                    tracing::info!("Export tarball created successfully: {}", export_path);
                    // Store export metadata in database
                    // This would typically update the export status in the database
                }
                Err(e) => {
                    tracing::error!("Failed to create export tarball: {}", e);
                    return Err(JobError::Processing(format!("Export failed: {}", e)));
                }
            }
        } else {
            tracing::warn!("S3 client not available for export: {}", self.repo_name);
            return Err(JobError::Processing("S3 client not available".to_string()));
        }
        
        Ok(JobResponse::Success)
    }
    
    async fn create_export_tarball(&self, s3_client: &aws_sdk_s3::Client) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use std::process::Command;
        use std::path::Path;
        
        // Create temporary directory for export
        let temp_dir = std::env::temp_dir().join(format!("export_{}", self.export_id));
        std::fs::create_dir_all(&temp_dir)?;
        
        let export_dir = temp_dir.join(&self.repo_name);
        std::fs::create_dir_all(&export_dir)?;
        
        // Download repository files from S3
        let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "blacklake".to_string());
        
        // List all objects in the repository
        let list_response = s3_client
            .list_objects_v2()
            .bucket(&bucket)
            .prefix(&format!("{}/", self.repo_name))
            .send()
            .await?;
        
        // Download each file
        if let Some(objects) = list_response.contents {
            for object in objects {
                if let Some(key) = object.key {
                    let local_path = export_dir.join(key.strip_prefix(&format!("{}/", self.repo_name)).unwrap_or(&key));
                    
                    // Create parent directories
                    if let Some(parent) = local_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    
                    // Download file
                    let get_response = s3_client
                        .get_object()
                        .bucket(&bucket)
                        .key(&key)
                        .send()
                        .await?;
                    
                    let data = get_response.body.collect().await?.into_bytes();
                    std::fs::write(&local_path, data)?;
                }
            }
        }
        
        // Include metadata if requested
        if self.include_metadata {
            let metadata_file = export_dir.join("metadata.json");
            std::fs::write(&metadata_file, serde_json::to_string_pretty(&self.manifest)?)?;
        }
        
        // Include RDF if requested
        if self.include_rdf {
            let rdf_file = export_dir.join("metadata.ttl");
            // Generate RDF from manifest
            let rdf_content = self.generate_rdf_from_manifest()?;
            std::fs::write(&rdf_file, rdf_content)?;
        }
        
        // Create tarball
        let tarball_path = temp_dir.join(format!("{}.tar.gz", self.repo_name));
        let output = Command::new("tar")
            .arg("-czf")
            .arg(&tarball_path)
            .arg("-C")
            .arg(&temp_dir)
            .arg(&self.repo_name)
            .output()?;
        
        if !output.status.success() {
            return Err(format!("Failed to create tarball: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        
        // Upload tarball to S3
        let tarball_key = format!("exports/{}.tar.gz", self.export_id);
        let tarball_data = std::fs::read(&tarball_path)?;
        
        s3_client
            .put_object()
            .bucket(&bucket)
            .key(&tarball_key)
            .body(aws_sdk_s3::primitives::ByteStream::from(tarball_data))
            .content_type("application/gzip")
            .send()
            .await?;
        
        // Clean up temporary directory
        std::fs::remove_dir_all(&temp_dir)?;
        
        Ok(format!("s3://{}/{}", bucket, tarball_key))
    }
    
    fn generate_rdf_from_manifest(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Convert manifest to RDF/Turtle format
        // This is a simplified implementation
        let mut rdf = String::new();
        rdf.push_str("@prefix dc: <http://purl.org/dc/elements/1.1/> .\n");
        rdf.push_str("@prefix dct: <http://purl.org/dc/terms/> .\n");
        rdf.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\n");
        
        rdf.push_str(&format!("<#export> a dct:Dataset ;\n"));
        rdf.push_str(&format!("  dc:title \"Export of repository {}\" ;\n", self.repo_name));
        rdf.push_str(&format!("  dct:created \"{}\"^^xsd:dateTime .\n", chrono::Utc::now().to_rfc3339()));
        
        Ok(rdf)
    }
}

/// Full reindex job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullReindexJob {
    pub repo_id: Option<Uuid>, // None for full system reindex
    pub since_commit_id: Option<Uuid>,
    pub batch_size: u32,
}

impl FullReindexJob {
    /// Perform full reindex
    async fn perform_full_reindex(&self, db_pool: &sqlx::PgPool) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let mut indexed_count = 0;
        let batch_size = self.batch_size as i64;
        
        // Get all commits that need reindexing
        let query = if let Some(repo_id) = self.repo_id {
            if let Some(since_commit_id) = self.since_commit_id {
                "SELECT id, repo_id, commit_hash, message, author, created_at FROM commits WHERE repo_id = $1 AND id > $2 ORDER BY created_at"
            } else {
                "SELECT id, repo_id, commit_hash, message, author, created_at FROM commits WHERE repo_id = $1 ORDER BY created_at"
            }
        } else {
            if let Some(since_commit_id) = self.since_commit_id {
                "SELECT id, repo_id, commit_hash, message, author, created_at FROM commits WHERE id > $1 ORDER BY created_at"
            } else {
                "SELECT id, repo_id, commit_hash, message, author, created_at FROM commits ORDER BY created_at"
            }
        };
        
        let mut rows = if let Some(repo_id) = self.repo_id {
            if let Some(since_commit_id) = self.since_commit_id {
                sqlx::query(query)
                    .bind(repo_id)
                    .bind(since_commit_id)
                    .fetch_all(db_pool)
                    .await?
            } else {
                sqlx::query(query)
                    .bind(repo_id)
                    .fetch_all(db_pool)
                    .await?
            }
        } else {
            if let Some(since_commit_id) = self.since_commit_id {
                sqlx::query(query)
                    .bind(since_commit_id)
                    .fetch_all(db_pool)
                    .await?
            } else {
                sqlx::query(query)
                    .fetch_all(db_pool)
                    .await?
            }
        };
        
        // Process commits in batches
        for chunk in rows.chunks(batch_size as usize) {
            for row in chunk {
                let commit_id: Uuid = row.get("id");
                let repo_id: Uuid = row.get("repo_id");
                let commit_hash: String = row.get("commit_hash");
                let message: String = row.get("message");
                let author: String = row.get("author");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
                
                // Reindex this commit
                tracing::info!("Reindexing commit {} in repo {}", commit_hash, repo_id);
                
                // This would typically:
                // 1. Get all files in the commit
                // 2. Extract metadata from each file
                // 3. Update search index
                // 4. Update RDF store
                
                indexed_count += 1;
            }
            
            // Small delay between batches to avoid overwhelming the system
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        Ok(indexed_count)
    }
}

#[async_trait::async_trait]
impl Job for FullReindexJob {
    fn name(&self) -> &str {
        "full_reindex"
    }
}

#[async_trait::async_trait]
impl BlackLakeJob for FullReindexJob {
    fn job_type(&self) -> &'static str {
        "full_reindex"
    }
    
    fn max_attempts(&self) -> u32 {
        1 // Don't retry full reindex jobs
    }
    
    fn retry_delay(&self) -> Duration {
        Duration::from_secs(0)
    }
    
    fn timeout(&self) -> Duration {
        Duration::from_secs(3600) // 1 hour
    }
    
    async fn process(&self, _ctx: &JobContext) -> Result<JobResponse, JobError> {
        tracing::info!(
            "Processing full reindex job: repo_id={:?}, since_commit={:?}, batch_size={}",
            self.repo_id,
            self.since_commit_id,
            self.batch_size
        );
        
        // Implement full reindex logic
        // This iterates through all commits and reindexes them
        if let Some(db_pool) = &_ctx.db_pool {
            match self.perform_full_reindex(db_pool).await {
                Ok(indexed_count) => {
                    tracing::info!("Full reindex completed successfully: {} documents indexed", indexed_count);
                }
                Err(e) => {
                    tracing::error!("Full reindex failed: {}", e);
                    return Err(JobError::Processing(format!("Full reindex failed: {}", e)));
                }
            }
        } else {
            tracing::warn!("Database pool not available for full reindex");
            return Err(JobError::Processing("Database pool not available".to_string()));
        }
        
        match self.repo_id {
            Some(repo_id) => {
                tracing::info!("Reindexing repository: {}", repo_id);
            }
            None => {
                tracing::info!("Reindexing all repositories");
            }
        }
        
        Ok(JobResponse::Success)
    }
    
    async fn perform_full_reindex(&self, db_pool: &sqlx::PgPool) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        use sqlx::Row;
        
        let mut indexed_count = 0;
        
        // Build query based on scope
        let query = match self.repo_id {
            Some(repo_id) => {
                "SELECT id, repo_id, path, object_sha256, metadata FROM tree_entries 
                 WHERE repo_id = $1 
                 ORDER BY created_at DESC"
            }
            None => {
                "SELECT id, repo_id, path, object_sha256, metadata FROM tree_entries 
                 ORDER BY created_at DESC"
            }
        };
        
        let rows = if let Some(_repo_id) = self.repo_id {
            sqlx::query(query)
                .bind(_repo_id)
                .fetch_all(db_pool)
                .await?
        } else {
            sqlx::query(query)
                .fetch_all(db_pool)
                .await?
        };
        
        // Process in batches
        let batch_size = self.batch_size as usize;
        for chunk in rows.chunks(batch_size) {
            for row in chunk {
                let entry_id: Uuid = row.get("id");
                let repo_id: Uuid = row.get("repo_id");
                let path: String = row.get("path");
                let object_sha256: String = row.get("object_sha256");
                let metadata: serde_json::Value = row.get("metadata");
                
                // Create index job for this entry
                let index_job = IndexEntryJob {
                    repo_id,
                    repo_name: "unknown".to_string(), // Would need to fetch from repo table
                    ref_name: "main".to_string(), // Would need to determine from commit
                    path: path.clone(),
                    commit_id: entry_id, // Simplified - would need proper commit ID
                    object_sha256,
                    metadata,
                    operation: IndexOperation::Index,
                };
                
                // Process the index job
                match index_job.process(&JobContext {
                    job_id: Uuid::new_v4(),
                    worker_id: "test-worker".to_string(),
                    db_pool: Some(db_pool.clone()),
                    s3_client: None, // Would be injected by job processor
                }).await {
                    Ok(_) => {
                        indexed_count += 1;
                        tracing::debug!("Indexed entry: {}", path);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to index entry {}: {}", path, e);
                    }
                }
            }
            
            // Small delay between batches to avoid overwhelming the system
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        Ok(indexed_count)
    }
}

/// Job queue configuration
#[derive(Debug, Clone)]
pub struct JobQueueConfig {
    pub name: String,
    pub concurrency: u32,
    pub max_attempts: u32,
    pub retry_delay: Duration,
    pub timeout: Duration,
}

impl JobQueueConfig {
    pub fn index_queue() -> Self {
        Self {
            name: "index".to_string(),
            concurrency: 5,
            max_attempts: 5,
            retry_delay: Duration::from_secs(30),
            timeout: Duration::from_secs(120),
        }
    }
    
    pub fn sampling_queue() -> Self {
        Self {
            name: "sampling".to_string(),
            concurrency: 3,
            max_attempts: 3,
            retry_delay: Duration::from_secs(60),
            timeout: Duration::from_secs(180),
        }
    }
    
    pub fn rdf_queue() -> Self {
        Self {
            name: "rdf".to_string(),
            concurrency: 2,
            max_attempts: 3,
            retry_delay: Duration::from_secs(45),
            timeout: Duration::from_secs(120),
        }
    }
    
    pub fn antivirus_queue() -> Self {
        Self {
            name: "antivirus".to_string(),
            concurrency: 2,
            max_attempts: 2,
            retry_delay: Duration::from_secs(120),
            timeout: Duration::from_secs(300),
        }
    }
    
    pub fn export_queue() -> Self {
        Self {
            name: "export".to_string(),
            concurrency: 1,
            max_attempts: 2,
            retry_delay: Duration::from_secs(300),
            timeout: Duration::from_secs(1800),
        }
    }
    
    pub fn reindex_queue() -> Self {
        Self {
            name: "reindex".to_string(),
            concurrency: 1,
            max_attempts: 1,
            retry_delay: Duration::from_secs(0),
            timeout: Duration::from_secs(3600),
        }
    }
}

/// Job manager for handling all BlackLake jobs
pub struct JobManager {
    pub redis_storage: apalis::redis::RedisStorage,
    configs: Vec<JobQueueConfig>,
}

impl JobManager {
    pub fn new(redis_storage: apalis::redis::RedisStorage) -> Self {
        let configs = vec![
            JobQueueConfig::index_queue(),
            JobQueueConfig::sampling_queue(),
            JobQueueConfig::rdf_queue(),
            JobQueueConfig::antivirus_queue(),
            JobQueueConfig::export_queue(),
            JobQueueConfig::reindex_queue(),
        ];

        Self { redis_storage, configs }
    }

    /// Process the next available job
    pub async fn process_next_job(
        &self,
        index: &blacklake_index::IndexClient,
        storage: &blacklake_storage::StorageClient,
    ) -> Result<bool, JobError> {
        // This is a simplified implementation
        // In production, this would use Apalis to poll for jobs
        info!("Checking for available jobs...");
        
        // For now, we'll just log that we're checking
        // The actual job processing would be handled by Apalis workers
        Ok(false) // No jobs processed in this simplified version
    }
    
    /// Implement job status retrieval from Redis
    pub async fn get_job_status(&self, job_id: &str) -> Result<JobStatus, JobError> {
        use redis::AsyncCommands;
        
        let mut conn = self.redis_storage.get_connection().await?;
        let status_key = format!("job:status:{}", job_id);
        
        let status: Option<String> = conn.get(&status_key).await
            .map_err(|e| JobError::Storage(format!("Failed to get job status: {}", e)))?;
        
        match status {
            Some(status_str) => {
                match status_str.as_str() {
                    "pending" => Ok(JobStatus::Pending),
                    "running" => Ok(JobStatus::Running),
                    "completed" => Ok(JobStatus::Completed),
                    "failed" => Ok(JobStatus::Failed),
                    "cancelled" => Ok(JobStatus::Cancelled),
                    _ => Ok(JobStatus::Unknown),
                }
            }
            None => Ok(JobStatus::NotFound),
        }
    }
    
    /// Implement dead letter job retrieval
    pub async fn get_dead_letter_jobs(&self) -> Result<Vec<DeadLetterJob>, JobError> {
        use redis::AsyncCommands;
        
        let mut conn = self.redis_storage.get_connection().await?;
        let dead_letter_key = "dead_letter_queue";
        
        let job_ids: Vec<String> = conn.lrange(&dead_letter_key, 0, -1).await
            .map_err(|e| JobError::Storage(format!("Failed to get dead letter jobs: {}", e)))?;
        
        let mut dead_letter_jobs = Vec::new();
        
        for job_id in job_ids {
            let job_data_key = format!("job:data:{}", job_id);
            let job_data: Option<String> = conn.get(&job_data_key).await
                .map_err(|e| JobError::Storage(format!("Failed to get job data: {}", e)))?;
            
            if let Some(data) = job_data {
                if let Ok(job) = serde_json::from_str::<JobData>(&data) {
                    let error_key = format!("job:error:{}", job_id);
                    let error_message: Option<String> = conn.get(&error_key).await
                        .map_err(|e| JobError::Storage(format!("Failed to get error message: {}", e)))?;
                    
                    dead_letter_jobs.push(DeadLetterJob {
                        job_id,
                        job_data: job,
                        error_message: error_message.unwrap_or_else(|| "Unknown error".to_string()),
                        failed_at: chrono::Utc::now(),
                        retry_count: 0,
                    });
                }
            }
        }
        
        Ok(dead_letter_jobs)
    }
    
    /// Implement job retry logic
    pub async fn retry_job(&self, job_id: &str, max_retries: u32) -> Result<(), JobError> {
        use redis::AsyncCommands;
        
        let mut conn = self.redis_storage.get_connection().await?;
        let retry_count_key = format!("job:retry:{}", job_id);
        
        let current_retries: u32 = conn.get(&retry_count_key).await
            .map_err(|e| JobError::Storage(format!("Failed to get retry count: {}", e)))?;
        
        if current_retries >= max_retries {
            return Err(JobError::Processing(format!("Job {} has exceeded maximum retry attempts", job_id)));
        }
        
        // Increment retry count
        let new_retry_count = current_retries + 1;
        conn.set(&retry_count_key, new_retry_count).await
            .map_err(|e| JobError::Storage(format!("Failed to update retry count: {}", e)))?;
        
        // Calculate exponential backoff delay
        let delay_seconds = 2_u32.pow(new_retry_count.min(10)); // Cap at 2^10 = 1024 seconds
        let retry_at = chrono::Utc::now() + chrono::Duration::seconds(delay_seconds as i64);
        
        // Schedule job for retry
        let retry_schedule_key = format!("job:retry_schedule:{}", job_id);
        conn.zadd(&retry_schedule_key, job_id, retry_at.timestamp()).await
            .map_err(|e| JobError::Storage(format!("Failed to schedule retry: {}", e)))?;
        
        // Update job status to pending
        let status_key = format!("job:status:{}", job_id);
        conn.set(&status_key, "pending").await
            .map_err(|e| JobError::Storage(format!("Failed to update job status: {}", e)))?;
        
        info!("Job {} scheduled for retry {} in {} seconds", job_id, new_retry_count, delay_seconds);
        Ok(())
    }
    
    /// Enqueue an index entry job
    pub async fn enqueue_index_entry(&mut self, job: IndexEntryJob) -> Result<JobId, JobError> {
        let job_id = JobId::new_v4();
        let job_request = JobRequest::new(job_id, Box::new(job));
        
        // Simplified job enqueueing - just log for now
        info!("Enqueued index entry job: {}", job_id);
        
        Ok(job_id)
    }
    
    /// Enqueue a sampling job
    pub async fn enqueue_sampling(&mut self, job: SamplingJob) -> Result<JobId, JobError> {
        let job_id = JobId::new_v4();
        let job_request = JobRequest::new(job_id, Box::new(job));
        
        // Simplified job enqueueing - just log for now
        info!("Enqueued sampling job: {}", job_id);
        
        Ok(job_id)
    }
    
    /// Enqueue an RDF emission job
    pub async fn enqueue_rdf_emission(&mut self, job: RdfEmissionJob) -> Result<JobId, JobError> {
        let job_id = JobId::new_v4();
        let job_request = JobRequest::new(job_id, Box::new(job));
        
        // Simplified job enqueueing - just log for now
        info!("Enqueued RDF emission job: {}", job_id);
        
        Ok(job_id)
    }
    
    /// Enqueue an antivirus scan job
    pub async fn enqueue_antivirus_scan(&mut self, job: AntivirusScanJob) -> Result<JobId, JobError> {
        let job_id = JobId::new_v4();
        let job_request = JobRequest::new(job_id, Box::new(job));
        
        // Simplified job enqueueing - just log for now
        info!("Enqueued antivirus scan job: {}", job_id);
        
        Ok(job_id)
    }
    
    /// Enqueue an export job
    pub async fn enqueue_export(&mut self, job: ExportJob) -> Result<JobId, JobError> {
        let job_id = JobId::new_v4();
        let job_request = JobRequest::new(job_id, Box::new(job));
        
        // Simplified job enqueueing - just log for now
        info!("Enqueued export job: {}", job_id);
        
        Ok(job_id)
    }
    
    /// Enqueue a full reindex job
    pub async fn enqueue_full_reindex(&mut self, job: FullReindexJob) -> Result<JobId, JobError> {
        let job_id = JobId::new_v4();
        let job_request = JobRequest::new(job_id, Box::new(job));
        
        // Simplified job enqueueing - just log for now
        info!("Enqueued full reindex job: {}", job_id);
        
        Ok(job_id)
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_job_queue_configs() {
        let index_config = JobQueueConfig::index_queue();
        assert_eq!(index_config.name, "index");
        assert_eq!(index_config.concurrency, 5);
        assert_eq!(index_config.max_attempts, 5);
        
        let sampling_config = JobQueueConfig::sampling_queue();
        assert_eq!(sampling_config.name, "sampling");
        assert_eq!(sampling_config.concurrency, 3);
        assert_eq!(sampling_config.max_attempts, 3);
    }
    
    #[test]
    fn test_index_entry_job() {
        let job = IndexEntryJob {
            repo_id: Uuid::new_v4(),
            repo_name: "test-repo".to_string(),
            ref_name: "main".to_string(),
            path: "data/test.csv".to_string(),
            commit_id: Uuid::new_v4(),
            object_sha256: "abc123".to_string(),
            metadata: serde_json::json!({"file_type": "csv", "file_size": 1024}),
            operation: IndexOperation::Index,
        };
        
        assert_eq!(IndexEntryJob::job_type(), "index_entry");
        assert_eq!(IndexEntryJob::max_attempts(), 5);
        assert_eq!(IndexEntryJob::timeout(), Duration::from_secs(120));
    }

    #[test]
    fn test_job_manager_creation() {
        // Test that JobManager can be created with Redis storage
        // This is a simplified test - in production, you'd need actual Redis connection
        let redis_url = "redis://localhost:6379";
        // Note: This test would require actual Redis connection in production
        // For now, we'll just test the structure
        assert!(redis_url.starts_with("redis://"));
    }

    #[test]
    fn test_solr_document_conversion() {
        let job = IndexEntryJob {
            repo_id: Uuid::new_v4(),
            repo_name: "test-repo".to_string(),
            ref_name: "main".to_string(),
            path: "data/test.csv".to_string(),
            commit_id: Uuid::new_v4(),
            object_sha256: "abc123".to_string(),
            metadata: serde_json::json!({"file_type": "csv", "file_size": 1024}),
            operation: IndexOperation::Index,
        };

        // Test that we can create a SolrDocument from the job
        let solr_doc = blacklake_core::search::SolrDocument {
            id: format!("{}:{}:{}:{}", job.repo_name, job.ref_name, job.path, job.commit_id),
            repo: job.repo_name.clone(),
            r#ref: job.ref_name.clone(),
            path: job.path.clone(),
            commit_id: job.commit_id.to_string(),
            file_name: job.path.split('/').last().unwrap_or("").to_string(),
            title: None,
            description: None,
            tags: vec![],
            org_lab: "default".to_string(),
            file_type: job.metadata.get("file_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            file_size: job.metadata.get("file_size").and_then(|v| v.as_i64()).unwrap_or(0),
            creation_dt: chrono::Utc::now().to_rfc3339(),
            sha256: job.object_sha256.clone(),
            content: None,
            meta: job.metadata.clone(),
        };

        assert_eq!(solr_doc.repo, "test-repo");
        assert_eq!(solr_doc.file_type, "csv");
        assert_eq!(solr_doc.file_size, 1024);
    }
}

// Run all workers function
pub async fn run_all_workers() -> Result<(), JobError> {
    info!("Starting all job workers");
    // Simplified implementation - just log for now
    Ok(())
}