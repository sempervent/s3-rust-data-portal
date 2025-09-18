# BlackLake Session Management

## Overview

BlackLake implements secure server-side sessions using `tower-sessions` with Redis as the backing store. This provides a seamless web experience while maintaining security best practices.

## Architecture

### Session Storage

- **Backend**: Redis with key namespace `bl:sess:`
- **Format**: Encrypted JSON with HMAC-SHA256 signing
- **TTL**: 12 hours sliding window, 7 days maximum absolute
- **Cookie**: `blksess` with secure, httpOnly, sameSite=Lax

### Authentication Flow

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Browser   │    │   BlackLake  │    │   Keycloak  │
│             │    │     API      │    │             │
└─────────────┘    └──────────────┘    └─────────────┘
       │                   │                   │
       │ 1. Login Request  │                   │
       ├──────────────────►│                   │
       │                   │ 2. OIDC Auth     │
       │                   ├──────────────────►│
       │                   │ 3. JWT Token     │
       │                   │◄──────────────────┤
       │ 4. Session Cookie │                   │
       │◄──────────────────┤                   │
       │                   │                   │
       │ 5. API Requests   │                   │
       │    (with cookie)  │                   │
       ├──────────────────►│                   │
       │                   │ 6. Session Lookup│
       │                   │    (Redis)        │
       │                   │                   │
       │ 7. Response       │                   │
       │◄──────────────────┤                   │
```

## Session Data Structure

### AuthSession

```rust
pub struct AuthSession {
    pub sub: String,                    // User ID from OIDC
    pub email: String,                  // User email
    pub roles: Vec<String>,             // User roles/permissions
    pub oidc_token_metadata: Option<serde_json::Value>, // Token claims
    pub csrf_token: CSRFToken,          // CSRF protection token
}
```

### Session Storage Format

```json
{
  "auth_session": {
    "sub": "user-123",
    "email": "user@example.com", 
    "roles": ["user", "admin"],
    "oidc_token_metadata": {
      "iss": "http://keycloak:8080/realms/master",
      "aud": "blacklake",
      "exp": 1640995200
    },
    "csrf_token": "base64-encoded-random-token"
  }
}
```

## Security Features

### CSRF Protection

- **Double Submit Pattern**: CSRF token in both cookie and header
- **Header Name**: `x-csrf-token`
- **Validation**: Required for all state-changing requests (POST, PUT, DELETE)
- **Exemption**: GET and HEAD requests bypass CSRF check

### Session Security

- **Encryption**: Session data encrypted with 256-bit key
- **Signing**: HMAC-SHA256 prevents tampering
- **Secure Cookies**: HTTPS-only in production
- **HttpOnly**: Prevents XSS access to session cookies
- **SameSite**: Lax policy prevents CSRF attacks

### Key Management

- **Session Secret**: 256-bit (32-byte) base64-encoded key
- **Environment Variable**: `SESSION_SECRET`
- **Rotation**: Support for key rotation without session invalidation
- **Generation**: `openssl rand -base64 32` for new secrets

## API Endpoints

### Session Management

```http
# Create session (after OIDC login)
POST /v1/session/login
Content-Type: application/json
{
  "oidc_token": "jwt-token-here"
}

# Get CSRF token
GET /v1/csrf
Response: {
  "csrf_token": "base64-token"
}

# Logout (revoke session)
POST /v1/session/logout
x-csrf-token: base64-token
```

### Authentication Middleware

```rust
// Extract AuthContext from session
#[async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
    RedisStore: FromRef<S>,
{
    type Rejection = SessionError;
    
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract session from cookie
        // 2. Validate CSRF token for state-changing requests
        // 3. Return AuthContext with user info
    }
}
```

## Configuration

### Environment Variables

```bash
# Required
SESSION_SECRET=base64-encoded-32-byte-key
REDIS_URL=redis://redis:6379

