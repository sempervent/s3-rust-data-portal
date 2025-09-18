// BlackLake Webhook System
// Week 4: Webhook delivery with retries and dead letter handling

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use blacklake_core::{
    ApiError, ApiResponse, AuthContext, Webhook, WebhookDelivery, WebhookDead,
    WebhookEvent, WebhookPayload, Uuid,
};
use blacklake_index::IndexClient;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::RwLock,
    time::{interval, sleep},
};
use tracing::{error, info, warn};
use url::Url;

/// Webhook delivery configuration
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
    pub timeout_seconds: u64,
    pub batch_size: usize,
    pub delivery_interval_seconds: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_seconds: 60,
            timeout_seconds: 30,
            batch_size: 10,
            delivery_interval_seconds: 5,
        }
    }
}

/// Webhook delivery client
#[derive(Debug, Clone)]
pub struct WebhookClient {
    index: Arc<IndexClient>,
    config: WebhookConfig,
    http_client: reqwest::Client,
}

/// Webhook delivery worker
#[derive(Debug)]
pub struct WebhookWorker {
    client: WebhookClient,
    running: Arc<RwLock<bool>>,
}

impl WebhookClient {
    /// Create a new webhook client
    pub fn new(index: Arc<IndexClient>, config: WebhookConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            index,
            config,
            http_client,
        }
    }

    /// Deliver a webhook payload
    pub async fn deliver_webhook(
        &self,
        webhook: &Webhook,
        payload: &WebhookPayload,
    ) -> Result<WebhookDelivery, ApiError> {
        let delivery_id = Uuid::new_v4();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create delivery record
        let mut delivery = WebhookDelivery {
            id: delivery_id,
            webhook_id: webhook.id,
            event: payload.event.clone(),
            payload: serde_json::to_value(payload).unwrap(),
            status: "pending".to_string(),
            attempts: 0,
            last_attempt_at: None,
            next_retry_at: Some(now),
            response_status: None,
            response_body: None,
            error_message: None,
            created_at: now,
            updated_at: now,
        };

        // Store delivery record
        self.index.create_webhook_delivery(&delivery).await?;

        // Attempt delivery
        match self.attempt_delivery(&webhook, &payload, &mut delivery).await {
            Ok(()) => {
                delivery.status = "delivered".to_string();
                delivery.updated_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                self.index.update_webhook_delivery(&delivery).await?;
                Ok(delivery)
            }
            Err(e) => {
                delivery.status = "failed".to_string();
                delivery.error_message = Some(e.to_string());
                delivery.updated_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                self.index.update_webhook_delivery(&delivery).await?;
                Err(e)
            }
        }
    }

    /// Attempt webhook delivery
    async fn attempt_delivery(
        &self,
        webhook: &Webhook,
        payload: &WebhookPayload,
        delivery: &mut WebhookDelivery,
    ) -> Result<(), ApiError> {
        delivery.attempts += 1;
        delivery.last_attempt_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        // Prepare request
        let payload_json = serde_json::to_string(payload)
            .map_err(|e| ApiError::InternalServerError(format!("Payload serialization failed: {}", e)))?;

        let mut request = self.http_client
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "BlackLake-Webhook/1.0")
            .header("X-Webhook-Event", &payload.event.to_string())
            .header("X-Webhook-Delivery", delivery.id.to_string())
            .body(payload_json);

        // Add signature if secret is provided
        if let Some(secret) = &webhook.secret {
            let signature = self.generate_signature(&payload_json, secret);
            request = request.header("X-Webhook-Signature", signature);
        }

        // Send request
        let response = request.send().await
            .map_err(|e| ApiError::InternalServerError(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        let response_body = response.text().await
            .map_err(|e| ApiError::InternalServerError(format!("Failed to read response: {}", e)))?;

        delivery.response_status = Some(status.as_u16());
        delivery.response_body = Some(response_body.clone());

        if status.is_success() {
            info!("Webhook delivered successfully: {} -> {}", webhook.id, webhook.url);
            Ok(())
        } else {
            let error_msg = format!("HTTP {}: {}", status, response_body);
            error!("Webhook delivery failed: {} -> {}: {}", webhook.id, webhook.url, error_msg);
            Err(ApiError::InternalServerError(error_msg))
        }
    }

    /// Generate webhook signature
    fn generate_signature(&self, payload: &str, secret: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        format!("sha256={}", hex::encode(result.into_bytes()))
    }

    /// Get pending webhook deliveries
    pub async fn get_pending_deliveries(&self) -> Result<Vec<WebhookDelivery>, ApiError> {
        self.index.get_pending_webhook_deliveries().await
    }

    /// Retry failed webhook delivery
    pub async fn retry_delivery(&self, delivery_id: Uuid) -> Result<(), ApiError> {
        let delivery = self.index.get_webhook_delivery(delivery_id).await?;
        let webhook = self.index.get_webhook(delivery.webhook_id).await?;

        // Parse payload
        let payload: WebhookPayload = serde_json::from_value(delivery.payload)
            .map_err(|e| ApiError::InternalServerError(format!("Payload deserialization failed: {}", e)))?;

        // Check if we should retry or move to dead letter
        if delivery.attempts < self.config.max_retries {
            // Calculate next retry time with exponential backoff
            let delay = self.config.retry_delay_seconds * (2_u64.pow(delivery.attempts));
            let next_retry = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + delay;

            let mut updated_delivery = delivery.clone();
            updated_delivery.status = "pending".to_string();
            updated_delivery.next_retry_at = Some(next_retry);
            updated_delivery.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            self.index.update_webhook_delivery(&updated_delivery).await?;
            info!("Webhook delivery scheduled for retry: {}", delivery_id);
        } else {
            // Move to dead letter queue
            self.move_to_dead_letter(delivery).await?;
        }

        Ok(())
    }

    /// Move delivery to dead letter queue
    async fn move_to_dead_letter(&self, delivery: WebhookDelivery) -> Result<(), ApiError> {
        let dead_letter = WebhookDead {
            id: delivery.id,
            webhook_id: delivery.webhook_id,
            event: delivery.event,
            payload: delivery.payload,
            attempts: delivery.attempts,
            last_error: delivery.error_message,
            created_at: delivery.created_at,
            moved_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.index.create_webhook_dead(&dead_letter).await?;
        self.index.delete_webhook_delivery(delivery.id).await?;

        warn!("Webhook delivery moved to dead letter: {}", delivery.id);
        Ok(())
    }

    /// Process pending deliveries
    pub async fn process_pending_deliveries(&self) -> Result<(), ApiError> {
        let pending_deliveries = self.get_pending_deliveries().await?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for delivery in pending_deliveries {
            // Check if it's time to retry
            if let Some(next_retry) = delivery.next_retry_at {
                if next_retry > now {
                    continue;
                }
            }

            // Get webhook
            let webhook = self.index.get_webhook(delivery.webhook_id).await?;

            // Parse payload
            let payload: WebhookPayload = serde_json::from_value(delivery.payload)
                .map_err(|e| ApiError::InternalServerError(format!("Payload deserialization failed: {}", e)))?;

            // Attempt delivery
            let mut updated_delivery = delivery.clone();
            match self.attempt_delivery(&webhook, &payload, &mut updated_delivery).await {
                Ok(()) => {
                    updated_delivery.status = "delivered".to_string();
                    updated_delivery.updated_at = now;
                    self.index.update_webhook_delivery(&updated_delivery).await?;
                }
                Err(e) => {
                    updated_delivery.status = "failed".to_string();
                    updated_delivery.error_message = Some(e.to_string());
                    updated_delivery.updated_at = now;

                    if updated_delivery.attempts >= self.config.max_retries {
                        self.move_to_dead_letter(updated_delivery).await?;
                    } else {
                        // Schedule next retry
                        let delay = self.config.retry_delay_seconds * (2_u64.pow(updated_delivery.attempts));
                        updated_delivery.next_retry_at = Some(now + delay);
                        self.index.update_webhook_delivery(&updated_delivery).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl WebhookWorker {
    /// Create a new webhook worker
    pub fn new(client: WebhookClient) -> Self {
        Self {
            client,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the webhook worker
    pub async fn start(&self) -> Result<(), ApiError> {
        let mut running = self.running.write().await;
        if *running {
            return Err(ApiError::BadRequest("Webhook worker is already running".to_string()));
        }
        *running = true;
        drop(running);

        info!("Starting webhook worker");

        let worker = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(worker.client.config.delivery_interval_seconds));

            loop {
                if !*worker.running.read().await {
                    break;
                }

                interval.tick().await;

                if let Err(e) = worker.client.process_pending_deliveries().await {
                    error!("Webhook delivery processing failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Stop the webhook worker
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Stopping webhook worker");
    }
}

/// API handlers for webhook management

/// Create a new webhook
async fn create_webhook(
    State(index): State<Arc<IndexClient>>,
    Path(repo): Path<String>,
    auth: AuthContext,
    Json(payload): Json<CreateWebhookRequest>,
) -> Result<Json<ApiResponse<Webhook>>, ApiError> {
    // Validate URL
    let url = Url::parse(&payload.url)
        .map_err(|_| ApiError::BadRequest("Invalid webhook URL".to_string()))?;

    if !url.scheme().starts_with("http") {
        return Err(ApiError::BadRequest("Webhook URL must use HTTP or HTTPS".to_string()));
    }

    // Get repository
    let repo_info = index.get_repo_by_name(&repo).await?;

    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    // Create webhook
    let webhook = Webhook {
        id: Uuid::new_v4(),
        repo_id: repo_info.id,
        url: payload.url,
        secret: payload.secret,
        events: payload.events,
        active: true,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    index.create_webhook(&webhook).await?;

    // Log audit
    index.log_audit(
        &auth.sub,
        "webhook_created",
        Some(&repo),
        None,
        None,
        Some(&serde_json::json!({
            "webhook_id": webhook.id,
            "url": webhook.url,
            "events": webhook.events
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(webhook)))
}

/// Get webhooks for a repository
async fn get_webhooks(
    State(index): State<Arc<IndexClient>>,
    Path(repo): Path<String>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<Vec<Webhook>>>, ApiError> {
    // Get repository
    let repo_info = index.get_repo_by_name(&repo).await?;

    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    let webhooks = index.get_webhooks(repo_info.id).await?;
    Ok(Json(ApiResponse::success(webhooks)))
}

/// Delete a webhook
async fn delete_webhook(
    State(index): State<Arc<IndexClient>>,
    Path((repo, webhook_id)): Path<(String, Uuid)>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Get repository
    let repo_info = index.get_repo_by_name(&repo).await?;

    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    // Get webhook
    let webhook = index.get_webhook(webhook_id).await?;
    if webhook.repo_id != repo_info.id {
        return Err(ApiError::NotFound("Webhook not found".to_string()));
    }

    // Delete webhook
    index.delete_webhook(webhook_id).await?;

    // Log audit
    index.log_audit(
        &auth.sub,
        "webhook_deleted",
        Some(&repo),
        None,
        None,
        Some(&serde_json::json!({
            "webhook_id": webhook_id,
            "url": webhook.url
        })),
        None,
    ).await?;

    Ok(Json(ApiResponse::success(())))
}

/// Test webhook delivery
async fn test_webhook(
    State(client): State<WebhookClient>,
    Path((repo, webhook_id)): Path<(String, Uuid)>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<WebhookDelivery>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    // Get webhook
    let webhook = client.index.get_webhook(webhook_id).await?;

    // Create test payload
    let test_payload = WebhookPayload {
        event: WebhookEvent::Test,
        repo_id: webhook.repo_id,
        repo_name: repo,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        data: serde_json::json!({
            "message": "Test webhook delivery",
            "test": true
        }),
    };

    // Deliver webhook
    let delivery = client.deliver_webhook(&webhook, &test_payload).await?;

    Ok(Json(ApiResponse::success(delivery)))
}

/// Get webhook deliveries
async fn get_webhook_deliveries(
    State(index): State<Arc<IndexClient>>,
    Path((repo, webhook_id)): Path<(String, Uuid)>,
    auth: AuthContext,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<WebhookDelivery>>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    // Get webhook
    let webhook = index.get_webhook(webhook_id).await?;

    // Get deliveries (simplified - would need proper filtering)
    let deliveries = index.get_webhook_deliveries(webhook_id).await?;
    Ok(Json(ApiResponse::success(deliveries)))
}

/// Get dead letter webhooks
async fn get_dead_letter_webhooks(
    State(index): State<Arc<IndexClient>>,
    Path(repo): Path<String>,
    auth: AuthContext,
) -> Result<Json<ApiResponse<Vec<WebhookDead>>>, ApiError> {
    // Check permissions
    if !auth.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden("Admin role required".to_string()));
    }

    // Get repository
    let repo_info = index.get_repo_by_name(&repo).await?;

    // Get dead letter webhooks
    let dead_webhooks = index.get_webhook_dead_letter(repo_info.id).await?;
    Ok(Json(ApiResponse::success(dead_webhooks)))
}

/// Create webhook routes
pub fn create_webhook_routes() -> Router<Arc<IndexClient>> {
    Router::new()
        .route("/repos/:repo/webhooks", post(create_webhook))
        .route("/repos/:repo/webhooks", get(get_webhooks))
        .route("/repos/:repo/webhooks/:webhook_id", delete(delete_webhook))
        .route("/repos/:repo/webhooks/:webhook_id/test", post(test_webhook))
        .route("/repos/:repo/webhooks/:webhook_id/deliveries", get(get_webhook_deliveries))
        .route("/repos/:repo/webhooks/dead-letter", get(get_dead_letter_webhooks))
}

/// Request types
#[derive(Debug, serde::Deserialize)]
struct CreateWebhookRequest {
    url: String,
    secret: Option<String>,
    events: Vec<WebhookEvent>,
}
