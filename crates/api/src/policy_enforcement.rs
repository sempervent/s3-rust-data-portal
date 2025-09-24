// Policy Enforcement Point (PEP) middleware for BlackLake API
// Week 7: Attribute-based access control enforcement

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use blacklake_core::{
    AuthContext,
};
use blacklake_core::policy::{PolicyEvaluator, AccessRequest, PolicyDecision, PolicyError};
use crate::ApiError;
use std::collections::HashMap;
use tower_sessions::Session;
use uuid::Uuid;
use crate::AppState;

/// Policy enforcement middleware
pub async fn policy_enforcement_middleware(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract authentication context
    let auth_context = request.extensions().get::<AuthContext>().cloned();
    
    if let Some(auth) = auth_context {
        // Extract request information
        let method = request.method().to_string();
        let path = request.uri().path().to_string();
        let action = map_method_to_action(&method, &path);
        let resource = extract_resource_from_path(&path);
        
        // Build access request context
        let mut context = HashMap::new();
        context.insert("method".to_string(), serde_json::Value::String(method));
        context.insert("path".to_string(), serde_json::Value::String(path.clone()));
        context.insert("user_agent".to_string(), 
            headers.get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| serde_json::Value::String(s.to_string()))
                .unwrap_or(serde_json::Value::Null));
        context.insert("ip_address".to_string(),
            headers.get("x-forwarded-for")
                .or_else(|| headers.get("x-real-ip"))
                .and_then(|h| h.to_str().ok())
                .map(|s| serde_json::Value::String(s.to_string()))
                .unwrap_or(serde_json::Value::Null));

        // Create access request
        let access_request = AccessRequest {
            subject: auth.sub.clone(),
            action,
            resource,
            context,
        };

        // Evaluate policy
        let mut evaluator = PolicyEvaluator::new();
        
        // Load policies for the user's tenant (if any)
        if let Some(tenant_id) = extract_tenant_from_path(&path) {
            if let Err(e) = evaluator.load_policies(tenant_id, &state.index.get_pool()).await {
                tracing::warn!("Failed to load policies for tenant {}: {}", tenant_id, e);
            }
        }

        // Load subject attributes
        if let Err(e) = evaluator.load_subject_attributes(&auth.sub, &state.index.get_pool()).await {
            tracing::warn!("Failed to load subject attributes for {}: {}", auth.sub, e);
        }

        // Evaluate access request
        match evaluator.evaluate(&access_request) {
            Ok(decision) => {
                // Add policy decision to request extensions for audit logging
                request.extensions_mut().insert(decision.clone());
                
                // Check if access is denied
                if decision.decision == blacklake_core::policy::PolicyEffect::Deny {
                    tracing::warn!(
                        "Access denied for {} to {}: {}",
                        auth.sub,
                        path,
                        decision.reason
                    );
                    
                    return Err(ApiError::Forbidden(format!(
                        "Access denied: {}",
                        decision.reason
                    )));
                }

                // Add policy ID to response headers for audit
                let mut response = next.run(request).await;
                if let Some(policy_id) = decision.policy_id {
                    response.headers_mut().insert(
                        "x-policy-id",
                        policy_id.to_string().parse().unwrap(),
                    );
                }
                
                Ok(response)
            }
            Err(e) => {
                tracing::error!("Policy evaluation failed: {}", e);
                Err(ApiError::Internal(format!("Policy evaluation failed: {}", e)))
            }
        }
    } else {
        // No authentication context - let other middleware handle this
        Ok(next.run(request).await)
    }
}

