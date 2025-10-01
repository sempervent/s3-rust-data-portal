// BlackLake JobRunner
// Week 5: Background job processor

use blacklake_core::Uuid;
use blacklake_index::IndexClient;
use blacklake_storage::StorageClient;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting BlackLake JobRunner");

    // Initialize database and storage clients
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://blacklake:password@localhost:5432/blacklake".to_string());
    
    let s3_bucket = std::env::var("S3_BUCKET")
        .unwrap_or_else(|_| "blacklake-storage".to_string());
    
    let s3_region = std::env::var("S3_REGION")
        .unwrap_or_else(|_| "us-east-1".to_string());
    
    let index = IndexClient::from_pool(
        sqlx::PgPool::connect(&database_url).await?
    );
    
    let storage = StorageClient::new(&s3_bucket);
    
    info!("Initialized database and storage clients");

    // Start health check server
    let health_server = tokio::spawn(start_health_server());

    // Start job processing loop
    let job_processor = tokio::spawn(process_jobs());

    // Wait for shutdown signal
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        _ = health_server => {
            error!("Health server stopped unexpectedly");
        }
        _ = job_processor => {
            error!("Job processor stopped unexpectedly");
        }
    }

    info!("BlackLake JobRunner shutting down");
    Ok(())
}

async fn start_health_server() {
    // Simple health check server
    use axum::{response::Json, routing::get, Router};
    use serde_json::json;

    let app = Router::new().route("/health", get(|| async {
        Json(json!({"status": "healthy", "service": "jobrunner"}))
    }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn process_jobs() {
    // Initialize job processing components
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let redis_client = redis::Client::open(redis_url).unwrap();
    let redis_conn = redis_client.get_async_connection().await.unwrap();
    
    // Initialize job manager
    let job_manager = blacklake_core::jobs::JobManager::new(
        Arc::new(StorageClient::new("blacklake-storage")),
        Arc::new(IndexClient::from_pool(
            sqlx::PgPool::connect(&std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://blacklake:password@localhost:5432/blacklake".to_string()))
                .await.unwrap()
        )),
        Some(redis_conn),
        None, // Solr client
    );
    
    loop {
        // Implement job processing logic
        // - Poll for pending jobs
        let pending_jobs = job_manager.get_pending_jobs().await.unwrap_or_default();
        
        for job in pending_jobs {
            info!("Processing job: {}", job.id);
            
            // Process different job types
            match job.job_type {
                blacklake_core::jobs::JobType::AntivirusScan => {
                    if let Err(e) = job_manager.process_antivirus_scan(&job).await {
                        error!("Failed to process antivirus scan job {}: {}", job.id, e);
                    }
                }
                blacklake_core::jobs::JobType::CsvSample => {
                    if let Err(e) = job_manager.process_csv_sample(&job).await {
                        error!("Failed to process CSV sample job {}: {}", job.id, e);
                    }
                }
                blacklake_core::jobs::JobType::ParquetSample => {
                    if let Err(e) = job_manager.process_parquet_sample(&job).await {
                        error!("Failed to process Parquet sample job {}: {}", job.id, e);
                    }
                }
                blacklake_core::jobs::JobType::OnnxSniff => {
                    if let Err(e) = job_manager.process_onnx_sniff(&job).await {
                        error!("Failed to process ONNX sniff job {}: {}", job.id, e);
                    }
                }
                blacklake_core::jobs::JobType::RdfEmit => {
                    if let Err(e) = job_manager.process_rdf_emit(&job).await {
                        error!("Failed to process RDF emit job {}: {}", job.id, e);
                    }
                }
                blacklake_core::jobs::JobType::Export => {
                    if let Err(e) = job_manager.process_export(&job).await {
                        error!("Failed to process export job {}: {}", job.id, e);
                    }
                }
                _ => {
                    warn!("Unknown job type: {:?}", job.job_type);
                }
            }
        }
        
        info!("Processed {} jobs", pending_jobs.len());
        sleep(Duration::from_secs(30)).await;
    }
}
