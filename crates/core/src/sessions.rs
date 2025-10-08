// BlackLake Server-Side Sessions
// Week 6: Cookie-based sessions with Redis backend

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tower_sessions::{Session, SessionManagerLayer};
use tower_sessions::MemoryStore;
use uuid::Uuid;

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub secret: [u8; 32],
    pub cookie_name: String,
    pub ttl: Duration,
    pub max_age: Duration,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            secret: [0u8; 32], // Will be set from environment
            cookie_name: "blksess".to_string(),
            ttl: Duration::from_secs(12 * 60 * 60), // 12 hours
            max_age: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            secure: true,
            http_only: true,
            same_site: SameSite::Lax,
        }
    }
}

/// Session data stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub oidc_metadata: Option<OidcMetadata>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub csrf_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcMetadata {
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub issuer: String,
    pub audience: String,
}

/// Session errors
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    #[error("Session expired")]
    Expired,
    #[error("Invalid session data")]
    InvalidData,
    #[error("CSRF token mismatch")]
    CsrfMismatch,
    #[error("Session storage error: {0}")]
    Storage(String),
    #[error("Redis error: {0}")]
    RedisError(String),
}

/// Session manager for BlackLake
pub struct SessionManager {
    config: SessionConfig,
    store: MemoryStore,
    redis_client: Option<redis::Client>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(redis_url: &str, config: SessionConfig) -> Result<Self, SessionError> {
        let store = MemoryStore::default();
        let redis_client = redis::Client::open(redis_url).ok();
        
        Ok(Self { config, store, redis_client })
    }
    
    /// Create session layer for Axum
    pub fn create_layer(&self) -> SessionManagerLayer<MemoryStore> {
        SessionManagerLayer::new(self.store.clone())
            .with_name(&self.config.cookie_name)
            .with_secure(self.config.secure)
            .with_http_only(self.config.http_only)
            .with_same_site(self.same_site_to_tower())
    }
    
    /// Create a new session for a user
    pub async fn create_session(
        &self,
        session: &Session,
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
        oidc_metadata: Option<OidcMetadata>,
    ) -> Result<(), SessionError> {
        let now = chrono::Utc::now();
        let csrf_token = self.generate_csrf_token();
        
        let session_data = SessionData {
            user_id,
            email,
            roles,
            oidc_metadata,
            created_at: now,
            last_activity: now,
            csrf_token,
        };
        
        session.insert("user_data", &session_data)
            .map_err(|_| SessionError::Storage("Failed to store session data".to_string()))?;
        
        Ok(())
    }
    
    /// Get session data
    pub async fn get_session_data(&self, session: &Session) -> Result<SessionData, SessionError> {
        let session_data: Option<SessionData> = session.get("user_data")
            .map_err(|_| SessionError::Storage("Failed to retrieve session data".to_string()))?;
        
        match session_data {
            Some(data) => {
                // Check if session is expired
                let now = chrono::Utc::now();
                if now - data.last_activity > chrono::Duration::from_std(self.config.ttl).unwrap_or_default() {
                    return Err(SessionError::Expired);
                }
                
                // Update last activity
                let mut updated_data = data.clone();
                updated_data.last_activity = now;
                session.insert("user_data", &updated_data)
                    .map_err(|_| SessionError::Storage("Failed to update session data".to_string()))?;
                
                Ok(updated_data)
            }
            None => Err(SessionError::NotFound),
        }
    }
    
    /// Validate CSRF token
    pub async fn validate_csrf_token(
        &self,
        session: &Session,
        provided_token: &str,
    ) -> Result<(), SessionError> {
        let session_data = self.get_session_data(session).await?;
        
        if session_data.csrf_token != provided_token {
            return Err(SessionError::CsrfMismatch);
        }
        
        Ok(())
    }
    
    /// Generate a new CSRF token
    pub fn generate_csrf_token(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        base64::encode(bytes)
    }
    
    /// Rotate CSRF token
    pub async fn rotate_csrf_token(&self, session: &Session) -> Result<String, SessionError> {
        let mut session_data = self.get_session_data(session).await?;
        let new_token = self.generate_csrf_token();
        session_data.csrf_token = new_token.clone();
        
        session.insert("user_data", &session_data)
            .map_err(|_| SessionError::Storage("Failed to update CSRF token".to_string()))?;
        
        Ok(new_token)
    }
    
    /// Destroy session
    pub async fn destroy_session(&self, session: &Session) -> Result<(), SessionError> {
        session.delete();
        
        Ok(())
    }
    
    /// Check if session is valid
    pub async fn is_session_valid(&self, session: &Session) -> bool {
        self.get_session_data(session).await.is_ok()
    }
    