# Optional
SESSION_TTL_HOURS=12
SESSION_MAX_AGE_DAYS=7
SESSION_COOKIE_NAME=blksess
```

### Session Layer Configuration

```rust
let session_layer = SessionManagerLayer::new(store)
    .with_cookie_name("blksess")
    .with_expiry(Expiry::OnInactivity(CookieDuration::hours(12)))
    .with_absolute_expiry(Expiry::OnApplicationClosure(CookieDuration::days(7)))
    .with_secure(true)                    // HTTPS only
    .with_http_only(true)                 // No JS access
    .with_same_site(SameSite::Lax)        // CSRF protection
    .with_signed(secret.as_bytes());      // HMAC signing
```

## Usage Examples

### React Frontend

```typescript
// Login flow
const login = async (oidcToken: string) => {
  const response = await fetch('/v1/session/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ oidc_token: oidcToken })
  });
  
  if (response.ok) {
    // Session cookie automatically set
    // Redirect to dashboard
  }
};

// API requests with CSRF protection
const apiRequest = async (url: string, options: RequestInit = {}) => {
  // Get CSRF token
  const csrfResponse = await fetch('/v1/csrf');
  const { csrf_token } = await csrfResponse.json();
  
  return fetch(url, {
    ...options,
    headers: {
      ...options.headers,
      'x-csrf-token': csrf_token
    }
  });
};

// Logout
const logout = async () => {
  await apiRequest('/v1/session/logout', { method: 'POST' });
  // Redirect to login page
};
```

### CLI/API Authentication

```bash
# CLI continues to use OIDC bearer tokens
export BLACKLAKE_TOKEN="eyJhbGciOiJSUzI1NiIs..."

# API requests with bearer token
curl -H "Authorization: Bearer $BLACKLAKE_TOKEN" \
     http://localhost:8080/v1/repos
```

## Monitoring & Observability

### Metrics

- **Session Count**: Active sessions in Redis
- **Session Duration**: Average and P95 session lifetime
- **CSRF Failures**: Failed CSRF token validations
- **Session Expiry**: Rate of session timeouts
- **Login Rate**: Successful vs failed logins

### Health Checks

```bash
# Check Redis connectivity
redis-cli -u $REDIS_URL ping

# Check session store
curl http://localhost:8080/v1/health/sessions

# Monitor session metrics
curl http://localhost:8080/metrics | grep session
```

## Troubleshooting

### Common Issues

1. **Session Not Persisting**: Check Redis connectivity and SESSION_SECRET
2. **CSRF Failures**: Verify token is sent in correct header
3. **Cookie Not Set**: Check secure flag and domain settings
4. **Session Expiry**: Verify TTL configuration and Redis persistence

### Debug Commands

```bash
# Check Redis session keys
redis-cli -u $REDIS_URL keys "bl:sess:*"

# Inspect session data
redis-cli -u $REDIS_URL get "bl:sess:session-id"

# Clear all sessions (emergency)
redis-cli -u $REDIS_URL flushdb
```

## Migration from JWT-Only

### Backward Compatibility

- **API Endpoints**: Continue to accept OIDC bearer tokens
- **CLI Tools**: No changes required
- **Web UI**: Migrate to session-based authentication
- **Gradual Rollout**: Feature flag for session vs JWT auth

### Migration Steps

1. **Deploy Session Support**: Add session layer to API
2. **Update Frontend**: Implement session-based auth flow
3. **Test Compatibility**: Verify both auth methods work
4. **Monitor Metrics**: Track session adoption and performance
5. **Deprecate JWT**: Eventually remove JWT support for web UI

## Security Considerations

### Best Practices

- **Regular Key Rotation**: Rotate SESSION_SECRET periodically
- **Session Monitoring**: Alert on unusual session patterns
- **Rate Limiting**: Prevent session creation abuse
- **Audit Logging**: Log all session events for security analysis

### Threat Mitigation

- **Session Hijacking**: Secure cookies and HTTPS prevent interception
- **CSRF Attacks**: Double submit tokens prevent cross-site requests
- **Session Fixation**: Generate new session ID on login
- **Brute Force**: Rate limiting and account lockout policies
