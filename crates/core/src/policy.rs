// BlackLake Policy Evaluation Engine (PEP)
// Week 7: Attribute-based access control with multi-tenant support

use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

/// Policy effect (allow or deny)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

impl FromStr for PolicyEffect {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "allow" => Ok(PolicyEffect::Allow),
            "deny" => Ok(PolicyEffect::Deny),
            _ => Err(()),
        }
    }
}

/// Policy condition operators
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    In,
    NotIn,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Regex,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

/// Access control policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub effect: PolicyEffect,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    pub condition: Option<PolicyCondition>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for Policy {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Policy {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            name: row.get("name"),
            effect: PolicyEffect::from_str(&row.get::<String, _>("effect")).unwrap_or(PolicyEffect::Deny),
            actions: serde_json::from_str(&row.get::<String, _>("actions")).unwrap_or_default(),
            resources: serde_json::from_str(&row.get::<String, _>("resources")).unwrap_or_default(),
            condition: row.get::<Option<String>, _>("condition")
                .and_then(|s| serde_json::from_str(&s).ok()),
        })
    }
}

/// Subject attributes for policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectAttributes {
    pub subject: String,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Access request context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    pub subject: String,
    pub action: String,
    pub resource: String,
    pub context: HashMap<String, serde_json::Value>,
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub decision: PolicyEffect,
    pub policy_id: Option<Uuid>,
    pub reason: String,
    pub matched_policies: Vec<Uuid>,
}

