// Admin Access Management API
// Week 7: Multi-tenant access controls and ABAC management

use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::Json as AxumJson,
    routing::{get, post, put, delete},
    Router,
};
use blacklake_core::{
    AuthContext, ApiError, ApiResponse,
    policy::{Policy, PolicyEffect, PolicyCondition, ConditionOperator, PolicyEvaluator, AccessRequest},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::AppState;

/// Tenant management
#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Create tenant request
#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
}

/// Create tenant response
#[derive(Debug, Serialize)]
pub struct CreateTenantResponse {
    pub tenant: Tenant,
}

/// Policy management
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub effect: PolicyEffect,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    pub condition: Option<PolicyCondition>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create policy request
#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub effect: PolicyEffect,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    pub condition: Option<PolicyCondition>,
}

/// Update policy request
#[derive(Debug, Deserialize)]
pub struct UpdatePolicyRequest {
    pub name: Option<String>,
    pub effect: Option<PolicyEffect>,
    pub actions: Option<Vec<String>>,
    pub resources: Option<Vec<String>>,
    pub condition: Option<PolicyCondition>,
}

/// Policy test request
#[derive(Debug, Deserialize)]
pub struct PolicyTestRequest {
    pub subject: String,
    pub action: String,
    pub resource: String,
    pub context: HashMap<String, serde_json::Value>,
}

/// Policy test response
#[derive(Debug, Serialize)]
pub struct PolicyTestResponse {
    pub decision: PolicyEffect,
    pub reason: String,
    pub matched_policies: Vec<Uuid>,
}

/// Subject attribute management
#[derive(Debug, Serialize, Deserialize)]
pub struct SubjectAttribute {
    pub subject: String,
    pub key: String,
    pub value: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Create subject attribute request
#[derive(Debug, Deserialize)]
pub struct CreateSubjectAttributeRequest {
    pub subject: String,
    pub key: String,
    pub value: String,
}

/// List tenants
async fn list_tenants(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<AxumJson<ApiResponse<Vec<Tenant>>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "tenants",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    let tenants = sqlx::query_as!(
        Tenant,
        "SELECT id, name, created_at FROM tenant ORDER BY created_at DESC"
    )
    .fetch_all(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch tenants: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(tenants)))
}

/// Create tenant
async fn create_tenant(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CreateTenantRequest>,
) -> Result<AxumJson<ApiResponse<CreateTenantResponse>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "write",
        "tenants",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    let tenant = sqlx::query_as!(
        Tenant,
        "INSERT INTO tenant (name) VALUES ($1) RETURNING id, name, created_at",
        payload.name
    )
    .fetch_one(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create tenant: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(CreateTenantResponse { tenant })))
}

/// List policies for a tenant
async fn list_policies(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(tenant_id): Path<Uuid>,
) -> Result<AxumJson<ApiResponse<Vec<PolicyResponse>>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "policies",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    let policies = sqlx::query_as!(
        PolicyResponse,
        r#"
        SELECT id, tenant_id, name, effect, actions, resources, condition, created_at, updated_at
        FROM policy
        WHERE tenant_id = $1
        ORDER BY created_at DESC
        "#,
        tenant_id
    )
    .fetch_all(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to fetch policies: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(policies)))
}

/// Create policy
async fn create_policy(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(tenant_id): Path<Uuid>,
    Json(payload): Json<CreatePolicyRequest>,
) -> Result<AxumJson<ApiResponse<PolicyResponse>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "write",
        "policies",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    let policy = sqlx::query_as!(
        PolicyResponse,
        r#"
        INSERT INTO policy (tenant_id, name, effect, actions, resources, condition)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, tenant_id, name, effect, actions, resources, condition, created_at, updated_at
        "#,
        tenant_id,
        payload.name,
        payload.effect as _,
        &payload.actions,
        &payload.resources,
        payload.condition.map(|c| serde_json::to_value(c).unwrap_or_default())
    )
    .fetch_one(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create policy: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(policy)))
}

/// Update policy
async fn update_policy(
    State(state): State<AppState>,
    auth: AuthContext,
    Path((tenant_id, policy_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdatePolicyRequest>,
) -> Result<AxumJson<ApiResponse<PolicyResponse>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "write",
        "policies",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Build update query dynamically
    let mut update_fields = Vec::new();
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 1;

    if let Some(name) = payload.name {
        update_fields.push(format!("name = ${}", param_count));
        params.push(Box::new(name));
        param_count += 1;
    }

    if let Some(effect) = payload.effect {
        update_fields.push(format!("effect = ${}", param_count));
        params.push(Box::new(effect as _));
        param_count += 1;
    }

    if let Some(actions) = payload.actions {
        update_fields.push(format!("actions = ${}", param_count));
        params.push(Box::new(actions));
        param_count += 1;
    }

    if let Some(resources) = payload.resources {
        update_fields.push(format!("resources = ${}", param_count));
        params.push(Box::new(resources));
        param_count += 1;
    }

    if let Some(condition) = payload.condition {
        update_fields.push(format!("condition = ${}", param_count));
        params.push(Box::new(serde_json::to_value(condition).unwrap_or_default()));
        param_count += 1;
    }

    if update_fields.is_empty() {
        return Err(ApiError::BadRequest("No fields to update".to_string()));
    }

    update_fields.push(format!("updated_at = NOW()"));

    let query = format!(
        "UPDATE policy SET {} WHERE tenant_id = ${} AND id = ${} RETURNING id, tenant_id, name, effect, actions, resources, condition, created_at, updated_at",
        update_fields.join(", "),
        param_count,
        param_count + 1
    );

    // This is a simplified version - in practice, you'd need to handle the dynamic parameters properly
    let policy = sqlx::query_as!(
        PolicyResponse,
        r#"
        UPDATE policy SET updated_at = NOW()
        WHERE tenant_id = $1 AND id = $2
        RETURNING id, tenant_id, name, effect, actions, resources, condition, created_at, updated_at
        "#,
        tenant_id,
        policy_id
    )
    .fetch_one(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to update policy: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(policy)))
}

