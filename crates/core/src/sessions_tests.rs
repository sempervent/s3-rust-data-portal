#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_session_secret_generation() {
        let secret = SessionSecret::generate_new();
        assert!(!secret.is_empty());
        
        // Should be base64 encoded
        let decoded = general_purpose::STANDARD.decode(&secret).unwrap();
        assert_eq!(decoded.len(), 32); // 256-bit key
    }

    #[test]
    fn test_session_secret_from_env() {
        // Test with valid base64 key
        std::env::set_var("SESSION_SECRET", general_purpose::STANDARD.encode(vec![0u8; 32]));
        let secret = SessionSecret::from_env().unwrap();
        assert_eq!(secret.as_bytes().len(), 32);

        // Test with invalid base64
        std::env::set_var("SESSION_SECRET", "invalid-base64");
        let result = SessionSecret::from_env();
        assert!(result.is_err());

        // Test with wrong length
        std::env::set_var("SESSION_SECRET", general_purpose::STANDARD.encode(vec![0u8; 16]));
        let result = SessionSecret::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_csrf_token_creation() {
        let token1 = CSRFToken::new();
        let token2 = CSRFToken::new();
        
        assert_ne!(token1.as_str(), token2.as_str());
        assert!(!token1.as_str().is_empty());
        assert!(!token2.as_str().is_empty());
    }

    #[test]
    fn test_csrf_token_display() {
        let token = CSRFToken::new();
        let display = format!("{}", token);
        assert_eq!(display, token.as_str());
    }

    #[test]
    fn test_auth_session_creation() {
        let session = AuthSession::new(
            "user-123".to_string(),
            "user@example.com".to_string(),
            vec!["user".to_string(), "admin".to_string()],
            Some(json!({
                "iss": "http://keycloak:8080/realms/master",
                "aud": "blacklake"
            })),
        );

        assert_eq!(session.sub, "user-123");
        assert_eq!(session.email, "user@example.com");
        assert_eq!(session.roles.len(), 2);
        assert!(session.roles.contains(&"user".to_string()));
        assert!(session.roles.contains(&"admin".to_string()));
        assert!(session.oidc_token_metadata.is_some());
        assert!(!session.csrf_token.as_str().is_empty());
    }

    #[test]
    fn test_auth_session_default() {
        let session = AuthSession::default();
        assert_eq!(session.sub, "");
        assert_eq!(session.email, "");
        assert!(session.roles.is_empty());
        assert!(session.oidc_token_metadata.is_none());
        assert!(!session.csrf_token.as_str().is_empty());
    }

    #[test]
    fn test_auth_session_serialization() {
        let session = AuthSession::new(
            "user-123".to_string(),
            "user@example.com".to_string(),
            vec!["user".to_string()],
            None,
        );

        let serialized = serde_json::to_string(&session).unwrap();
        let deserialized: AuthSession = serde_json::from_str(&serialized).unwrap();

        assert_eq!(session.sub, deserialized.sub);
        assert_eq!(session.email, deserialized.email);
        assert_eq!(session.roles, deserialized.roles);
        assert_eq!(session.csrf_token.as_str(), deserialized.csrf_token.as_str());
    }

    #[test]
    fn test_session_error_display() {
        let errors = vec![
            SessionError::StoreError("Redis connection failed".to_string()),
            SessionError::Unauthorized,
            SessionError::CsrfMismatch,
            SessionError::ConfigurationError("Missing SESSION_SECRET".to_string()),
            SessionError::InternalError("Database error".to_string()),
        ];

        for error in errors {
            let error_string = format!("{}", error);
            assert!(!error_string.is_empty());
        }
    }

    #[test]
    fn test_session_error_into_response() {
        let error = SessionError::Unauthorized;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let error = SessionError::CsrfMismatch;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let error = SessionError::ConfigurationError("Test".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_csrf_header_constant() {
        assert_eq!(CSRF_HEADER, "x-csrf-token");
    }

    #[tokio::test]
    async fn test_session_manager_layer_creation() {
        // This test would require a real Redis client, so we'll just test the error cases
        let result = SessionManager::layer(tower_sessions_redis_store::fred::prelude::RedisClient::new(
            tower_sessions_redis_store::fred::prelude::RedisConfig::from_url("redis://invalid-url").unwrap(),
            None,
            None,
            None,
            6,
        )).await;
        
        // Should fail with invalid Redis URL
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_session_with_oidc_metadata() {
        let oidc_metadata = json!({
            "iss": "http://keycloak:8080/realms/master",
            "aud": "blacklake",
            "exp": 1640995200,
            "iat": 1640908800,
            "sub": "user-123",
            "preferred_username": "testuser",
            "email": "user@example.com",
            "groups": ["user", "admin"]
        });

        let session = AuthSession::new(
            "user-123".to_string(),
            "user@example.com".to_string(),
            vec!["user".to_string(), "admin".to_string()],
            Some(oidc_metadata.clone()),
        );

        assert_eq!(session.sub, "user-123");
        assert_eq!(session.email, "user@example.com");
        assert_eq!(session.roles.len(), 2);
        assert!(session.oidc_token_metadata.is_some());
        
        let metadata = session.oidc_token_metadata.unwrap();
        assert_eq!(metadata["iss"], "http://keycloak:8080/realms/master");
        assert_eq!(metadata["aud"], "blacklake");
        assert_eq!(metadata["sub"], "user-123");
    }

    #[test]
    fn test_auth_session_minimal() {
        let session = AuthSession::new(
            "user-123".to_string(),
            "user@example.com".to_string(),
            vec![],
            None,
        );

        assert_eq!(session.sub, "user-123");
        assert_eq!(session.email, "user@example.com");
        assert!(session.roles.is_empty());
        assert!(session.oidc_token_metadata.is_none());
        assert!(!session.csrf_token.as_str().is_empty());
    }
}