/// Map HTTP method and path to action
fn map_method_to_action(method: &str, path: &str) -> String {
    match method {
        "GET" => {
            if path.contains("/search") {
                "search".to_string()
            } else if path.contains("/export") {
                "export".to_string()
            } else {
                "read".to_string()
            }
        }
        "POST" => {
            if path.contains("/upload") {
                "upload".to_string()
            } else if path.contains("/commit") {
                "commit".to_string()
            } else if path.contains("/session") {
                "session".to_string()
            } else {
                "write".to_string()
            }
        }
        "PUT" => "write".to_string(),
        "PATCH" => "write".to_string(),
        "DELETE" => "delete".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Extract resource from path
fn extract_resource_from_path(path: &str) -> String {
    // Extract repo name and path from URL
    if let Some(repo_start) = path.find("/repos/") {
        let after_repos = &path[repo_start + 7..];
        if let Some(slash_pos) = after_repos.find('/') {
            let repo_name = &after_repos[..slash_pos];
            let resource_path = &after_repos[slash_pos + 1..];
            
            if resource_path.is_empty() {
                format!("repo:{}", repo_name)
            } else {
                format!("repo:{}/{}", repo_name, resource_path)
            }
        } else {
            format!("repo:{}", after_repos)
        }
    } else if path.contains("/admin") {
        "admin".to_string()
    } else if path.contains("/search") {
        "search".to_string()
    } else {
        "system".to_string()
    }
}

/// Extract tenant ID from path (if present)
fn extract_tenant_from_path(path: &str) -> Option<Uuid> {
    // For now, we'll use a simple approach - in a real implementation,
    // you might have tenant-specific paths or headers
    if let Some(tenant_start) = path.find("/tenants/") {
        let after_tenants = &path[tenant_start + 9..];
        if let Some(slash_pos) = after_tenants.find('/') {
            let tenant_str = &after_tenants[..slash_pos];
            Uuid::parse_str(tenant_str).ok()
        } else {
            Uuid::parse_str(after_tenants).ok()
        }
    } else {
        None
    }
}

/// Policy enforcement for specific endpoints
pub struct PolicyEnforcement {
    evaluator: PolicyEvaluator,
}

impl PolicyEnforcement {
    pub fn new() -> Self {
        Self {
            evaluator: PolicyEvaluator::new(),
        }
    }

    /// Check if a user can perform an action on a resource
    pub async fn check_access(
        &mut self,
        subject: &str,
        action: &str,
        resource: &str,
        context: HashMap<String, serde_json::Value>,
        db_pool: &sqlx::PgPool,
    ) -> Result<PolicyDecision, PolicyError> {
        // Load policies and attributes
        if let Some(tenant_id) = extract_tenant_from_path(resource) {
            self.evaluator.load_policies(tenant_id, db_pool).await?;
        }
        
        self.evaluator.load_subject_attributes(subject, db_pool).await?;

        // Create access request
        let request = AccessRequest {
            subject: subject.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            context,
        };

        // Evaluate
        self.evaluator.evaluate(&request)
    }

    /// Check if a user can access a repository
    pub async fn check_repo_access(
        &mut self,
        subject: &str,
        action: &str,
        repo_name: &str,
        path: Option<&str>,
        db_pool: &sqlx::PgPool,
    ) -> Result<PolicyDecision, PolicyError> {
        let resource = if let Some(p) = path {
            format!("repo:{}/{}", repo_name, p)
        } else {
            format!("repo:{}", repo_name)
        };

        let mut context = HashMap::new();
        context.insert("repo_name".to_string(), serde_json::Value::String(repo_name.to_string()));
        if let Some(p) = path {
            context.insert("path".to_string(), serde_json::Value::String(p.to_string()));
        }

        self.check_access(subject, action, &resource, context, db_pool).await
    }

    /// Check if a user can access admin functions
    pub async fn check_admin_access(
        &mut self,
        subject: &str,
        action: &str,
        admin_function: &str,
        db_pool: &sqlx::PgPool,
    ) -> Result<PolicyDecision, PolicyError> {
        let resource = format!("admin:{}", admin_function);
        
        let mut context = HashMap::new();
        context.insert("admin_function".to_string(), serde_json::Value::String(admin_function.to_string()));

        self.check_access(subject, action, &resource, context, db_pool).await
    }
}

impl Default for PolicyEnforcement {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create policy enforcement middleware
pub fn create_policy_enforcement_layer() -> axum::middleware::from_fn_with_state::<
    AppState,
    fn(State<AppState>, Session, HeaderMap, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, ApiError>> + Send + '_>>,
    AppState,
> {
    axum::middleware::from_fn_with_state(policy_enforcement_middleware)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_map_method_to_action() {
        assert_eq!(map_method_to_action("GET", "/repos/test"), "read");
        assert_eq!(map_method_to_action("GET", "/search"), "search");
        assert_eq!(map_method_to_action("POST", "/upload"), "upload");
        assert_eq!(map_method_to_action("POST", "/commit"), "commit");
        assert_eq!(map_method_to_action("DELETE", "/repos/test"), "delete");
    }

    #[test]
    fn test_extract_resource_from_path() {
        assert_eq!(extract_resource_from_path("/repos/test"), "repo:test");
        assert_eq!(extract_resource_from_path("/repos/test/data/file.csv"), "repo:test/data/file.csv");
        assert_eq!(extract_resource_from_path("/admin/users"), "admin");
        assert_eq!(extract_resource_from_path("/search"), "search");
        assert_eq!(extract_resource_from_path("/health"), "system");
    }

    #[test]
    fn test_extract_tenant_from_path() {
        assert_eq!(extract_tenant_from_path("/tenants/123e4567-e89b-12d3-a456-426614174000/repos"), 
                   Some(Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap()));
        assert_eq!(extract_tenant_from_path("/repos/test"), None);
        assert_eq!(extract_tenant_from_path("/admin"), None);
    }
}