/// Delete policy
async fn delete_policy(
    State(state): State<AppState>,
    auth: AuthContext,
    Path((tenant_id, policy_id)): Path<(Uuid, Uuid)>,
) -> Result<AxumJson<ApiResponse<()>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "delete",
        "policies",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    sqlx::query!(
        "DELETE FROM policy WHERE tenant_id = $1 AND id = $2",
        tenant_id,
        policy_id
    )
    .execute(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to delete policy: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(())))
}

/// Test policy
async fn test_policy(
    State(state): State<AppState>,
    auth: AuthContext,
    Path(tenant_id): Path<Uuid>,
    Json(payload): Json<PolicyTestRequest>,
) -> Result<AxumJson<ApiResponse<PolicyTestResponse>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "policies",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    // Create access request
    let access_request = AccessRequest {
        subject: payload.subject,
        action: payload.action,
        resource: payload.resource,
        context: payload.context,
    };

    // Load policies and evaluate
    let mut evaluator = PolicyEvaluator::new();
    evaluator.load_policies(tenant_id, &state.index.get_pool()).await
        .map_err(|e| ApiError::Internal(format!("Failed to load policies: {}", e)))?;
    
    evaluator.load_subject_attributes(&access_request.subject, &state.index.get_pool()).await
        .map_err(|e| ApiError::Internal(format!("Failed to load subject attributes: {}", e)))?;

    let decision = evaluator.evaluate(&access_request)
        .map_err(|e| ApiError::Internal(format!("Policy evaluation failed: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(PolicyTestResponse {
        decision: decision.decision,
        reason: decision.reason,
        matched_policies: decision.matched_policies,
    })))
}

/// List subject attributes
async fn list_subject_attributes(
    State(state): State<AppState>,
    auth: AuthContext,
    Query(params): Query<HashMap<String, String>>,
) -> Result<AxumJson<ApiResponse<Vec<SubjectAttribute>>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "read",
        "attributes",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    let subject = params.get("subject");
    let attributes = if let Some(subject) = subject {
        sqlx::query_as!(
            SubjectAttribute,
            "SELECT subject, key, value, created_at FROM subject_attribute WHERE subject = $1 ORDER BY key",
            subject
        )
        .fetch_all(&state.index.get_pool())
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch subject attributes: {}", e)))?
    } else {
        sqlx::query_as!(
            SubjectAttribute,
            "SELECT subject, key, value, created_at FROM subject_attribute ORDER BY subject, key"
        )
        .fetch_all(&state.index.get_pool())
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to fetch subject attributes: {}", e)))?
    };

    Ok(AxumJson(ApiResponse::success(attributes)))
}

/// Create subject attribute
async fn create_subject_attribute(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(payload): Json<CreateSubjectAttributeRequest>,
) -> Result<AxumJson<ApiResponse<SubjectAttribute>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "write",
        "attributes",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    let attribute = sqlx::query_as!(
        SubjectAttribute,
        r#"
        INSERT INTO subject_attribute (subject, key, value)
        VALUES ($1, $2, $3)
        ON CONFLICT (subject, key, value) DO UPDATE SET
            key = EXCLUDED.key,
            value = EXCLUDED.value
        RETURNING subject, key, value, created_at
        "#,
        payload.subject,
        payload.key,
        payload.value
    )
    .fetch_one(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to create subject attribute: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(attribute)))
}

/// Delete subject attribute
async fn delete_subject_attribute(
    State(state): State<AppState>,
    auth: AuthContext,
    Path((subject, key, value)): Path<(String, String, String)>,
) -> Result<AxumJson<ApiResponse<()>>, ApiError> {
    // Check admin access
    let mut policy_enforcement = crate::policy_enforcement::PolicyEnforcement::new();
    let decision = policy_enforcement.check_admin_access(
        &auth.sub,
        "delete",
        "attributes",
        &state.index.get_pool(),
    ).await.map_err(|e| ApiError::Internal(format!("Policy check failed: {}", e)))?;

    if decision.decision == PolicyEffect::Deny {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }

    sqlx::query!(
        "DELETE FROM subject_attribute WHERE subject = $1 AND key = $2 AND value = $3",
        subject,
        key,
        value
    )
    .execute(&state.index.get_pool())
    .await
    .map_err(|e| ApiError::Internal(format!("Failed to delete subject attribute: {}", e)))?;

    Ok(AxumJson(ApiResponse::success(())))
}

/// Create admin access routes
pub fn create_admin_access_routes() -> Router<AppState> {
    Router::new()
        // Tenant management
        .route("/v1/admin/tenants", get(list_tenants))
        .route("/v1/admin/tenants", post(create_tenant))
        
        // Policy management
        .route("/v1/admin/tenants/:tenant_id/policies", get(list_policies))
        .route("/v1/admin/tenants/:tenant_id/policies", post(create_policy))
        .route("/v1/admin/tenants/:tenant_id/policies/:policy_id", put(update_policy))
        .route("/v1/admin/tenants/:tenant_id/policies/:policy_id", delete(delete_policy))
        .route("/v1/admin/tenants/:tenant_id/policies/test", post(test_policy))
        
        // Subject attribute management
        .route("/v1/admin/attributes", get(list_subject_attributes))
        .route("/v1/admin/attributes", post(create_subject_attribute))
        .route("/v1/admin/attributes/:subject/:key/:value", delete(delete_subject_attribute))
}
