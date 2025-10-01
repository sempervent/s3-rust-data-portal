// Request Signing for Sensitive Operations
// Implements request signing using HMAC-SHA256 for sensitive operations

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
pub struct SignedRequest {
    pub request_id: Uuid,
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body_hash: String,
    pub timestamp: DateTime<Utc>,
    pub nonce: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningKey {
    pub id: Uuid,
    pub name: String,
    pub secret: String,
    pub user_id: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub usage_count: u64,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    pub secret_key: String,
    pub signature_lifetime: Duration,
    pub clock_skew_tolerance: Duration,
    pub required_headers: Vec<String>,
    pub sensitive_paths: Vec<String>,
}

impl Default for SigningConfig {
    fn default() -> Self {
        Self {
            secret_key: std::env::var("REQUEST_SIGNING_SECRET").unwrap_or_else(|_| "default-signing-secret".to_string()),
            signature_lifetime: Duration::minutes(5),
            clock_skew_tolerance: Duration::minutes(1),
            required_headers: vec![
                "x-request-id".to_string(),
                "x-timestamp".to_string(),
                "x-nonce".to_string(),
            ],
            sensitive_paths: vec![
                "/api/v1/admin".to_string(),
                "/api/v1/users".to_string(),
                "/api/v1/permissions".to_string(),
                "/api/v1/keys".to_string(),
            ],
        }
    }
}

pub struct RequestSigningService {
    config: SigningConfig,
    signing_keys: Arc<RwLock<HashMap<String, SigningKey>>>,
    used_nonces: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl RequestSigningService {
    pub fn new(config: SigningConfig) -> Self {
        Self {
            config,
            signing_keys: Arc::new(RwLock::new(HashMap::new())),
            used_nonces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a signing key
    pub async fn generate_signing_key(
        &self,
        name: String,
        user_id: String,
        permissions: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<SigningKey, Box<dyn std::error::Error + Send + Sync>> {
        let key_id = Uuid::new_v4();
        let secret = self.generate_secret(&key_id, &user_id)?;
        
        let signing_key = SigningKey {
            id: key_id,
            name,
            secret: secret.clone(),
            user_id,
            permissions,
            created_at: Utc::now(),
            expires_at,
            is_active: true,
            usage_count: 0,
            last_used_at: None,
        };
        
        let mut keys = self.signing_keys.write().await;
        keys.insert(secret, signing_key.clone());
        
        Ok(signing_key)
    }

    /// Generate secret for signing key
    fn generate_secret(&self, key_id: &Uuid, user_id: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())?;
        mac.update(key_id.as_bytes());
        mac.update(user_id.as_bytes());
        mac.update(Utc::now().timestamp().to_string().as_bytes());
        
        let signature = mac.finalize().into_bytes();
        Ok(format!("sk_{}.{}", key_id, general_purpose::STANDARD.encode(signature)))
    }

    /// Sign a request
    pub async fn sign_request(
        &self,
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
        body: &[u8],
        signing_key: &str,
    ) -> Result<SignedRequest, Box<dyn std::error::Error + Send + Sync>> {
        let request_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let nonce = Uuid::new_v4().to_string();
        
        // Calculate body hash
        let body_hash = self.calculate_body_hash(body);
        
        // Create signature string
        let signature_string = self.create_signature_string(
            method,
            path,
            headers,
            &body_hash,
            &timestamp,
            &nonce,
        );
        
        // Generate signature
        let signature = self.generate_signature(&signature_string, signing_key)?;
        
        Ok(SignedRequest {
            request_id,
            method: method.to_string(),
            path: path.to_string(),
            headers: headers.clone(),
            body_hash,
            timestamp,
            nonce,
            signature,
        })
    }

    /// Verify a signed request
    pub async fn verify_request(
        &self,
        method: &str,
        path: &str,
        headers: &HeaderMap,
        body: &[u8],
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Extract required headers
        let request_id = headers.get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .ok_or("Missing x-request-id header")?;
            
        let timestamp_str = headers.get("x-timestamp")
            .and_then(|h| h.to_str().ok())
            .ok_or("Missing x-timestamp header")?;
            
        let nonce = headers.get("x-nonce")
            .and_then(|h| h.to_str().ok())
            .ok_or("Missing x-nonce header")?;
            
        let signature = headers.get("x-signature")
            .and_then(|h| h.to_str().ok())
            .ok_or("Missing x-signature header")?;
            
        let signing_key = headers.get("x-signing-key")
            .and_then(|h| h.to_str().ok())
            .ok_or("Missing x-signing-key header")?;

        // Parse timestamp
        let timestamp = timestamp_str.parse::<i64>()
            .map_err(|_| "Invalid timestamp format")?;
        let timestamp = DateTime::from_timestamp(timestamp, 0)
            .ok_or("Invalid timestamp")?;

        // Check timestamp freshness
        let now = Utc::now();
        let time_diff = (now - timestamp).abs();
        if time_diff > self.config.signature_lifetime + self.config.clock_skew_tolerance {
            return Ok(false);
        }

        // Check nonce reuse
        if !self.check_nonce(nonce).await? {
            return Ok(false);
        }

        // Verify signing key
        let signing_key_data = self.get_signing_key(signing_key).await?;
        if !signing_key_data.is_active {
            return Ok(false);
        }

        if let Some(expires_at) = signing_key_data.expires_at {
            if now > expires_at {
                return Ok(false);
            }
        }

        // Calculate body hash
        let body_hash = self.calculate_body_hash(body);

        // Create signature string
        let mut header_map = HashMap::new();
        for (key, value) in headers.iter() {
            if let Ok(key_str) = key.as_str() {
                if let Ok(value_str) = value.to_str() {
                    header_map.insert(key_str.to_string(), value_str.to_string());
                }
            }
        }

        let signature_string = self.create_signature_string(
            method,
            path,
            &header_map,
            &body_hash,
            &timestamp,
            nonce,
        );

        // Verify signature
        let expected_signature = self.generate_signature(&signature_string, &signing_key_data.secret)?;
        
        if signature == expected_signature {
            // Record nonce usage
            self.record_nonce(nonce).await?;
            
            // Update signing key usage
            self.update_signing_key_usage(signing_key).await?;
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Calculate body hash
    fn calculate_body_hash(&self, body: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(body);
        let hash = hasher.finalize();
        general_purpose::STANDARD.encode(hash)
    }

    /// Create signature string
    fn create_signature_string(
        &self,
        method: &str,
        path: &str,
        headers: &HashMap<String, String>,
        body_hash: &str,
        timestamp: &DateTime<Utc>,
        nonce: &str,
    ) -> String {
        let mut parts = Vec::new();
        parts.push(method.to_uppercase());
        parts.push(path);
        
        // Add headers in sorted order
        let mut sorted_headers: Vec<_> = headers.iter().collect();
        sorted_headers.sort_by(|a, b| a.0.cmp(b.0));
        
        for (key, value) in sorted_headers {
            parts.push(format!("{}:{}", key, value));
        }
        
        parts.push(format!("body:{}", body_hash));
        parts.push(format!("timestamp:{}", timestamp.timestamp()));
        parts.push(format!("nonce:{}", nonce));
        
        parts.join("\n")
    }

    /// Generate signature
    fn generate_signature(&self, data: &str, secret: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;
        mac.update(data.as_bytes());
        let signature = mac.finalize().into_bytes();
        Ok(general_purpose::STANDARD.encode(signature))
    }

    /// Check nonce reuse
    async fn check_nonce(&self, nonce: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let used_nonces = self.used_nonces.read().await;
        Ok(!used_nonces.contains_key(nonce))
    }

    /// Record nonce usage
    async fn record_nonce(&self, nonce: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut used_nonces = self.used_nonces.write().await;
        used_nonces.insert(nonce.to_string(), Utc::now());
        
        // Clean up old nonces
        let cutoff = Utc::now() - Duration::hours(1);
        used_nonces.retain(|_, &mut timestamp| timestamp > cutoff);
        
        Ok(())
    }

    /// Get signing key
    async fn get_signing_key(&self, key: &str) -> Result<SigningKey, Box<dyn std::error::Error + Send + Sync>> {
        let keys = self.signing_keys.read().await;
        keys.get(key)
            .cloned()
            .ok_or("Signing key not found".into())
    }

    /// Update signing key usage
    async fn update_signing_key_usage(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut keys = self.signing_keys.write().await;
        if let Some(signing_key) = keys.get_mut(key) {
            signing_key.usage_count += 1;
            signing_key.last_used_at = Some(Utc::now());
        }
        Ok(())
    }

    /// Check if path requires signing
    pub fn requires_signing(&self, path: &str) -> bool {
        self.config.sensitive_paths.iter().any(|sensitive_path| {
            path.starts_with(sensitive_path)
        })
    }

    /// Clean up expired nonces
    pub async fn cleanup_expired_nonces(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let cutoff = Utc::now() - Duration::hours(1);
        let mut used_nonces = self.used_nonces.write().await;
        let initial_count = used_nonces.len();
        
        used_nonces.retain(|_, &mut timestamp| timestamp > cutoff);
        
        Ok(initial_count - used_nonces.len())
    }
}

/// Request signing middleware
pub async fn request_signing_middleware(
    State(signing_service): State<Arc<RequestSigningService>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    // Check if path requires signing
    if !signing_service.requires_signing(path) {
        return Ok(next.run(request).await);
    }
    
    let headers = request.headers();
    let method = request.method().as_str();
    
    // For now, we'll simulate body extraction
    // In a real implementation, you'd need to handle body extraction carefully
    let body = b"";
    
    match signing_service.verify_request(method, path, headers, body).await {
        Ok(true) => Ok(next.run(request).await),
        Ok(false) => Err(StatusCode::FORBIDDEN),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Request signing router
pub fn request_signing_router() -> axum::Router {
    axum::Router::new()
        .route("/signing-keys", axum::routing::post(create_signing_key))
        .route("/signing-keys", axum::routing::get(list_signing_keys))
        .route("/signing-keys/:id", axum::routing::get(get_signing_key))
        .route("/signing-keys/:id/revoke", axum::routing::post(revoke_signing_key))
        .route("/signing-keys/cleanup", axum::routing::post(cleanup_expired_nonces))
}

/// Create signing key
async fn create_signing_key(
    State(service): State<Arc<RequestSigningService>>,
    axum::extract::Json(request): axum::extract::Json<CreateSigningKeyRequest>,
) -> Result<axum::response::Json<SigningKey>, StatusCode> {
    let signing_key = service.generate_signing_key(
        request.name,
        request.user_id,
        request.permissions,
        request.expires_at,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(axum::response::Json(signing_key))
}

/// List signing keys
async fn list_signing_keys(
    State(service): State<Arc<RequestSigningService>>,
    axum::extract::Query(params): axum::extract::Query<ListSigningKeysQuery>,
) -> Result<axum::response::Json<Vec<SigningKey>>, StatusCode> {
    // In a real implementation, you'd filter by user_id
    let keys = service.signing_keys.read().await;
    let user_keys: Vec<SigningKey> = keys.values()
        .filter(|key| key.user_id == params.user_id)
        .cloned()
        .collect();
    
    Ok(axum::response::Json(user_keys))
}

/// Get signing key
async fn get_signing_key(
    State(service): State<Arc<RequestSigningService>>,
    axum::extract::Path(key_id): axum::extract::Path<Uuid>,
) -> Result<axum::response::Json<SigningKey>, StatusCode> {
    let keys = service.signing_keys.read().await;
    let signing_key = keys.values()
        .find(|key| key.id == key_id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    
    Ok(axum::response::Json(signing_key))
}

/// Revoke signing key
async fn revoke_signing_key(
    State(service): State<Arc<RequestSigningService>>,
    axum::extract::Path(key_id): axum::extract::Path<Uuid>,
) -> Result<axum::response::Json<serde_json::Value>, StatusCode> {
    let mut keys = service.signing_keys.write().await;
    
    for signing_key in keys.values_mut() {
        if signing_key.id == key_id {
            signing_key.is_active = false;
            return Ok(axum::response::Json(serde_json::json!({
                "status": "success",
                "message": "Signing key revoked successfully"
            })));
        }
    }
    
    Err(StatusCode::NOT_FOUND)
}

/// Cleanup expired nonces
async fn cleanup_expired_nonces(
    State(service): State<Arc<RequestSigningService>>,
) -> Result<axum::response::Json<serde_json::Value>, StatusCode> {
    let cleaned_count = service.cleanup_expired_nonces().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(axum::response::Json(serde_json::json!({
        "status": "success",
        "cleaned_count": cleaned_count
    })))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSigningKeyRequest {
    pub name: String,
    pub user_id: String,
    pub permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSigningKeysQuery {
    pub user_id: String,
}
