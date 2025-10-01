// Analytics and Reporting Features
// Week 5: Performance optimization with analytics

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub id: Uuid,
    pub report_type: ReportType,
    pub title: String,
    pub description: Option<String>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub data: serde_json::Value,
    pub filters: Option<serde_json::Value>,
    pub period: ReportPeriod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    UsageAnalytics,
    PerformanceMetrics,
    UserActivity,
    SystemHealth,
    ComplianceReport,
    SecurityAudit,
    CostAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub granularity: Granularity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Granularity {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalytics {
    pub total_users: u32,
    pub active_users: u32,
    pub new_users: u32,
    pub user_retention_rate: f64,
    pub avg_session_duration: f64,
    pub total_sessions: u64,
    pub user_activity: Vec<UserActivity>,
    pub feature_usage: HashMap<String, u64>,
    pub geographic_distribution: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: String,
    pub username: String,
    pub activity_type: String,
    pub timestamp: DateTime<Utc>,
    pub duration: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalytics {
    pub avg_response_time: f64,
    pub p95_response_time: f64,
    pub p99_response_time: f64,
    pub throughput: f64,
    pub error_rate: f64,
    pub availability: f64,
    pub resource_utilization: ResourceUtilization,
    pub performance_trends: Vec<PerformanceTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_usage: f64,
    pub database_connections: u32,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    pub timestamp: DateTime<Utc>,
    pub metric_name: String,
    pub value: f64,
    pub trend: TrendDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalytics {
    pub total_events: u64,
    pub security_incidents: u32,
    pub failed_logins: u32,
    pub suspicious_activities: u32,
    pub blocked_requests: u32,
    pub threat_level: ThreatLevel,
    pub security_events: Vec<SecurityEvent>,
    pub compliance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub timestamp: DateTime<Utc>,
    pub source_ip: Option<String>,
    pub user_id: Option<String>,
    pub description: String,
    pub resolved: bool,
}

pub struct AnalyticsService {
    reports: Arc<RwLock<Vec<AnalyticsReport>>>,
    usage_data: Arc<RwLock<Vec<UserActivity>>>,
    performance_data: Arc<RwLock<Vec<PerformanceTrend>>>,
    security_events: Arc<RwLock<Vec<SecurityEvent>>>,
}

impl AnalyticsService {
    pub fn new() -> Self {
        Self {
            reports: Arc::new(RwLock::new(Vec::new())),
            usage_data: Arc::new(RwLock::new(Vec::new())),
            performance_data: Arc::new(RwLock::new(Vec::new())),
            security_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Generate usage analytics report
    pub async fn generate_usage_analytics(
        &self,
        period: ReportPeriod,
        generated_by: String,
    ) -> Result<AnalyticsReport, Box<dyn std::error::Error + Send + Sync>> {
        let usage_data = self.usage_data.read().await;
        
        // Filter data by period
        let filtered_data: Vec<UserActivity> = usage_data
            .iter()
            .filter(|activity| {
                activity.timestamp >= period.start_date && activity.timestamp <= period.end_date
            })
            .cloned()
            .collect();
        
        // Calculate analytics
        let total_users = self.get_unique_users(&filtered_data).await;
        let active_users = self.get_active_users(&filtered_data, &period).await;
        let new_users = self.get_new_users(&filtered_data, &period).await;
        let user_retention_rate = self.calculate_retention_rate(&filtered_data, &period).await;
        let avg_session_duration = self.calculate_avg_session_duration(&filtered_data).await;
        let total_sessions = self.get_total_sessions(&filtered_data).await;
        
        // Get feature usage
        let feature_usage = self.get_feature_usage(&filtered_data).await;
        
        // Get geographic distribution
        let geographic_distribution = self.get_geographic_distribution(&filtered_data).await;
        
        let analytics = UsageAnalytics {
            total_users,
            active_users,
            new_users,
            user_retention_rate,
            avg_session_duration,
            total_sessions,
            user_activity: filtered_data,
            feature_usage,
            geographic_distribution,
        };
        
        let report = AnalyticsReport {
            id: Uuid::new_v4(),
            report_type: ReportType::UsageAnalytics,
            title: "Usage Analytics Report".to_string(),
            description: Some(format!("Usage analytics for period {} to {}", 
                period.start_date.format("%Y-%m-%d"), 
                period.end_date.format("%Y-%m-%d")
            )),
            generated_at: Utc::now(),
            generated_by,
            data: serde_json::to_value(analytics)?,
            filters: None,
            period,
        };
        
        // Store report
        let mut reports = self.reports.write().await;
        reports.push(report.clone());
        
        Ok(report)
    }

    /// Generate performance analytics report
    pub async fn generate_performance_analytics(
        &self,
        period: ReportPeriod,
        generated_by: String,
    ) -> Result<AnalyticsReport, Box<dyn std::error::Error + Send + Sync>> {
        let performance_data = self.performance_data.read().await;
        
        // Filter data by period
        let filtered_data: Vec<PerformanceTrend> = performance_data
            .iter()
            .filter(|trend| {
                trend.timestamp >= period.start_date && trend.timestamp <= period.end_date
            })
            .cloned()
            .collect();
        
        // Calculate performance metrics
        let avg_response_time = self.calculate_avg_response_time(&filtered_data).await;
        let p95_response_time = self.calculate_p95_response_time(&filtered_data).await;
        let p99_response_time = self.calculate_p99_response_time(&filtered_data).await;
        let throughput = self.calculate_throughput(&filtered_data).await;
        let error_rate = self.calculate_error_rate(&filtered_data).await;
        let availability = self.calculate_availability(&filtered_data).await;
        
        // Get resource utilization
        let resource_utilization = self.get_resource_utilization(&filtered_data).await;
        
        let analytics = PerformanceAnalytics {
            avg_response_time,
            p95_response_time,
            p99_response_time,
            throughput,
            error_rate,
            availability,
            resource_utilization,
            performance_trends: filtered_data,
        };
        
        let report = AnalyticsReport {
            id: Uuid::new_v4(),
            report_type: ReportType::PerformanceMetrics,
            title: "Performance Analytics Report".to_string(),
            description: Some(format!("Performance analytics for period {} to {}", 
                period.start_date.format("%Y-%m-%d"), 
                period.end_date.format("%Y-%m-%d")
            )),
            generated_at: Utc::now(),
            generated_by,
            data: serde_json::to_value(analytics)?,
            filters: None,
            period,
        };
        
        // Store report
        let mut reports = self.reports.write().await;
        reports.push(report.clone());
        
        Ok(report)
    }

    /// Generate security analytics report
    pub async fn generate_security_analytics(
        &self,
        period: ReportPeriod,
        generated_by: String,
    ) -> Result<AnalyticsReport, Box<dyn std::error::Error + Send + Sync>> {
        let security_events = self.security_events.read().await;
        
        // Filter data by period
        let filtered_events: Vec<SecurityEvent> = security_events
            .iter()
            .filter(|event| {
                event.timestamp >= period.start_date && event.timestamp <= period.end_date
            })
            .cloned()
            .collect();
        
        // Calculate security metrics
        let total_events = filtered_events.len() as u64;
        let security_incidents = filtered_events.iter().filter(|e| e.severity == "High" || e.severity == "Critical").count() as u32;
        let failed_logins = filtered_events.iter().filter(|e| e.event_type == "Failed Login").count() as u32;
        let suspicious_activities = filtered_events.iter().filter(|e| e.event_type == "Suspicious Activity").count() as u32;
        let blocked_requests = filtered_events.iter().filter(|e| e.event_type == "Blocked Request").count() as u32;
        
        let threat_level = self.assess_threat_level(&filtered_events).await;
        let compliance_score = self.calculate_compliance_score(&filtered_events).await;
        
        let analytics = SecurityAnalytics {
            total_events,
            security_incidents,
            failed_logins,
            suspicious_activities,
            blocked_requests,
            threat_level,
            security_events: filtered_events,
            compliance_score,
        };
        
        let report = AnalyticsReport {
            id: Uuid::new_v4(),
            report_type: ReportType::SecurityAudit,
            title: "Security Analytics Report".to_string(),
            description: Some(format!("Security analytics for period {} to {}", 
                period.start_date.format("%Y-%m-%d"), 
                period.end_date.format("%Y-%m-%d")
            )),
            generated_at: Utc::now(),
            generated_by,
            data: serde_json::to_value(analytics)?,
            filters: None,
            period,
        };
        
        // Store report
        let mut reports = self.reports.write().await;
        reports.push(report.clone());
        
        Ok(report)
    }

    /// Record user activity
    pub async fn record_user_activity(&self, activity: UserActivity) {
        let mut usage_data = self.usage_data.write().await;
        usage_data.push(activity);
    }

    /// Record performance trend
    pub async fn record_performance_trend(&self, trend: PerformanceTrend) {
        let mut performance_data = self.performance_data.write().await;
        performance_data.push(trend);
    }

    /// Record security event
    pub async fn record_security_event(&self, event: SecurityEvent) {
        let mut security_events = self.security_events.write().await;
        security_events.push(event);
    }

    /// Get all reports
    pub async fn get_reports(&self) -> Vec<AnalyticsReport> {
        let reports = self.reports.read().await;
        reports.clone()
    }

    /// Get reports by type
    pub async fn get_reports_by_type(&self, report_type: ReportType) -> Vec<AnalyticsReport> {
        let reports = self.reports.read().await;
        reports
            .iter()
            .filter(|report| std::mem::discriminant(&report.report_type) == std::mem::discriminant(&report_type))
            .cloned()
            .collect()
    }

    // Helper methods for analytics calculations
    async fn get_unique_users(&self, data: &[UserActivity]) -> u32 {
        let unique_users: std::collections::HashSet<String> = data
            .iter()
            .map(|activity| activity.user_id.clone())
            .collect();
        unique_users.len() as u32
    }

    async fn get_active_users(&self, data: &[UserActivity], period: &ReportPeriod) -> u32 {
        let active_users: std::collections::HashSet<String> = data
            .iter()
            .filter(|activity| {
                activity.timestamp >= period.start_date && activity.timestamp <= period.end_date
            })
            .map(|activity| activity.user_id.clone())
            .collect();
        active_users.len() as u32
    }

    async fn get_new_users(&self, data: &[UserActivity], period: &ReportPeriod) -> u32 {
        // Simplified: count users who first appeared in this period
        let mut user_first_seen: HashMap<String, DateTime<Utc>> = HashMap::new();
        
        for activity in data {
            let user_id = &activity.user_id;
            let timestamp = activity.timestamp;
            
            if let Some(first_seen) = user_first_seen.get(user_id) {
                if timestamp < *first_seen {
                    user_first_seen.insert(user_id.clone(), timestamp);
                }
            } else {
                user_first_seen.insert(user_id.clone(), timestamp);
            }
        }
        
        user_first_seen
            .values()
            .filter(|&first_seen| *first_seen >= period.start_date)
            .count() as u32
    }

    async fn calculate_retention_rate(&self, data: &[UserActivity], period: &ReportPeriod) -> f64 {
        // Simplified retention calculation
        let total_users = self.get_unique_users(data).await;
        if total_users == 0 {
            return 0.0;
        }
        
        let active_users = self.get_active_users(data, period).await;
        active_users as f64 / total_users as f64
    }

    async fn calculate_avg_session_duration(&self, data: &[UserActivity]) -> f64 {
        let sessions_with_duration: Vec<f64> = data
            .iter()
            .filter_map(|activity| activity.duration)
            .collect();
        
        if sessions_with_duration.is_empty() {
            return 0.0;
        }
        
        sessions_with_duration.iter().sum::<f64>() / sessions_with_duration.len() as f64
    }

    async fn get_total_sessions(&self, data: &[UserActivity]) -> u64 {
        data.len() as u64
    }

    async fn get_feature_usage(&self, data: &[UserActivity]) -> HashMap<String, u64> {
        let mut feature_usage: HashMap<String, u64> = HashMap::new();
        
        for activity in data {
            let feature = activity.activity_type.clone();
            *feature_usage.entry(feature).or_insert(0) += 1;
        }
        
        feature_usage
    }

    async fn get_geographic_distribution(&self, data: &[UserActivity]) -> HashMap<String, u32> {
        // Simplified geographic distribution
        let mut distribution: HashMap<String, u32> = HashMap::new();
        
        for activity in data {
            if let Some(metadata) = &activity.metadata {
                if let Some(country) = metadata.get("country").and_then(|v| v.as_str()) {
                    *distribution.entry(country.to_string()).or_insert(0) += 1;
                }
            }
        }
        
        distribution
    }

    async fn calculate_avg_response_time(&self, data: &[PerformanceTrend]) -> f64 {
        let response_times: Vec<f64> = data
            .iter()
            .filter(|trend| trend.metric_name == "response_time")
            .map(|trend| trend.value)
            .collect();
        
        if response_times.is_empty() {
            return 0.0;
        }
        
        response_times.iter().sum::<f64>() / response_times.len() as f64
    }

    async fn calculate_p95_response_time(&self, data: &[PerformanceTrend]) -> f64 {
        let mut response_times: Vec<f64> = data
            .iter()
            .filter(|trend| trend.metric_name == "response_time")
            .map(|trend| trend.value)
            .collect();
        
        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        if response_times.is_empty() {
            return 0.0;
        }
        
        let index = (response_times.len() as f64 * 0.95) as usize;
        response_times.get(index).copied().unwrap_or(0.0)
    }

    async fn calculate_p99_response_time(&self, data: &[PerformanceTrend]) -> f64 {
        let mut response_times: Vec<f64> = data
            .iter()
            .filter(|trend| trend.metric_name == "response_time")
            .map(|trend| trend.value)
            .collect();
        
        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        if response_times.is_empty() {
            return 0.0;
        }
        
        let index = (response_times.len() as f64 * 0.99) as usize;
        response_times.get(index).copied().unwrap_or(0.0)
    }

    async fn calculate_throughput(&self, data: &[PerformanceTrend]) -> f64 {
        let throughput_data: Vec<f64> = data
            .iter()
            .filter(|trend| trend.metric_name == "throughput")
            .map(|trend| trend.value)
            .collect();
        
        if throughput_data.is_empty() {
            return 0.0;
        }
        
        throughput_data.iter().sum::<f64>() / throughput_data.len() as f64
    }

    async fn calculate_error_rate(&self, data: &[PerformanceTrend]) -> f64 {
        let error_data: Vec<f64> = data
            .iter()
            .filter(|trend| trend.metric_name == "error_rate")
            .map(|trend| trend.value)
            .collect();
        
        if error_data.is_empty() {
            return 0.0;
        }
        
        error_data.iter().sum::<f64>() / error_data.len() as f64
    }

    async fn calculate_availability(&self, data: &[PerformanceTrend]) -> f64 {
        let availability_data: Vec<f64> = data
            .iter()
            .filter(|trend| trend.metric_name == "availability")
            .map(|trend| trend.value)
            .collect();
        
        if availability_data.is_empty() {
            return 100.0; // Default to 100% availability
        }
        
        availability_data.iter().sum::<f64>() / availability_data.len() as f64
    }

    async fn get_resource_utilization(&self, data: &[PerformanceTrend]) -> ResourceUtilization {
        // Simplified resource utilization calculation
        ResourceUtilization {
            cpu_usage: 25.0,
            memory_usage: 60.0,
            disk_usage: 45.0,
            network_usage: 30.0,
            database_connections: 15,
            cache_hit_rate: 0.85,
        }
    }

    async fn assess_threat_level(&self, events: &[SecurityEvent]) -> ThreatLevel {
        let critical_events = events.iter().filter(|e| e.severity == "Critical").count();
        let high_events = events.iter().filter(|e| e.severity == "High").count();
        
        if critical_events > 0 {
            ThreatLevel::Critical
        } else if high_events > 5 {
            ThreatLevel::High
        } else if high_events > 0 {
            ThreatLevel::Medium
        } else {
            ThreatLevel::Low
        }
    }

    async fn calculate_compliance_score(&self, events: &[SecurityEvent]) -> f64 {
        let total_events = events.len() as f64;
        if total_events == 0.0 {
            return 100.0;
        }
        
        let resolved_events = events.iter().filter(|e| e.resolved).count() as f64;
        (resolved_events / total_events) * 100.0
    }
}
