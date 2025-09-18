use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use blacklake_core::AuthContext;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_governor::{
    governor::{
        clock::DefaultClock,
        state::{InMemoryState, NotKeyed},
        RateLimiter,
    },
    GovernorConfig, GovernorConfigBuilder,
};
use tracing::{error, warn, info};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub per_user_limit: u32,
    pub per_ip_limit: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: std::env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            burst_size: std::env::var("RATE_LIMIT_BURST_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20),
            per_user_limit: std::env::var("RATE_LIMIT_PER_USER")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            per_ip_limit: std::env::var("RATE_LIMIT_PER_IP")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(500),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRateLimit {
    pub user_id: String,
    pub requests: u32,
    pub window_start: Instant,
    pub last_request: Instant,
}

#[derive(Debug, Clone)]
pub struct IpRateLimit {
    pub ip: String,
    pub requests: u32,
    pub window_start: Instant,
    pub last_request: Instant,
}

#[derive(Debug, Clone)]
pub struct RateLimitState {
    pub config: RateLimitConfig,
    pub user_limits: Arc<RwLock<HashMap<String, UserRateLimit>>>,
    pub ip_limits: Arc<RwLock<HashMap<String, IpRateLimit>>>,
    pub global_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimitState {
    pub fn new(config: RateLimitConfig) -> Self {
        let global_limiter = Arc::new(
            GovernorConfigBuilder::default()
                .per_second(config.requests_per_minute / 60)
                .burst_size(config.burst_size)
                .finish()
                .unwrap()
        );

        Self {
            config,
            user_limits: Arc::new(RwLock::new(HashMap::new())),
            ip_limits: Arc::new(RwLock::new(HashMap::new())),
            global_limiter,
        }
    }

    pub async fn check_rate_limit(
        &self,
        user_id: Option<&str>,
        ip: &str,
    ) -> Result<(), RateLimitError> {
        let now = Instant::now();

        // Check global rate limit
        if self.global_limiter.check().is_err() {
            return Err(RateLimitError::GlobalLimitExceeded);
        }

        // Check per-user rate limit
        if let Some(user_id) = user_id {
            if let Err(_) = self.check_user_rate_limit(user_id, now).await {
                return Err(RateLimitError::UserLimitExceeded);
            }
        }

        // Check per-IP rate limit
        if let Err(_) = self.check_ip_rate_limit(ip, now).await {
            return Err(RateLimitError::IpLimitExceeded);
        }

        Ok(())
    }

    async fn check_user_rate_limit(&self, user_id: &str, now: Instant) -> Result<(), ()> {
        let mut user_limits = self.user_limits.write().await;
        
        if let Some(limit) = user_limits.get_mut(user_id) {
            // Reset window if needed
            if now.duration_since(limit.window_start) >= Duration::from_secs(60) {
                limit.requests = 0;
                limit.window_start = now;
            }

            if limit.requests >= self.config.per_user_limit {
                return Err(());
            }

            limit.requests += 1;
            limit.last_request = now;
        } else {
            // First request for this user
            user_limits.insert(
                user_id.to_string(),
                UserRateLimit {
                    user_id: user_id.to_string(),
                    requests: 1,
                    window_start: now,
                    last_request: now,
                },
            );
        }

        Ok(())
    }

    async fn check_ip_rate_limit(&self, ip: &str, now: Instant) -> Result<(), ()> {
        let mut ip_limits = self.ip_limits.write().await;
        
        if let Some(limit) = ip_limits.get_mut(ip) {
            // Reset window if needed
            if now.duration_since(limit.window_start) >= Duration::from_secs(60) {
                limit.requests = 0;
                limit.window_start = now;
            }

            if limit.requests >= self.config.per_ip_limit {
                return Err(());
            }

            limit.requests += 1;
            limit.last_request = now;
        } else {
            // First request from this IP
            ip_limits.insert(
                ip.to_string(),
                IpRateLimit {
                    ip: ip.to_string(),
                    requests: 1,
                    window_start: now,
                    last_request: now,
                },
            );
        }

        Ok(())
    }

    pub async fn cleanup_expired_limits(&self) {
        let now = Instant::now();
        let cleanup_threshold = Duration::from_secs(300); // 5 minutes

        // Cleanup user limits
        {
            let mut user_limits = self.user_limits.write().await;
            user_limits.retain(|_, limit| {
                now.duration_since(limit.last_request) < cleanup_threshold
            });
        }

        // Cleanup IP limits
        {
            let mut ip_limits = self.ip_limits.write().await;
            ip_limits.retain(|_, limit| {
                now.duration_since(limit.last_request) < cleanup_threshold
            });
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Global rate limit exceeded")]
    GlobalLimitExceeded,
    #[error("User rate limit exceeded")]
    UserLimitExceeded,
    #[error("IP rate limit exceeded")]
    IpLimitExceeded,
}

impl axum::response::IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            RateLimitError::GlobalLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Global rate limit exceeded",
            ),
            RateLimitError::UserLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "User rate limit exceeded",
            ),
            RateLimitError::IpLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "IP rate limit exceeded",
            ),
        };

        let response = serde_json::json!({
            "error": message,
            "retry_after": 60
        });

        (status, response).into_response()
    }
}

