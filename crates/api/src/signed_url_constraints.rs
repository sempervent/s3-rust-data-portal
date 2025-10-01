// Signed URL Constraints System
// Implements optional IP CIDR restrictions, user agent pinning capabilities
// Enforces max rate per URL on gateway and time-based access controls

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use ipnet::{IpNet, Ipv4Net, Ipv6Net};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedUrlConstraint {
    pub id: Uuid,
    pub url_id: Uuid,
    pub constraint_type: ConstraintType,
    pub configuration: ConstraintConfiguration,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    IpCidrRestriction,
    UserAgentPinning,
    RateLimit,
    TimeBasedAccess,
    GeographicRestriction,
    DeviceFingerprinting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintConfiguration {
    pub ip_cidr_restrictions: Option<IpCidrRestrictions>,
    pub user_agent_pinning: Option<UserAgentPinning>,
    pub rate_limit: Option<RateLimit>,
    pub time_based_access: Option<TimeBasedAccess>,
    pub geographic_restriction: Option<GeographicRestriction>,
    pub device_fingerprinting: Option<DeviceFingerprinting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpCidrRestrictions {
    pub allowed_cidrs: Vec<String>,
    pub blocked_cidrs: Vec<String>,
    pub allow_private_ips: bool,
    pub allow_public_ips: bool,
    pub log_violations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgentPinning {
    pub required_user_agents: Vec<String>,
    pub blocked_user_agents: Vec<String>,
    pub case_sensitive: bool,
    pub partial_match: bool,
    pub log_violations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_limit: u32,
    pub window_size_seconds: u64,
    pub enforcement_action: EnforcementAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementAction {
    Block,
    Throttle,
    Log,
    Challenge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedAccess {
    pub allowed_hours: Vec<u8>, // 0-23
    pub allowed_days: Vec<u8>,  // 0-6 (Monday-Sunday)
    pub timezone: String,
    pub start_time: Option<String>, // HH:MM format
    pub end_time: Option<String>,   // HH:MM format
    pub grace_period_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicRestriction {
    pub allowed_countries: Vec<String>, // ISO country codes
    pub blocked_countries: Vec<String>,
    pub allowed_regions: Vec<String>,
    pub blocked_regions: Vec<String>,
    pub require_vpn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprinting {
    pub required_attributes: Vec<String>,
    pub blocked_attributes: Vec<String>,
    pub fingerprint_algorithm: String,
    pub tolerance_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedUrlRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub client_ip: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
    pub constraints: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedUrlResponse {
    pub url: String,
    pub expires_at: DateTime<Utc>,
    pub constraints_applied: Vec<Uuid>,
    pub access_token: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    pub id: Uuid,
    pub url_id: Uuid,
    pub constraint_id: Uuid,
    pub violation_type: ViolationType,
    pub client_ip: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
    pub details: ViolationDetails,
    pub action_taken: EnforcementAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    IpCidrViolation,
    UserAgentViolation,
    RateLimitExceeded,
    TimeRestrictionViolation,
    GeographicViolation,
    DeviceFingerprintViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationDetails {
    pub constraint_value: String,
    pub actual_value: String,
    pub severity: ViolationSeverity,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct SignedUrlConstraintService {
    constraints: Arc<RwLock<Vec<SignedUrlConstraint>>>,
    violations: Arc<RwLock<Vec<ConstraintViolation>>>,
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiterState>>>,
    ip_geolocation: Arc<RwLock<HashMap<String, GeographicInfo>>>,
}

#[derive(Debug, Clone)]
pub struct RateLimiterState {
    pub requests: Vec<DateTime<Utc>>,
    pub last_reset: DateTime<Utc>,
    pub current_window: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicInfo {
    pub country: String,
    pub region: String,
    pub city: String,
    pub isp: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl SignedUrlConstraintService {
    pub fn new() -> Self {
        Self {
            constraints: Arc::new(RwLock::new(Vec::new())),
            violations: Arc::new(RwLock::new(Vec::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            ip_geolocation: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new signed URL constraint
    pub async fn create_constraint(
        &self,
        url_id: Uuid,
        constraint_type: ConstraintType,
        configuration: ConstraintConfiguration,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<SignedUrlConstraint, Box<dyn std::error::Error + Send + Sync>> {
        let constraint = SignedUrlConstraint {
            id: Uuid::new_v4(),
            url_id,
            constraint_type,
            configuration,
            created_at: Utc::now(),
            expires_at,
            active: true,
        };

        let mut constraints = self.constraints.write().await;
        constraints.push(constraint.clone());

        Ok(constraint)
    }

    /// Validate signed URL request against constraints
    pub async fn validate_request(
        &self,
        request: &SignedUrlRequest,
    ) -> Result<ValidationResult, Box<dyn std::error::Error + Send + Sync>> {
        let constraints = self.constraints.read().await;
        let applicable_constraints: Vec<&SignedUrlConstraint> = constraints
            .iter()
            .filter(|c| c.url_id == Uuid::parse_str(&request.url).unwrap_or_default())
            .filter(|c| c.active)
            .filter(|c| c.expires_at.is_none() || c.expires_at.unwrap() > Utc::now())
            .collect();

        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        for constraint in applicable_constraints {
            match &constraint.constraint_type {
                ConstraintType::IpCidrRestriction => {
                    if let Some(ip_restrictions) = &constraint.configuration.ip_cidr_restrictions {
                        match self.validate_ip_cidr(request.client_ip.as_str(), ip_restrictions).await {
                            Ok(_) => {}
                            Err(violation) => {
                                violations.push(violation);
                            }
                        }
                    }
                }
                ConstraintType::UserAgentPinning => {
                    if let Some(ua_pinning) = &constraint.configuration.user_agent_pinning {
                        match self.validate_user_agent(request.user_agent.as_str(), ua_pinning).await {
                            Ok(_) => {}
                            Err(violation) => {
                                violations.push(violation);
                            }
                        }
                    }
                }
                ConstraintType::RateLimit => {
                    if let Some(rate_limit) = &constraint.configuration.rate_limit {
                        match self.validate_rate_limit(request, rate_limit).await {
                            Ok(_) => {}
                            Err(violation) => {
                                violations.push(violation);
                            }
                        }
                    }
                }
                ConstraintType::TimeBasedAccess => {
                    if let Some(time_access) = &constraint.configuration.time_based_access {
                        match self.validate_time_based_access(time_access).await {
                            Ok(_) => {}
                            Err(violation) => {
                                violations.push(violation);
                            }
                        }
                    }
                }
                ConstraintType::GeographicRestriction => {
                    if let Some(geo_restriction) = &constraint.configuration.geographic_restriction {
                        match self.validate_geographic_restriction(request.client_ip.as_str(), geo_restriction).await {
                            Ok(_) => {}
                            Err(violation) => {
                                violations.push(violation);
                            }
                        }
                    }
                }
                ConstraintType::DeviceFingerprinting => {
                    if let Some(device_fp) = &constraint.configuration.device_fingerprinting {
                        match self.validate_device_fingerprint(request, device_fp).await {
                            Ok(_) => {}
                            Err(violation) => {
                                violations.push(violation);
                            }
                        }
                    }
                }
            }
        }

        // Record violations
        for violation in &violations {
            self.record_violation(violation).await?;
        }

        Ok(ValidationResult {
            valid: violations.is_empty(),
            violations,
            warnings,
            applied_constraints: applicable_constraints.iter().map(|c| c.id).collect(),
        })
    }

    /// Validate IP CIDR restrictions
    async fn validate_ip_cidr(
        &self,
        client_ip: &str,
        restrictions: &IpCidrRestrictions,
    ) -> Result<(), ConstraintViolation> {
        let client_ip_addr: IpAddr = client_ip.parse()
            .map_err(|_| ConstraintViolation {
                id: Uuid::new_v4(),
                url_id: Uuid::new_v4(),
                constraint_id: Uuid::new_v4(),
                violation_type: ViolationType::IpCidrViolation,
                client_ip: client_ip.to_string(),
                user_agent: String::new(),
                timestamp: Utc::now(),
                details: ViolationDetails {
                    constraint_value: "Valid IP".to_string(),
                    actual_value: client_ip.to_string(),
                    severity: ViolationSeverity::High,
                    context: HashMap::new(),
                },
                action_taken: EnforcementAction::Block,
            })?;

        // Check blocked CIDRs first
        for blocked_cidr in &restrictions.blocked_cidrs {
            let cidr: IpNet = blocked_cidr.parse()
                .map_err(|_| ConstraintViolation {
                    id: Uuid::new_v4(),
                    url_id: Uuid::new_v4(),
                    constraint_id: Uuid::new_v4(),
                    violation_type: ViolationType::IpCidrViolation,
                    client_ip: client_ip.to_string(),
                    user_agent: String::new(),
                    timestamp: Utc::now(),
                    details: ViolationDetails {
                        constraint_value: "Not in blocked CIDR".to_string(),
                        actual_value: client_ip.to_string(),
                        severity: ViolationSeverity::High,
                        context: HashMap::new(),
                    },
                    action_taken: EnforcementAction::Block,
                })?;

            if cidr.contains(&client_ip_addr) {
                return Err(ConstraintViolation {
                    id: Uuid::new_v4(),
                    url_id: Uuid::new_v4(),
                    constraint_id: Uuid::new_v4(),
                    violation_type: ViolationType::IpCidrViolation,
                    client_ip: client_ip.to_string(),
                    user_agent: String::new(),
                    timestamp: Utc::now(),
                    details: ViolationDetails {
                        constraint_value: format!("Not in {}", blocked_cidr),
                        actual_value: client_ip.to_string(),
                        severity: ViolationSeverity::High,
                        context: HashMap::new(),
                    },
                    action_taken: EnforcementAction::Block,
                });
            }
        }

        // Check allowed CIDRs
        if !restrictions.allowed_cidrs.is_empty() {
            let mut allowed = false;
            for allowed_cidr in &restrictions.allowed_cidrs {
                let cidr: IpNet = allowed_cidr.parse()
                    .map_err(|_| ConstraintViolation {
                        id: Uuid::new_v4(),
                        url_id: Uuid::new_v4(),
                        constraint_id: Uuid::new_v4(),
                        violation_type: ViolationType::IpCidrViolation,
                        client_ip: client_ip.to_string(),
                        user_agent: String::new(),
                        timestamp: Utc::now(),
                        details: ViolationDetails {
                            constraint_value: "In allowed CIDR".to_string(),
                            actual_value: client_ip.to_string(),
                            severity: ViolationSeverity::High,
                            context: HashMap::new(),
                        },
                        action_taken: EnforcementAction::Block,
                    })?;

                if cidr.contains(&client_ip_addr) {
                    allowed = true;
                    break;
                }
            }

            if !allowed {
                return Err(ConstraintViolation {
                    id: Uuid::new_v4(),
                    url_id: Uuid::new_v4(),
                    constraint_id: Uuid::new_v4(),
                    violation_type: ViolationType::IpCidrViolation,
                    client_ip: client_ip.to_string(),
                    user_agent: String::new(),
                    timestamp: Utc::now(),
                    details: ViolationDetails {
                        constraint_value: "In allowed CIDR".to_string(),
                        actual_value: client_ip.to_string(),
                        severity: ViolationSeverity::High,
                        context: HashMap::new(),
                    },
                    action_taken: EnforcementAction::Block,
                });
            }
        }

        Ok(())
    }

    /// Validate user agent pinning
    async fn validate_user_agent(
        &self,
        user_agent: &str,
        pinning: &UserAgentPinning,
    ) -> Result<(), ConstraintViolation> {
        // Check blocked user agents
        for blocked_ua in &pinning.blocked_user_agents {
            let matches = if pinning.case_sensitive {
                user_agent.contains(blocked_ua)
            } else {
                user_agent.to_lowercase().contains(&blocked_ua.to_lowercase())
            };

            if matches {
                return Err(ConstraintViolation {
                    id: Uuid::new_v4(),
                    url_id: Uuid::new_v4(),
                    constraint_id: Uuid::new_v4(),
                    violation_type: ViolationType::UserAgentViolation,
                    client_ip: String::new(),
                    user_agent: user_agent.to_string(),
                    timestamp: Utc::now(),
                    details: ViolationDetails {
                        constraint_value: format!("Not {}", blocked_ua),
                        actual_value: user_agent.to_string(),
                        severity: ViolationSeverity::Medium,
                        context: HashMap::new(),
                    },
                    action_taken: EnforcementAction::Block,
                });
            }
        }

        // Check required user agents
        if !pinning.required_user_agents.is_empty() {
            let mut allowed = false;
            for required_ua in &pinning.required_user_agents {
                let matches = if pinning.case_sensitive {
                    user_agent.contains(required_ua)
                } else {
                    user_agent.to_lowercase().contains(&required_ua.to_lowercase())
                };

                if matches {
                    allowed = true;
                    break;
                }
            }

            if !allowed {
                return Err(ConstraintViolation {
                    id: Uuid::new_v4(),
                    url_id: Uuid::new_v4(),
                    constraint_id: Uuid::new_v4(),
                    violation_type: ViolationType::UserAgentViolation,
                    client_ip: String::new(),
                    user_agent: user_agent.to_string(),
                    timestamp: Utc::now(),
                    details: ViolationDetails {
                        constraint_value: "Required user agent".to_string(),
                        actual_value: user_agent.to_string(),
                        severity: ViolationSeverity::Medium,
                        context: HashMap::new(),
                    },
                    action_taken: EnforcementAction::Block,
                });
            }
        }

        Ok(())
    }

    /// Validate rate limit
    async fn validate_rate_limit(
        &self,
        request: &SignedUrlRequest,
        rate_limit: &RateLimit,
    ) -> Result<(), ConstraintViolation> {
        let key = format!("{}:{}", request.client_ip, request.url);
        let now = Utc::now();

        let mut rate_limiters = self.rate_limiters.write().await;
        let limiter = rate_limiters.entry(key.clone()).or_insert_with(|| RateLimiterState {
            requests: Vec::new(),
            last_reset: now,
            current_window: 0,
        });

        // Clean old requests
        limiter.requests.retain(|&timestamp| now - timestamp < Duration::seconds(rate_limit.window_size_seconds as i64));
        limiter.requests.push(now);

        let request_count = limiter.requests.len() as u32;

        if request_count > rate_limit.requests_per_minute {
            return Err(ConstraintViolation {
                id: Uuid::new_v4(),
                url_id: Uuid::new_v4(),
                constraint_id: Uuid::new_v4(),
                violation_type: ViolationType::RateLimitExceeded,
                client_ip: request.client_ip.clone(),
                user_agent: request.user_agent.clone(),
                timestamp: now,
                details: ViolationDetails {
                    constraint_value: format!("Max {} requests per minute", rate_limit.requests_per_minute),
                    actual_value: request_count.to_string(),
                    severity: ViolationSeverity::High,
                    context: HashMap::new(),
                },
                action_taken: rate_limit.enforcement_action.clone(),
            });
        }

        Ok(())
    }

    /// Validate time-based access
    async fn validate_time_based_access(
        &self,
        time_access: &TimeBasedAccess,
    ) -> Result<(), ConstraintViolation> {
        let now = Utc::now();
        let current_hour = now.hour() as u8;
        let current_day = now.weekday().num_days_from_monday() as u8;

        // Check allowed hours
        if !time_access.allowed_hours.is_empty() && !time_access.allowed_hours.contains(&current_hour) {
            return Err(ConstraintViolation {
                id: Uuid::new_v4(),
                url_id: Uuid::new_v4(),
                constraint_id: Uuid::new_v4(),
                violation_type: ViolationType::TimeRestrictionViolation,
                client_ip: String::new(),
                user_agent: String::new(),
                timestamp: now,
                details: ViolationDetails {
                    constraint_value: format!("Allowed hours: {:?}", time_access.allowed_hours),
                    actual_value: current_hour.to_string(),
                    severity: ViolationSeverity::Medium,
                    context: HashMap::new(),
                },
                action_taken: EnforcementAction::Block,
            });
        }

        // Check allowed days
        if !time_access.allowed_days.is_empty() && !time_access.allowed_days.contains(&current_day) {
            return Err(ConstraintViolation {
                id: Uuid::new_v4(),
                url_id: Uuid::new_v4(),
                constraint_id: Uuid::new_v4(),
                violation_type: ViolationType::TimeRestrictionViolation,
                client_ip: String::new(),
                user_agent: String::new(),
                timestamp: now,
                details: ViolationDetails {
                    constraint_value: format!("Allowed days: {:?}", time_access.allowed_days),
                    actual_value: current_day.to_string(),
                    severity: ViolationSeverity::Medium,
                    context: HashMap::new(),
                },
                action_taken: EnforcementAction::Block,
            });
        }

        Ok(())
    }

    /// Validate geographic restriction
    async fn validate_geographic_restriction(
        &self,
        client_ip: &str,
        geo_restriction: &GeographicRestriction,
    ) -> Result<(), ConstraintViolation> {
        // In a real implementation, you would use a geolocation service
        // For now, we'll simulate the check
        let geo_info = self.get_geographic_info(client_ip).await;

        if let Some(geo_info) = geo_info {
            // Check blocked countries
            if geo_restriction.blocked_countries.contains(&geo_info.country) {
                return Err(ConstraintViolation {
                    id: Uuid::new_v4(),
                    url_id: Uuid::new_v4(),
                    constraint_id: Uuid::new_v4(),
                    violation_type: ViolationType::GeographicViolation,
                    client_ip: client_ip.to_string(),
                    user_agent: String::new(),
                    timestamp: Utc::now(),
                    details: ViolationDetails {
                        constraint_value: format!("Not in blocked countries: {:?}", geo_restriction.blocked_countries),
                        actual_value: geo_info.country,
                        severity: ViolationSeverity::High,
                        context: HashMap::new(),
                    },
                    action_taken: EnforcementAction::Block,
                });
            }

            // Check allowed countries
            if !geo_restriction.allowed_countries.is_empty() && !geo_restriction.allowed_countries.contains(&geo_info.country) {
                return Err(ConstraintViolation {
                    id: Uuid::new_v4(),
                    url_id: Uuid::new_v4(),
                    constraint_id: Uuid::new_v4(),
                    violation_type: ViolationType::GeographicViolation,
                    client_ip: client_ip.to_string(),
                    user_agent: String::new(),
                    timestamp: Utc::now(),
                    details: ViolationDetails {
                        constraint_value: format!("In allowed countries: {:?}", geo_restriction.allowed_countries),
                        actual_value: geo_info.country,
                        severity: ViolationSeverity::High,
                        context: HashMap::new(),
                    },
                    action_taken: EnforcementAction::Block,
                });
            }
        }

        Ok(())
    }

    /// Validate device fingerprinting
    async fn validate_device_fingerprint(
        &self,
        request: &SignedUrlRequest,
        device_fp: &DeviceFingerprinting,
    ) -> Result<(), ConstraintViolation> {
        // In a real implementation, you would generate and validate device fingerprints
        // For now, we'll simulate the check
        let fingerprint = self.generate_device_fingerprint(request).await;

        // Check blocked attributes
        for blocked_attr in &device_fp.blocked_attributes {
            if fingerprint.contains(blocked_attr) {
                return Err(ConstraintViolation {
                    id: Uuid::new_v4(),
                    url_id: Uuid::new_v4(),
                    constraint_id: Uuid::new_v4(),
                    violation_type: ViolationType::DeviceFingerprintViolation,
                    client_ip: request.client_ip.clone(),
                    user_agent: request.user_agent.clone(),
                    timestamp: Utc::now(),
                    details: ViolationDetails {
                        constraint_value: format!("Not containing blocked attributes: {:?}", device_fp.blocked_attributes),
                        actual_value: fingerprint,
                        severity: ViolationSeverity::Medium,
                        context: HashMap::new(),
                    },
                    action_taken: EnforcementAction::Block,
                });
            }
        }

        Ok(())
    }

    /// Get geographic information for IP
    async fn get_geographic_info(&self, ip: &str) -> Option<GeographicInfo> {
        let mut geo_cache = self.ip_geolocation.write().await;
        
        if let Some(info) = geo_cache.get(ip) {
            return Some(info.clone());
        }

        // In a real implementation, you would call a geolocation service
        let geo_info = GeographicInfo {
            country: "US".to_string(),
            region: "CA".to_string(),
            city: "San Francisco".to_string(),
            isp: "Example ISP".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
        };

        geo_cache.insert(ip.to_string(), geo_info.clone());
        Some(geo_info)
    }

    /// Generate device fingerprint
    async fn generate_device_fingerprint(&self, request: &SignedUrlRequest) -> String {
        // In a real implementation, you would generate a proper device fingerprint
        // For now, we'll create a simple hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.client_ip.hash(&mut hasher);
        request.user_agent.hash(&mut hasher);
        request.timestamp.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }

    /// Record constraint violation
    async fn record_violation(&self, violation: &ConstraintViolation) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut violations = self.violations.write().await;
        violations.push(violation.clone());
        Ok(())
    }

    /// Get constraint violations
    pub async fn get_violations(&self, url_id: Option<Uuid>) -> Result<Vec<ConstraintViolation>, Box<dyn std::error::Error + Send + Sync>> {
        let violations = self.violations.read().await;
        let filtered_violations: Vec<ConstraintViolation> = if let Some(url_id) = url_id {
            violations.iter().filter(|v| v.url_id == url_id).cloned().collect()
        } else {
            violations.clone()
        };
        Ok(filtered_violations)
    }

    /// Get constraint statistics
    pub async fn get_constraint_statistics(&self) -> Result<ConstraintStatistics, Box<dyn std::error::Error + Send + Sync>> {
        let constraints = self.constraints.read().await;
        let violations = self.violations.read().await;

        let total_constraints = constraints.len();
        let active_constraints = constraints.iter().filter(|c| c.active).count();

        let total_violations = violations.len();
        let ip_violations = violations.iter().filter(|v| matches!(v.violation_type, ViolationType::IpCidrViolation)).count();
        let ua_violations = violations.iter().filter(|v| matches!(v.violation_type, ViolationType::UserAgentViolation)).count();
        let rate_violations = violations.iter().filter(|v| matches!(v.violation_type, ViolationType::RateLimitExceeded)).count();

        Ok(ConstraintStatistics {
            total_constraints,
            active_constraints,
            total_violations,
            ip_violations,
            ua_violations,
            rate_violations,
            violation_rate: if total_constraints > 0 { total_violations as f64 / total_constraints as f64 } else { 0.0 },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<ConstraintViolation>,
    pub warnings: Vec<String>,
    pub applied_constraints: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintStatistics {
    pub total_constraints: usize,
    pub active_constraints: usize,
    pub total_violations: usize,
    pub ip_violations: usize,
    pub ua_violations: usize,
    pub rate_violations: usize,
    pub violation_rate: f64,
}

/// Signed URL constraints router
pub fn signed_url_constraints_router() -> Router {
    Router::new()
        .route("/signed-url-constraints", post(create_constraint))
        .route("/signed-url-constraints/validate", post(validate_request))
        .route("/signed-url-constraints/violations", get(get_violations))
        .route("/signed-url-constraints/statistics", get(get_constraint_statistics))
}

/// Create constraint
async fn create_constraint(
    State(service): State<Arc<SignedUrlConstraintService>>,
    Json(request): Json<CreateConstraintRequest>,
) -> Result<Json<SignedUrlConstraint>, String> {
    let constraint = service.create_constraint(
        request.url_id,
        request.constraint_type,
        request.configuration,
        request.expires_at,
    ).await
        .map_err(|e| format!("Failed to create constraint: {}", e))?;
    
    Ok(Json(constraint))
}

/// Validate request
async fn validate_request(
    State(service): State<Arc<SignedUrlConstraintService>>,
    Json(request): Json<SignedUrlRequest>,
) -> Result<Json<ValidationResult>, String> {
    let result = service.validate_request(&request).await
        .map_err(|e| format!("Failed to validate request: {}", e))?;
    
    Ok(Json(result))
}

/// Get violations
async fn get_violations(
    State(service): State<Arc<SignedUrlConstraintService>>,
    Query(params): Query<GetViolationsQuery>,
) -> Result<Json<Vec<ConstraintViolation>>, String> {
    let violations = service.get_violations(params.url_id).await
        .map_err(|e| format!("Failed to get violations: {}", e))?;
    
    Ok(Json(violations))
}

/// Get constraint statistics
async fn get_constraint_statistics(
    State(service): State<Arc<SignedUrlConstraintService>>,
) -> Result<Json<ConstraintStatistics>, String> {
    let stats = service.get_constraint_statistics().await
        .map_err(|e| format!("Failed to get constraint statistics: {}", e))?;
    
    Ok(Json(stats))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConstraintRequest {
    pub url_id: Uuid,
    pub constraint_type: ConstraintType,
    pub configuration: ConstraintConfiguration,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetViolationsQuery {
    pub url_id: Option<Uuid>,
}