/// Policy evaluation errors
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Policy evaluation failed: {0}")]
    EvaluationError(String),
    #[error("Invalid condition: {0}")]
    InvalidCondition(String),
    #[error("Missing attribute: {0}")]
    MissingAttribute(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Policy Evaluation Point (PEP)
pub struct PolicyEvaluator {
    policies: Vec<Policy>,
    subject_attributes: HashMap<String, SubjectAttributes>,
}

impl PolicyEvaluator {
    /// Create a new policy evaluator
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            subject_attributes: HashMap::new(),
        }
    }

    /// Load policies for a tenant
    pub async fn load_policies(&mut self, tenant_id: Uuid, db_pool: &sqlx::PgPool) -> Result<(), PolicyError> {
        let policies = sqlx::query_as::<_, Policy>(
            r#"
            SELECT id, tenant_id, name, effect, actions, resources, condition
            FROM policies
            WHERE tenant_id = $1
            ORDER BY created_at ASC
            "#
        )
        .bind(tenant_id)
        .fetch_all(db_pool)
        .await?;

        self.policies = policies;
        Ok(())
    }

    /// Load subject attributes
    pub async fn load_subject_attributes(&mut self, subject: &str, db_pool: &sqlx::PgPool) -> Result<(), PolicyError> {
        let attributes = sqlx::query(
            r#"
            SELECT key, value
            FROM subject_attributes
            WHERE subject = $1
            "#
        )
        .bind(subject)
        .fetch_all(db_pool)
        .await?;

        let mut attr_map = HashMap::new();
        for attr in attributes {
            let value: serde_json::Value = serde_json::from_str(&attr.get::<String, _>("value"))?;
            attr_map.insert(attr.get::<String, _>("key"), value);
        }

        self.subject_attributes.insert(
            subject.to_string(),
            SubjectAttributes {
                subject: subject.to_string(),
                attributes: attr_map,
            },
        );

        Ok(())
    }

    /// Evaluate an access request
    pub fn evaluate(&self, request: &AccessRequest) -> Result<PolicyDecision, PolicyError> {
        let mut matched_policies = Vec::new();
        let mut deny_policies = Vec::new();
        let mut allow_policies = Vec::new();

        // Check each policy
        for policy in &self.policies {
            if self.matches_policy(policy, request)? {
                matched_policies.push(policy.id);
                
                match policy.effect {
                    PolicyEffect::Allow => allow_policies.push(policy.id),
                    PolicyEffect::Deny => deny_policies.push(policy.id),
                }
            }
        }

        // Deny takes precedence over allow
        if !deny_policies.is_empty() {
            return Ok(PolicyDecision {
                decision: PolicyEffect::Deny,
                policy_id: Some(deny_policies[0]),
                reason: format!("Denied by {} policy(ies)", deny_policies.len()),
                matched_policies,
            });
        }

        if !allow_policies.is_empty() {
            return Ok(PolicyDecision {
                decision: PolicyEffect::Allow,
                policy_id: Some(allow_policies[0]),
                reason: format!("Allowed by {} policy(ies)", allow_policies.len()),
                matched_policies,
            });
        }

        // Default deny
        Ok(PolicyDecision {
            decision: PolicyEffect::Deny,
            policy_id: None,
            reason: "No matching policies found".to_string(),
            matched_policies,
        })
    }

    /// Check if a policy matches the request
    fn matches_policy(&self, policy: &Policy, request: &AccessRequest) -> Result<bool, PolicyError> {
        // Check actions
        if !policy.actions.is_empty() && !policy.actions.contains(&request.action) {
            return Ok(false);
        }

        // Check resources
        if !policy.resources.is_empty() {
            let mut resource_matches = false;
            for resource_pattern in &policy.resources {
                if self.matches_resource_pattern(resource_pattern, &request.resource) {
                    resource_matches = true;
                    break;
                }
            }
            if !resource_matches {
                return Ok(false);
            }
        }

        // Check condition
        if let Some(condition) = &policy.condition {
            if !self.evaluate_condition(condition, request)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a resource matches a pattern (supports wildcards and path prefixes)
    fn matches_resource_pattern(&self, pattern: &str, resource: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            return resource.starts_with(prefix);
        }

        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            return resource.starts_with(prefix) && !resource[prefix.len()..].contains('/');
        }

        pattern == resource
    }

    /// Evaluate a policy condition
    fn evaluate_condition(&self, condition: &PolicyCondition, request: &AccessRequest) -> Result<bool, PolicyError> {
        let value = self.get_field_value(&condition.field, request)?;
        
        match condition.operator {
            ConditionOperator::Equals => Ok(value == condition.value),
            ConditionOperator::NotEquals => Ok(value != condition.value),
            ConditionOperator::In => {
                if let Some(array) = condition.value.as_array() {
                    Ok(array.contains(&value))
                } else {
                    Err(PolicyError::InvalidCondition("IN operator requires array value".to_string()))
                }
            }
            ConditionOperator::NotIn => {
                if let Some(array) = condition.value.as_array() {
                    Ok(!array.contains(&value))
                } else {
                    Err(PolicyError::InvalidCondition("NOT IN operator requires array value".to_string()))
                }
            }
            ConditionOperator::Contains => {
                if let Some(subject_attrs) = self.subject_attributes.get(&request.subject) {
                    if let Some(attr_value) = subject_attrs.attributes.get(&condition.field) {
                        if let Some(array) = attr_value.as_array() {
                            return Ok(array.contains(&condition.value));
                        }
                    }
                }
                Ok(false)
            }
            ConditionOperator::NotContains => {
                if let Some(subject_attrs) = self.subject_attributes.get(&request.subject) {
                    if let Some(attr_value) = subject_attrs.attributes.get(&condition.field) {
                        if let Some(array) = attr_value.as_array() {
                            return Ok(!array.contains(&condition.value));
                        }
                    }
                }
                Ok(true)
            }
            ConditionOperator::StartsWith => {
                if let (Some(str_value), Some(str_condition)) = (value.as_str(), condition.value.as_str()) {
                    Ok(str_value.starts_with(str_condition))
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::EndsWith => {
                if let (Some(str_value), Some(str_condition)) = (value.as_str(), condition.value.as_str()) {
                    Ok(str_value.ends_with(str_condition))
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::Regex => {
                if let (Some(str_value), Some(str_condition)) = (value.as_str(), condition.value.as_str()) {
                    let regex = regex::Regex::new(str_condition)
                        .map_err(|e| PolicyError::InvalidCondition(format!("Invalid regex: {}", e)))?;
                    Ok(regex.is_match(str_value))
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::GreaterThan => {
                if let (Some(num_value), Some(num_condition)) = (value.as_f64(), condition.value.as_f64()) {
                    Ok(num_value > num_condition)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::LessThan => {
                if let (Some(num_value), Some(num_condition)) = (value.as_f64(), condition.value.as_f64()) {
                    Ok(num_value < num_condition)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::GreaterThanOrEqual => {
                if let (Some(num_value), Some(num_condition)) = (value.as_f64(), condition.value.as_f64()) {
                    Ok(num_value >= num_condition)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::LessThanOrEqual => {
                if let (Some(num_value), Some(num_condition)) = (value.as_f64(), condition.value.as_f64()) {
                    Ok(num_value <= num_condition)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Get field value from request context or subject attributes
    fn get_field_value(&self, field: &str, request: &AccessRequest) -> Result<serde_json::Value, PolicyError> {
        // Check request context first
        if let Some(value) = request.context.get(field) {
            return Ok(value.clone());
        }

        // Check subject attributes
        if let Some(subject_attrs) = self.subject_attributes.get(&request.subject) {
            if let Some(value) = subject_attrs.attributes.get(field) {
                return Ok(value.clone());
            }
        }

        // Check built-in fields
        match field {
            "subject" => Ok(serde_json::Value::String(request.subject.clone())),
            "action" => Ok(serde_json::Value::String(request.action.clone())),
            "resource" => Ok(serde_json::Value::String(request.resource.clone())),
            _ => Err(PolicyError::MissingAttribute(field.to_string())),
        }
    }

    /// Create a default policy evaluator with basic policies
    pub fn create_default() -> Self {
        let mut evaluator = Self::new();
        
        // Add default policies (these would normally be loaded from database)
        evaluator.policies = vec![
            Policy {
                id: Uuid::new_v4(),
                tenant_id: Uuid::nil(),
                name: "default-allow-read".to_string(),
                effect: PolicyEffect::Allow,
                actions: vec!["read".to_string(), "search".to_string()],
                resources: vec!["*".to_string()],
                condition: Some(PolicyCondition {
                    field: "subject.roles".to_string(),
                    operator: ConditionOperator::Contains,
                    value: serde_json::Value::String("user".to_string()),
                }),
            },
            Policy {
                id: Uuid::new_v4(),
                tenant_id: Uuid::nil(),
                name: "default-allow-write".to_string(),
                effect: PolicyEffect::Allow,
                actions: vec!["write".to_string(), "upload".to_string(), "commit".to_string()],
                resources: vec!["*".to_string()],
                condition: Some(PolicyCondition {
                    field: "subject.roles".to_string(),
                    operator: ConditionOperator::Contains,
                    value: serde_json::Value::String("user".to_string()),
                }),
            },
            Policy {
                id: Uuid::new_v4(),
                tenant_id: Uuid::nil(),
                name: "default-allow-admin".to_string(),
                effect: PolicyEffect::Allow,
                actions: vec!["admin".to_string(), "delete".to_string(), "export".to_string()],
                resources: vec!["*".to_string()],
                condition: Some(PolicyCondition {
                    field: "subject.roles".to_string(),
                    operator: ConditionOperator::Contains,
                    value: serde_json::Value::String("admin".to_string()),
                }),
            },
        ];

        evaluator
    }
}

impl Default for PolicyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for policy management
pub mod helpers {
    use super::*;

    /// Create a simple allow policy
    pub fn create_allow_policy(
        tenant_id: Uuid,
        name: &str,
        actions: Vec<String>,
        resources: Vec<String>,
        condition: Option<PolicyCondition>,
    ) -> Policy {
        Policy {
            id: Uuid::new_v4(),
            tenant_id,
            name: name.to_string(),
            effect: PolicyEffect::Allow,
            actions,
            resources,
            condition,
        }
    }

    /// Create a simple deny policy
    pub fn create_deny_policy(
        tenant_id: Uuid,
        name: &str,
        actions: Vec<String>,
        resources: Vec<String>,
        condition: Option<PolicyCondition>,
    ) -> Policy {
        Policy {
            id: Uuid::new_v4(),
            tenant_id,
            name: name.to_string(),
            effect: PolicyEffect::Deny,
            actions,
            resources,
            condition,
        }
    }

    /// Create a role-based condition
    pub fn create_role_condition(roles: Vec<String>) -> PolicyCondition {
        PolicyCondition {
            field: "subject.roles".to_string(),
            operator: ConditionOperator::Contains,
            value: serde_json::Value::Array(
                roles.into_iter().map(serde_json::Value::String).collect()
            ),
        }
    }

    /// Create a classification-based condition
    pub fn create_classification_condition(classification: &str) -> PolicyCondition {
        PolicyCondition {
            field: "resource.classification".to_string(),
            operator: ConditionOperator::Equals,
            value: serde_json::Value::String(classification.to_string()),
        }
    }

    /// Create a path-prefix condition
    pub fn create_path_prefix_condition(prefix: &str) -> PolicyCondition {
        PolicyCondition {
            field: "resource.path".to_string(),
            operator: ConditionOperator::StartsWith,
            value: serde_json::Value::String(prefix.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_policy_effect_serialization() {
        let allow = PolicyEffect::Allow;
        let deny = PolicyEffect::Deny;

        assert_eq!(serde_json::to_string(&allow).unwrap(), "\"allow\"");
        assert_eq!(serde_json::to_string(&deny).unwrap(), "\"deny\"");
    }

    #[test]
    fn test_resource_pattern_matching() {
        let evaluator = PolicyEvaluator::new();

        // Wildcard
        assert!(evaluator.matches_resource_pattern("*", "any/resource"));
        assert!(evaluator.matches_resource_pattern("*", "repo/data/file.csv"));

        // Path prefix
        assert!(evaluator.matches_resource_pattern("datasets/**", "datasets/secure/file.csv"));
        assert!(evaluator.matches_resource_pattern("datasets/**", "datasets/public/file.csv"));
        assert!(!evaluator.matches_resource_pattern("datasets/**", "other/file.csv"));

        // Single level wildcard
        assert!(evaluator.matches_resource_pattern("datasets/*", "datasets/file.csv"));
        assert!(!evaluator.matches_resource_pattern("datasets/*", "datasets/secure/file.csv"));

        // Exact match
        assert!(evaluator.matches_resource_pattern("repo/data/file.csv", "repo/data/file.csv"));
        assert!(!evaluator.matches_resource_pattern("repo/data/file.csv", "repo/data/other.csv"));
    }

    #[test]
    fn test_condition_evaluation() {
        let mut evaluator = PolicyEvaluator::new();
        
        // Add test subject attributes
        let mut attributes = HashMap::new();
        attributes.insert("roles".to_string(), serde_json::json!(["user", "admin"]));
        attributes.insert("org".to_string(), serde_json::json!("acme"));
        
        evaluator.subject_attributes.insert(
            "test-user".to_string(),
            SubjectAttributes {
                subject: "test-user".to_string(),
                attributes,
            },
        );

        let request = AccessRequest {
            subject: "test-user".to_string(),
            action: "read".to_string(),
            resource: "repo/data/file.csv".to_string(),
            context: HashMap::new(),
        };

        // Test equals condition
        let condition = PolicyCondition {
            field: "subject.roles".to_string(),
            operator: ConditionOperator::Contains,
            value: serde_json::json!("user"),
        };

        assert!(evaluator.evaluate_condition(&condition, &request).unwrap());

        // Test not equals condition
        let condition = PolicyCondition {
            field: "subject.org".to_string(),
            operator: ConditionOperator::Equals,
            value: serde_json::json!("acme"),
        };

        assert!(evaluator.evaluate_condition(&condition, &request).unwrap());
    }

    #[test]
    fn test_policy_evaluation() {
        let evaluator = PolicyEvaluator::create_default();
        
        let request = AccessRequest {
            subject: "test-user".to_string(),
            action: "read".to_string(),
            resource: "repo/data/file.csv".to_string(),
            context: HashMap::new(),
        };

        let decision = evaluator.evaluate(&request).unwrap();
        assert_eq!(decision.decision, PolicyEffect::Allow);
        assert!(decision.policy_id.is_some());
    }

    #[test]
    fn test_deny_takes_precedence() {
        let mut evaluator = PolicyEvaluator::new();
        
        // Add allow policy
        evaluator.policies.push(Policy {
            id: Uuid::new_v4(),
            tenant_id: Uuid::nil(),
            name: "allow".to_string(),
            effect: PolicyEffect::Allow,
            actions: vec!["read".to_string()],
            resources: vec!["*".to_string()],
            condition: None,
        });

        // Add deny policy
        evaluator.policies.push(Policy {
            id: Uuid::new_v4(),
            tenant_id: Uuid::nil(),
            name: "deny".to_string(),
            effect: PolicyEffect::Deny,
            actions: vec!["read".to_string()],
            resources: vec!["*".to_string()],
            condition: None,
        });

        let request = AccessRequest {
            subject: "test-user".to_string(),
            action: "read".to_string(),
            resource: "repo/data/file.csv".to_string(),
            context: HashMap::new(),
        };

        let decision = evaluator.evaluate(&request).unwrap();
        assert_eq!(decision.decision, PolicyEffect::Deny);
    }

    #[test]
    fn test_default_deny() {
        let evaluator = PolicyEvaluator::new();
        
        let request = AccessRequest {
            subject: "test-user".to_string(),
            action: "read".to_string(),
            resource: "repo/data/file.csv".to_string(),
            context: HashMap::new(),
        };

        let decision = evaluator.evaluate(&request).unwrap();
        assert_eq!(decision.decision, PolicyEffect::Deny);
        assert_eq!(decision.reason, "No matching policies found");
    }
}