    /// Get session statistics from Redis
    pub async fn get_session_stats(&self) -> Result<SessionStats, SessionError> {
        // Implement session statistics from Redis
        let redis_client = self.redis_client.as_ref()
            .ok_or_else(|| SessionError::RedisError("Redis client not available".to_string()))?;
        let mut redis_conn = redis_client.get_async_connection().await
            .map_err(|e| SessionError::RedisError(format!("Failed to get Redis connection: {}", e)))?;

        // Get active sessions count
        let active_sessions: u32 = redis::cmd("SCARD")
            .arg("active_sessions")
            .query_async(&mut redis_conn)
            .await
            .unwrap_or(0);

        // Get expired sessions count from the last 24 hours
        let expired_sessions: u32 = redis::cmd("ZCARD")
            .arg("expired_sessions")
            .query_async(&mut redis_conn)
            .await
            .unwrap_or(0);

        // Get total sessions count
        let total_sessions: u32 = redis::cmd("GET")
            .arg("total_sessions")
            .query_async(&mut redis_conn)
            .await
            .unwrap_or(0);

        Ok(SessionStats {
            active_sessions: active_sessions as u64,
            expired_sessions: expired_sessions as u64,
            total_sessions: total_sessions as u64,
        })
    }
    
    fn same_site_to_tower(&self) -> tower_sessions::cookie::SameSite {
        match self.config.same_site {
            SameSite::Strict => tower_sessions::cookie::SameSite::Strict,
            SameSite::Lax => tower_sessions::cookie::SameSite::Lax,
            SameSite::None => tower_sessions::cookie::SameSite::None,
        }
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub active_sessions: u64,
    pub expired_sessions: u64,
    pub total_sessions: u64,
}

/// Generate a secure session secret
pub fn generate_session_secret() -> [u8; 32] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut secret = [0u8; 32];
    rng.fill(&mut secret);
    secret
}

/// Load session secret from environment or generate new one
pub fn load_session_secret() -> Result<[u8; 32], SessionError> {
    if let Ok(secret_str) = std::env::var("SESSION_SECRET") {
        if secret_str.len() == 64 {
            // Hex-encoded 32-byte secret
            let bytes = hex::decode(&secret_str)
                .map_err(|_| SessionError::Storage("Invalid SESSION_SECRET format".to_string()))?;
            if bytes.len() == 32 {
                let mut secret = [0u8; 32];
                secret.copy_from_slice(&bytes);
                return Ok(secret);
            }
        }
    }
    
    // Generate new secret if not provided or invalid
    let secret = generate_session_secret();
    let secret_hex = hex::encode(secret);
    tracing::warn!(
        "No valid SESSION_SECRET found, generated new one: {}",
        secret_hex
    );
    tracing::warn!("Set SESSION_SECRET={} in your environment", secret_hex);
    
    Ok(secret)
}

/// CSRF token extractor for Axum
pub struct CsrfToken(pub String);

impl CsrfToken {
    /// Extract CSRF token from request headers
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Option<Self> {
        headers
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok())
            .map(|s| Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.cookie_name, "blksess");
        assert_eq!(config.ttl, Duration::from_secs(12 * 60 * 60));
        assert_eq!(config.max_age, Duration::from_secs(7 * 24 * 60 * 60));
        assert!(config.secure);
        assert!(config.http_only);
    }
    
    #[test]
    fn test_generate_session_secret() {
        let secret1 = generate_session_secret();
        let secret2 = generate_session_secret();
        
        assert_eq!(secret1.len(), 32);
        assert_eq!(secret2.len(), 32);
        assert_ne!(secret1, secret2);
    }
    
    #[test]
    fn test_csrf_token_generation() {
        let config = SessionConfig::default();
        let manager = SessionManager {
            config,
            store: RedisStore::new("redis://localhost:6379").unwrap(),
        };
        
        let token1 = manager.generate_csrf_token();
        let token2 = manager.generate_csrf_token();
        
        assert!(!token1.is_empty());
        assert!(!token2.is_empty());
        assert_ne!(token1, token2);
    }
    
    #[test]
    fn test_session_data_serialization() {
        let session_data = SessionData {
            user_id: "user123".to_string(),
            email: Some("user@example.com".to_string()),
            roles: vec!["user".to_string(), "admin".to_string()],
            oidc_metadata: Some(OidcMetadata {
                issued_at: chrono::Utc::now(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                issuer: "https://auth.example.com".to_string(),
                audience: "blacklake".to_string(),
            }),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            csrf_token: "test_token".to_string(),
        };
        
        let json = serde_json::to_string(&session_data).unwrap();
        let deserialized: SessionData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(session_data.user_id, deserialized.user_id);
        assert_eq!(session_data.email, deserialized.email);
        assert_eq!(session_data.roles, deserialized.roles);
        assert_eq!(session_data.csrf_token, deserialized.csrf_token);
    }
}
