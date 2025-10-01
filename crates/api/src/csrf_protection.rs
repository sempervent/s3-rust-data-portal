// CSRF Protection Implementation
// Implements CSRF protection using double-submit token pattern

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
pub struct CsrfToken {
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfConfig {
    pub secret_key: String,
    pub token_lifetime: Duration,
    pub cookie_name: String,
    pub header_name: String,
    pub require_https: bool,
    pub same_site: SameSitePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            secret_key: std::env::var("CSRF_SECRET_KEY").unwrap_or_else(|_| "default-secret-key".to_string()),
            token_lifetime: Duration::hours(1),
            cookie_name: "csrf-token".to_string(),
            header_name: "x-csrf-token".to_string(),
            require_https: true,
            same_site: SameSitePolicy::Strict,
        }
    }
}

pub struct CsrfProtection {
    config: CsrfConfig,
    tokens: Arc<RwLock<HashMap<String, CsrfToken>>>,
}

impl CsrfProtection {
    pub fn new(config: CsrfConfig) -> Self {
        Self {
            config,
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new CSRF token
    pub async fn generate_token(&self, user_id: Option<String>, session_id: Option<String>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let token_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Create HMAC signature
        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())?;
        mac.update(token_id.as_bytes());
        mac.update(now.timestamp().to_string().as_bytes());
        if let Some(ref uid) = user_id {
            mac.update(uid.as_bytes());
        }
        if let Some(ref sid) = session_id {
            mac.update(sid.as_bytes());
        }
        
        let signature = mac.finalize().into_bytes();
        let token = format!("{}.{}", token_id, general_purpose::STANDARD.encode(signature));
        
        let csrf_token = CsrfToken {
            token: token.clone(),
            created_at: now,
            expires_at: now + self.config.token_lifetime,
            user_id,
            session_id,
        };
        
        let mut tokens = self.tokens.write().await;
        tokens.insert(token_id, csrf_token);
        
        Ok(token)
    }

    /// Validate a CSRF token
    pub async fn validate_token(&self, token: &str, user_id: Option<String>, session_id: Option<String>) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 2 {
            return Ok(false);
        }
        
        let token_id = parts[0];
        let provided_signature = general_purpose::STANDARD.decode(parts[1])?;
        
        // Verify signature
        let mut mac = HmacSha256::new_from_slice(self.config.secret_key.as_bytes())?;
        mac.update(token_id.as_bytes());
        
        // Get token from storage to verify timestamp and user info
        let tokens = self.tokens.read().await;
        if let Some(stored_token) = tokens.get(token_id) {
            mac.update(stored_token.created_at.timestamp().to_string().as_bytes());
            if let Some(ref uid) = stored_token.user_id {
                mac.update(uid.as_bytes());
            }
            if let Some(ref sid) = stored_token.session_id {
                mac.update(sid.as_bytes());
            }
            
            let expected_signature = mac.finalize().into_bytes();
            
            // Check if token is expired
            if Utc::now() > stored_token.expired_at {
                return Ok(false);
            }
            
            // Check if user/session matches
            if stored_token.user_id != user_id || stored_token.session_id != session_id {
                return Ok(false);
            }
            
            // Verify signature
            if provided_signature == expected_signature {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired_tokens(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();
        let mut tokens = self.tokens.write().await;
        let initial_count = tokens.len();
        
        tokens.retain(|_, token| token.expires_at > now);
        
        Ok(initial_count - tokens.len())
    }

    /// Get token from request headers
    fn get_token_from_headers(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get(&self.config.header_name)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Get token from cookies
    fn get_token_from_cookies(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get("cookie")
            .and_then(|value| value.to_str().ok())
            .and_then(|cookie_str| {
                cookie_str
                    .split(';')
                    .find(|cookie| cookie.trim().starts_with(&format!("{}=", self.config.cookie_name)))
                    .and_then(|cookie| cookie.split('=').nth(1))
                    .map(|s| s.to_string())
            })
    }

    /// Extract user and session info from request
    fn extract_user_session(&self, headers: &HeaderMap) -> (Option<String>, Option<String>) {
        let user_id = headers
            .get("x-user-id")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string());
            
        let session_id = headers
            .get("x-session-id")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string());
            
        (user_id, session_id)
    }
}

/// CSRF middleware
pub async fn csrf_middleware(
    State(csrf_protection): State<Arc<CsrfProtection>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    let method = request.method();
    
    // Skip CSRF protection for safe methods
    if method == "GET" || method == "HEAD" || method == "OPTIONS" {
        return Ok(next.run(request).await);
    }
    
    // Extract user and session info
    let (user_id, session_id) = csrf_protection.extract_user_session(headers);
    
    // Get token from header or cookie
    let token = csrf_protection.get_token_from_headers(headers)
        .or_else(|| csrf_protection.get_token_from_cookies(headers));
    
    match token {
        Some(token) => {
            match csrf_protection.validate_token(&token, user_id, session_id).await {
                Ok(true) => Ok(next.run(request).await),
                Ok(false) => Err(StatusCode::FORBIDDEN),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        None => Err(StatusCode::FORBIDDEN),
    }
}

/// Generate CSRF token endpoint
pub async fn generate_csrf_token(
    State(csrf_protection): State<Arc<CsrfProtection>>,
    headers: HeaderMap,
) -> Result<axum::response::Json<serde_json::Value>, StatusCode> {
    let (user_id, session_id) = csrf_protection.extract_user_session(&headers);
    
    match csrf_protection.generate_token(user_id, session_id).await {
        Ok(token) => Ok(axum::response::Json(serde_json::json!({
            "csrf_token": token,
            "expires_in": csrf_protection.config.token_lifetime.num_seconds()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// CSRF protection router
pub fn csrf_router() -> axum::Router {
    axum::Router::new()
        .route("/csrf/token", axum::routing::get(generate_csrf_token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_generate_and_validate_token() {
        let config = CsrfConfig::default();
        let csrf = CsrfProtection::new(config);
        
        let user_id = Some("user123".to_string());
        let session_id = Some("session456".to_string());
        
        // Generate token
        let token = csrf.generate_token(user_id.clone(), session_id.clone()).await.unwrap();
        assert!(!token.is_empty());
        
        // Validate token
        let is_valid = csrf.validate_token(&token, user_id, session_id).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_token_expiration() {
        let mut config = CsrfConfig::default();
        config.token_lifetime = Duration::seconds(1);
        let csrf = CsrfProtection::new(config);
        
        let user_id = Some("user123".to_string());
        let session_id = Some("session456".to_string());
        
        // Generate token
        let token = csrf.generate_token(user_id.clone(), session_id.clone()).await.unwrap();
        
        // Wait for token to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Validate expired token
        let is_valid = csrf.validate_token(&token, user_id, session_id).await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_cleanup_expired_tokens() {
        let mut config = CsrfConfig::default();
        config.token_lifetime = Duration::seconds(1);
        let csrf = CsrfProtection::new(config);
        
        let user_id = Some("user123".to_string());
        let session_id = Some("session456".to_string());
        
        // Generate multiple tokens
        let _token1 = csrf.generate_token(user_id.clone(), session_id.clone()).await.unwrap();
        let _token2 = csrf.generate_token(user_id.clone(), session_id.clone()).await.unwrap();
        
        // Wait for tokens to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Cleanup expired tokens
        let cleaned_count = csrf.cleanup_expired_tokens().await.unwrap();
        assert!(cleaned_count > 0);
    }
}
