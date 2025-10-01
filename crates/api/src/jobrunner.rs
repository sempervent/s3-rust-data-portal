// BlackLake JobRunner
// Week 1: Production-ready job processor with Redis

use blacklake_core::{
    jobs::{JobContext, JobManager, BlackLakeJob, JobResponse, JobError},
    Uuid,
};
use blacklake_index::IndexClient;
use blacklake_storage::StorageClient;
use apalis::{
    prelude::*,
    redis::RedisStorage,
};
use redis::Client as RedisClient;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting BlackLake JobRunner");

    // Initialize Redis connection
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let redis_client = RedisClient::open(redis_url)
        .map_err(|e| format!("Failed to connect to Redis: {}", e))?;
    
    let redis_storage = RedisStorage::new(redis_client);

    // Initialize database and storage clients
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/blacklake".to_string());
    
    let index = IndexClient::new(&database_url).await
        .map_err(|e| format!("Failed to initialize index client: {}", e))?;
    
    let storage = StorageClient::new().await
        .map_err(|e| format!("Failed to initialize storage client: {}", e))?;

    // Initialize job manager
    let job_manager = JobManager::new(redis_storage.clone());

    // Start health check server
    let health_server = tokio::spawn(start_health_server());

    // Start job processing workers
    let job_processor = tokio::spawn(process_jobs(job_manager, index, storage));

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

async fn process_jobs(
    job_manager: JobManager,
    index: IndexClient,
    storage: StorageClient,
) {
    info!("Starting job processing workers");
    
    loop {
        match job_manager.process_next_job(&index, &storage).await {
            Ok(processed) => {
                if processed {
                    info!("Successfully processed a job");
                } else {
                    // No jobs available, wait before checking again
                    sleep(Duration::from_secs(5)).await;
                }
            }
            Err(e) => {
                error!("Error processing job: {}", e);
                // Wait before retrying
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}