pub async fn rate_limit_middleware(
    State(rate_limit_state): State<RateLimitState>,
    mut request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    let headers = request.headers();
    
    // Extract user ID from auth context if available
    let user_id = request
        .extensions()
        .get::<AuthContext>()
        .map(|auth| auth.user_id.as_str());

    // Extract IP address
    let ip = extract_client_ip(headers)
        .unwrap_or_else(|| "unknown".to_string());

    // Check rate limits
    rate_limit_state.check_rate_limit(user_id, &ip).await?;

    // Add rate limit info to request extensions
    request.extensions_mut().insert(RateLimitInfo {
        user_id: user_id.map(|s| s.to_string()),
        ip,
    });

    Ok(next.run(request).await)
}

#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub user_id: Option<String>,
    pub ip: String,
}

fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Check X-Forwarded-For header first
    if let Some(forwarded) = headers.get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip) = forwarded_str.split(',').next() {
                return Some(ip.trim().to_string());
            }
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }

    // Check X-Client-IP header
    if let Some(client_ip) = headers.get("X-Client-IP") {
        if let Ok(ip_str) = client_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }

    None
}

pub fn create_rate_limit_config() -> RateLimitConfig {
    RateLimitConfig {
        requests_per_minute: std::env::var("RATE_LIMIT_RPM")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100),
        burst_size: std::env::var("RATE_LIMIT_BURST")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(20),
        per_user_limit: std::env::var("RATE_LIMIT_PER_USER")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000),
        per_ip_limit: std::env::var("RATE_LIMIT_PER_IP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(500),
    }
}

pub async fn start_rate_limit_cleanup(rate_limit_state: RateLimitState) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    
    loop {
        interval.tick().await;
        rate_limit_state.cleanup_expired_limits().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 100);
        assert_eq!(config.burst_size, 20);
        assert_eq!(config.per_user_limit, 1000);
        assert_eq!(config.per_ip_limit, 500);
    }

    #[tokio::test]
    async fn test_rate_limit_state_creation() {
        let config = RateLimitConfig::default();
        let state = RateLimitState::new(config);
        
        assert_eq!(state.user_limits.read().await.len(), 0);
        assert_eq!(state.ip_limits.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_rate_limit_check() {
        let config = RateLimitConfig {
            per_user_limit: 2,
            per_ip_limit: 3,
            ..Default::default()
        };
        let state = RateLimitState::new(config);
        
        // First request should succeed
        assert!(state.check_rate_limit(Some("user1"), "192.168.1.1").await.is_ok());
        
        // Second request should succeed
        assert!(state.check_rate_limit(Some("user1"), "192.168.1.1").await.is_ok());
        
        // Third request should fail (user limit exceeded)
        assert!(state.check_rate_limit(Some("user1"), "192.168.1.1").await.is_err());
        
        // Different user should still work
        assert!(state.check_rate_limit(Some("user2"), "192.168.1.1").await.is_ok());
    }

    #[test]
    fn test_extract_client_ip() {
        let mut headers = HeaderMap::new();
        
        // Test X-Forwarded-For
        headers.insert("X-Forwarded-For", "192.168.1.1, 10.0.0.1".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), Some("192.168.1.1".to_string()));
        
        // Test X-Real-IP
        headers.clear();
        headers.insert("X-Real-IP", "192.168.1.2".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), Some("192.168.1.2".to_string()));
        
        // Test X-Client-IP
        headers.clear();
        headers.insert("X-Client-IP", "192.168.1.3".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), Some("192.168.1.3".to_string()));
        
        // Test no IP headers
        headers.clear();
        assert_eq!(extract_client_ip(&headers), None);
    }
}
