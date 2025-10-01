// API Key Authentication for Service-to-Service Communication
// Implements API key authentication with scoped permissions

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub user_id: String,
    pub permissions: Vec<Permission>,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub rate_limit: Option<RateLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub actions: Vec<String>,
    pub conditions: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyUsage {
    pub key_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub timestamp: DateTime<Utc>,
    pub response_status: u16,
    pub response_time_ms: u64,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

pub struct ApiKeyService {
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    usage_log: Arc<RwLock<Vec<ApiKeyUsage>>>,
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiterState>>>,
}

#[derive(Debug, Clone)]
struct RateLimiterState {
    pub requests: Vec<DateTime<Utc>>,
    pub last_reset: DateTime<Utc>,
}

impl ApiKeyService {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            usage_log: Arc::new(RwLock::new(Vec::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new API key
    pub async fn generate_api_key(
        &self,
        name: String,
        description: Option<String>,
        user_id: String,
        permissions: Vec<Permission>,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
        rate_limit: Option<RateLimit>,
    ) -> Result<ApiKey, Box<dyn std::error::Error + Send + Sync>> {
        let key_id = Uuid::new_v4();
        let key_value = self.generate_key_value(&key_id, &user_id)?;
        
        let api_key = ApiKey {
            id: key_id,
            key: key_value.clone(),
            name,
            description,
            user_id,
            permissions,
            scopes,
            created_at: Utc::now(),
            expires_at,
            last_used_at: None,
            is_active: true,
            rate_limit,
        };
        
        let mut keys = self.keys.write().await;
        keys.insert(key_value, api_key.clone());
        
        Ok(api_key)
    }

    /// Generate key value with HMAC signature
    fn generate_key_value(&self, key_id: &Uuid, user_id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let secret = std::env::var("API_KEY_SECRET").unwrap_or_else(|_| "default-secret".to_string());
        let timestamp = Utc::now().timestamp();
        
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
        mac.update(key_id.as_bytes());
        mac.update(user_id.as_bytes());
        mac.update(timestamp.to_string().as_bytes());
        
        let signature = mac.finalize().into_bytes();
        let encoded_signature = general_purpose::STANDARD.encode(signature);
        
        Ok(format!("blk_{}.{}", key_id, encoded_signature))
    }

    /// Validate API key
    pub async fn validate_api_key(
        &self,
        key: &str,
        required_permission: &str,
        required_action: &str,
    ) -> Result<Option<ApiKey>, Box<dyn std::error::Error + Send + Sync>> {
        let keys = self.keys.read().await;
        
        if let Some(api_key) = keys.get(key) {
            // Check if key is active
            if !api_key.is_active {
                return Ok(None);
            }
            
            // Check if key is expired
            if let Some(expires_at) = api_key.expires_at {
                if Utc::now() > expires_at {
                    return Ok(None);
                }
            }
            
            // Check permissions
            let has_permission = api_key.permissions.iter().any(|perm| {
                perm.resource == required_permission && perm.actions.contains(&required_action.to_string())
            });
            
            if !has_permission {
                return Ok(None);
            }
            
            // Check rate limits
            if let Some(rate_limit) = &api_key.rate_limit {
                if !self.check_rate_limit(key, rate_limit).await? {
                    return Ok(None);
                }
            }
            
            // Log usage
            self.log_api_key_usage(api_key.id, "unknown", "unknown", 200, 0).await?;
            
            Ok(Some(api_key.clone()))
        } else {
            Ok(None)
        }
    }

    /// Check rate limits for API key
    async fn check_rate_limit(&self, key: &str, rate_limit: &RateLimit) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut rate_limiters = self.rate_limiters.write().await;
        let now = Utc::now();
        
        let limiter = rate_limiters.entry(key.to_string()).or_insert_with(|| RateLimiterState {
            requests: Vec::new(),
            last_reset: now,
        });
        
        // Clean old requests
        limiter.requests.retain(|&timestamp| now - timestamp < Duration::hours(24));
        
        // Check rate limits
        let minute_ago = now - Duration::minutes(1);
        let hour_ago = now - Duration::hours(1);
        let day_ago = now - Duration::days(1);
        
        let requests_last_minute = limiter.requests.iter().filter(|&&t| t > minute_ago).count() as u32;
        let requests_last_hour = limiter.requests.iter().filter(|&&t| t > hour_ago).count() as u32;
        let requests_last_day = limiter.requests.iter().filter(|&&t| t > day_ago).count() as u32;
        
        if requests_last_minute > rate_limit.requests_per_minute ||
           requests_last_hour > rate_limit.requests_per_hour ||
           requests_last_day > rate_limit.requests_per_day {
            return Ok(false);
        }
        
        // Add current request
        limiter.requests.push(now);
        
        Ok(true)
    }

    /// Log API key usage
    async fn log_api_key_usage(
        &self,
        key_id: Uuid,
        endpoint: &str,
        method: &str,
        response_status: u16,
        response_time_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let usage = ApiKeyUsage {
            key_id,
            endpoint: endpoint.to_string(),
            method: method.to_string(),
            timestamp: Utc::now(),
            response_status,
            response_time_ms,
            user_agent: None,
            ip_address: None,
        };
        
        let mut usage_log = self.usage_log.write().await;
        usage_log.push(usage);
        
        // Keep only last 10000 entries
        if usage_log.len() > 10000 {
            usage_log.drain(0..1000);
        }
        
        Ok(())
    }

    /// Get API key by ID
    pub async fn get_api_key(&self, key_id: Uuid) -> Result<Option<ApiKey>, Box<dyn std::error::Error + Send + Sync>> {
        let keys = self.keys.read().await;
        Ok(keys.values().find(|key| key.id == key_id).cloned())
    }

    /// List API keys for user
    pub async fn list_api_keys(&self, user_id: &str) -> Result<Vec<ApiKey>, Box<dyn std::error::Error + Send + Sync>> {
        let keys = self.keys.read().await;
        Ok(keys.values()
            .filter(|key| key.user_id == user_id)
            .cloned()
            .collect())
    }

    /// Revoke API key
    pub async fn revoke_api_key(&self, key_id: Uuid) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut keys = self.keys.write().await;
        
        for (key_value, api_key) in keys.iter_mut() {
            if api_key.id == key_id {
                api_key.is_active = false;
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Get usage statistics
    pub async fn get_usage_statistics(&self, key_id: Option<Uuid>) -> Result<UsageStatistics, Box<dyn std::error::Error + Send + Sync>> {
        let usage_log = self.usage_log.read().await;
        
        let filtered_usage: Vec<&ApiKeyUsage> = if let Some(key_id) = key_id {
            usage_log.iter().filter(|usage| usage.key_id == key_id).collect()
        } else {
            usage_log.iter().collect()
        };
        
        let total_requests = filtered_usage.len();
        let successful_requests = filtered_usage.iter().filter(|usage| usage.response_status < 400).count();
        let failed_requests = total_requests - successful_requests;
        
        let avg_response_time = if total_requests > 0 {
            filtered_usage.iter().map(|usage| usage.response_time_ms).sum::<u64>() as f64 / total_requests as f64
        } else {
            0.0
        };
        
        Ok(UsageStatistics {
            total_requests,
            successful_requests,
            failed_requests,
            success_rate: if total_requests > 0 { successful_requests as f64 / total_requests as f64 } else { 0.0 },
            avg_response_time_ms: avg_response_time,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
}

/// API key authentication middleware
pub async fn api_key_middleware(
    State(api_key_service): State<Arc<ApiKeyService>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    
    // Extract API key from header
    let api_key = headers
        .get("x-api-key")
        .or_else(|| headers.get("authorization"))
        .and_then(|value| value.to_str().ok())
        .and_then(|header_value| {
            if header_value.starts_with("Bearer ") {
                Some(header_value[7..].to_string())
            } else if header_value.starts_with("ApiKey ") {
                Some(header_value[8..].to_string())
            } else {
                Some(header_value.to_string())
            }
        });
    
    match api_key {
        Some(key) => {
            // For now, we'll use a generic permission check
            // In a real implementation, you'd extract the required permission from the route
            match api_key_service.validate_api_key(&key, "api", "read").await {
                Ok(Some(_)) => Ok(next.run(request).await),
                Ok(None) => Err(StatusCode::FORBIDDEN),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// API key management router
pub fn api_key_router() -> axum::Router {
    axum::Router::new()
        .route("/api-keys", axum::routing::post(create_api_key))
        .route("/api-keys", axum::routing::get(list_api_keys))
        .route("/api-keys/:id", axum::routing::get(get_api_key))
        .route("/api-keys/:id/revoke", axum::routing::post(revoke_api_key))
        .route("/api-keys/usage", axum::routing::get(get_usage_statistics))
}

/// Create API key
async fn create_api_key(
    State(service): State<Arc<ApiKeyService>>,
    axum::extract::Json(request): axum::extract::Json<CreateApiKeyRequest>,
) -> Result<axum::response::Json<ApiKey>, StatusCode> {
    let api_key = service.generate_api_key(
        request.name,
        request.description,
        request.user_id,
        request.permissions,
        request.scopes,
        request.expires_at,
        request.rate_limit,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(axum::response::Json(api_key))
}

/// List API keys
async fn list_api_keys(
    State(service): State<Arc<ApiKeyService>>,
    axum::extract::Query(params): axum::extract::Query<ListApiKeysQuery>,
) -> Result<axum::response::Json<Vec<ApiKey>>, StatusCode> {
    let api_keys = service.list_api_keys(&params.user_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(axum::response::Json(api_keys))
}

/// Get API key
async fn get_api_key(
    State(service): State<Arc<ApiKeyService>>,
    axum::extract::Path(key_id): axum::extract::Path<Uuid>,
) -> Result<axum::response::Json<ApiKey>, StatusCode> {
    let api_key = service.get_api_key(key_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(axum::response::Json(api_key))
}

/// Revoke API key
async fn revoke_api_key(
    State(service): State<Arc<ApiKeyService>>,
    axum::extract::Path(key_id): axum::extract::Path<Uuid>,
) -> Result<axum::response::Json<serde_json::Value>, StatusCode> {
    let success = service.revoke_api_key(key_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if success {
        Ok(axum::response::Json(serde_json::json!({
            "status": "success",
            "message": "API key revoked successfully"
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get usage statistics
async fn get_usage_statistics(
    State(service): State<Arc<ApiKeyService>>,
    axum::extract::Query(params): axum::extract::Query<UsageStatisticsQuery>,
) -> Result<axum::response::Json<UsageStatistics>, StatusCode> {
    let stats = service.get_usage_statistics(params.key_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(axum::response::Json(stats))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub user_id: String,
    pub permissions: Vec<Permission>,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit: Option<RateLimit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListApiKeysQuery {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatisticsQuery {
    pub key_id: Option<Uuid>,
}
