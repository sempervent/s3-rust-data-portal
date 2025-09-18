use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use blacklake_core::AuthContext;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,
    pub scope: Option<String>,
    pub groups: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct JwksKey {
    pub kid: String,
    pub kty: String,
    pub alg: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Clone)]
pub struct JwksResponse {
    pub keys: Vec<JwksKey>,
}

#[derive(Debug, Clone)]
pub struct OidcConfig {
    pub issuer: String,
    pub audience: String,
    pub jwks_uri: String,
    pub cache_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct CachedJwks {
    pub keys: HashMap<String, DecodingKey>,
    pub expires_at: SystemTime,
}

#[derive(Debug, Clone)]
pub struct AuthLayer {
    pub config: OidcConfig,
    pub jwks_cache: Arc<RwLock<Option<CachedJwks>>>,
    pub client: reqwest::Client,
}

impl AuthLayer {
    pub fn new(config: OidcConfig) -> Self {
        Self {
            config,
            jwks_cache: Arc::new(RwLock::new(None)),
            client: reqwest::Client::new(),
        }
    }

    pub async fn verify_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        // Decode header to get kid
        let header = jsonwebtoken::decode_header(token)
            .map_err(|_| AuthError::InvalidToken("Invalid JWT header".to_string()))?;

        let kid = header.kid.ok_or_else(|| {
            AuthError::InvalidToken("Missing key ID in JWT header".to_string())
        })?;

        // Get JWKS
        let jwks = self.get_jwks().await?;
        let decoding_key = jwks.get(&kid).ok_or_else(|| {
            AuthError::InvalidToken(format!("Key ID {} not found in JWKS", kid))
        })?;

        // Validate token
        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.set_required_spec_claims(&["sub", "iss", "aud", "exp", "iat"]);

        let token_data = decode::<JwtClaims>(token, decoding_key, &validation)
            .map_err(|e| AuthError::InvalidToken(format!("Token validation failed: {}", e)))?;

        // Check expiration
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if token_data.claims.exp < now {
            return Err(AuthError::ExpiredToken);
        }

        Ok(token_data.claims)
    }

    async fn get_jwks(&self) -> Result<HashMap<String, DecodingKey>, AuthError> {
        // Check cache first
        {
            let cache = self.jwks_cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.expires_at > SystemTime::now() {
                    return Ok(cached.keys.clone());
                }
            }
        }

        // Fetch fresh JWKS
        let jwks_response = self.fetch_jwks().await?;
        let mut keys = HashMap::new();

        for key in jwks_response.keys {
            if key.kty == "RSA" && key.alg == "RS256" {
                let decoding_key = DecodingKey::from_rsa_components(&key.n, &key.e)
                    .map_err(|e| AuthError::JwksError(format!("Invalid RSA key: {}", e)))?;
                keys.insert(key.kid, decoding_key);
            }
        }

        // Update cache
        let expires_at = SystemTime::now() + self.config.cache_ttl;
        let cached_jwks = CachedJwks { keys: keys.clone(), expires_at };
        {
            let mut cache = self.jwks_cache.write().await;
            *cache = Some(cached_jwks);
        }

        Ok(keys)
    }

    async fn fetch_jwks(&self) -> Result<JwksResponse, AuthError> {
        let response = self
            .client
            .get(&self.config.jwks_uri)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AuthError::JwksError(format!("Failed to fetch JWKS: {}", e)))?;

        if !response.status().is_success() {
            return Err(AuthError::JwksError(format!(
                "JWKS endpoint returned status: {}",
                response.status()
            )));
        }

        let jwks: JwksResponse = response
            .json()
            .await
            .map_err(|e| AuthError::JwksError(format!("Failed to parse JWKS: {}", e)))?;

        Ok(jwks)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Token expired")]
    ExpiredToken,
    #[error("JWKS error: {0}")]
    JwksError(String),
    #[error("Missing authorization header")]
    MissingAuth,
    #[error("Invalid authorization format")]
    InvalidAuthFormat,
}

impl axum::response::IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidToken(msg) => (StatusCode::UNAUTHORIZED, msg),
            AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "Token expired".to_string()),
            AuthError::JwksError(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            AuthError::MissingAuth => (StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()),
            AuthError::InvalidAuthFormat => (StatusCode::UNAUTHORIZED, "Invalid authorization format".to_string()),
        };

        (status, message).into_response()
    }
}

pub async fn auth_middleware(
    State(auth_layer): State<AuthLayer>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let headers = request.headers();
    
    // Extract token from Authorization header
    let auth_header = headers
        .get("Authorization")
        .ok_or(AuthError::MissingAuth)?
        .to_str()
        .map_err(|_| AuthError::InvalidAuthFormat)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidAuthFormat);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Verify token
    let claims = auth_layer.verify_token(token).await?;

    // Create auth context
    let auth_context = AuthContext {
        user_id: claims.sub.clone(),
        groups: claims.groups.unwrap_or_default(),
        scope: claims.scope.unwrap_or_default(),
    };

    // Add to request extensions
    request.extensions_mut().insert(auth_context);

    info!("Authenticated user: {}", claims.sub);

    Ok(next.run(request).await)
}

pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();

    // Add request ID to headers
    request.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    // Create tracing span
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %request.method(),
        uri = %request.uri()
    );

    let _enter = span.enter();

    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    response
}

pub fn create_auth_layer() -> Result<AuthLayer, AuthError> {
    let issuer = std::env::var("OIDC_ISSUER")
        .map_err(|_| AuthError::JwksError("OIDC_ISSUER not set".to_string()))?;
    
    let audience = std::env::var("OIDC_AUDIENCE")
        .map_err(|_| AuthError::JwksError("OIDC_AUDIENCE not set".to_string()))?;

    let jwks_uri = format!("{}/.well-known/jwks.json", issuer.trim_end_matches('/'));

    let config = OidcConfig {
        issuer,
        audience,
        jwks_uri,
        cache_ttl: Duration::from_secs(3600), // 1 hour
    };

    Ok(AuthLayer::new(config))
}
