// Cost Tracking and Estimation System
// Implements cost tracking and reporting for storage and egress

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub repo_id: Uuid,
    pub repo_name: String,
    pub storage_cost: f64,
    pub egress_cost: f64,
    pub total_cost: f64,
    pub cost_breakdown: CostBreakdown,
    pub estimated_monthly_cost: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub storage_size_bytes: u64,
    pub storage_cost_per_gb: f64,
    pub egress_bytes: u64,
    pub egress_cost_per_gb: f64,
    pub api_requests: u64,
    pub api_cost_per_request: f64,
    pub job_minutes: u64,
    pub job_cost_per_minute: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub user_id: String,
    pub event_type: UsageEventType,
    pub size_bytes: u64,
    pub duration_minutes: Option<u64>,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageEventType {
    Ingress,
    Egress,
    JobProcessing,
    ApiRequest,
    StorageOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostConfiguration {
    pub storage_cost_per_gb_month: f64,
    pub egress_cost_per_gb: f64,
    pub api_cost_per_request: f64,
    pub job_cost_per_minute: f64,
    pub currency: String,
}

impl Default for CostConfiguration {
    fn default() -> Self {
        Self {
            storage_cost_per_gb_month: 0.023, // AWS S3 Standard pricing
            egress_cost_per_gb: 0.09, // AWS S3 egress pricing
            api_cost_per_request: 0.0004, // AWS S3 API pricing
            job_cost_per_minute: 0.10, // Estimated compute cost
            currency: "USD".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub threshold_percentage: f64,
    pub current_usage_percentage: f64,
    pub budget_limit: f64,
    pub current_spend: f64,
    pub alert_type: AlertType,
    pub created_at: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    BudgetThreshold,
    BudgetExceeded,
    UnusualSpending,
    CostAnomaly,
}

pub struct CostTrackingService {
    usage_events: Arc<RwLock<Vec<UsageEvent>>>,
    cost_estimates: Arc<RwLock<HashMap<Uuid, CostEstimate>>>,
    cost_config: CostConfiguration,
    budget_alerts: Arc<RwLock<Vec<BudgetAlert>>>,
}

impl CostTrackingService {
    pub fn new(cost_config: CostConfiguration) -> Self {
        Self {
            usage_events: Arc::new(RwLock::new(Vec::new())),
            cost_estimates: Arc::new(RwLock::new(HashMap::new())),
            cost_config,
            budget_alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a usage event
    pub async fn record_usage_event(&self, event: UsageEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut events = self.usage_events.write().await;
        events.push(event.clone());
        
        // Update cost estimate for the repository
        self.update_cost_estimate(event.repo_id).await?;
        
        // Check for budget alerts
        self.check_budget_alerts(event.repo_id).await?;
        
        Ok(())
    }

    /// Get cost estimate for a repository
    pub async fn get_cost_estimate(&self, repo_id: Uuid) -> Result<Option<CostEstimate>, Box<dyn std::error::Error + Send + Sync>> {
        let estimates = self.cost_estimates.read().await;
        Ok(estimates.get(&repo_id).cloned())
    }

    /// Get cost estimates for all repositories
    pub async fn get_all_cost_estimates(&self) -> Result<Vec<CostEstimate>, Box<dyn std::error::Error + Send + Sync>> {
        let estimates = self.cost_estimates.read().await;
        Ok(estimates.values().cloned().collect())
    }

    /// Update cost estimate for a repository
    async fn update_cost_estimate(&self, repo_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let events = self.usage_events.read().await;
        
        // Filter events for this repository
        let repo_events: Vec<&UsageEvent> = events.iter()
            .filter(|event| event.repo_id == repo_id)
            .collect();
        
        if repo_events.is_empty() {
            return Ok(());
        }
        
        // Calculate cost breakdown
        let mut storage_size_bytes = 0u64;
        let mut egress_bytes = 0u64;
        let mut api_requests = 0u64;
        let mut job_minutes = 0u64;
        
        for event in &repo_events {
            match event.event_type {
                UsageEventType::Ingress | UsageEventType::StorageOperation => {
                    storage_size_bytes += event.size_bytes;
                }
                UsageEventType::Egress => {
                    egress_bytes += event.size_bytes;
                }
                UsageEventType::ApiRequest => {
                    api_requests += 1;
                }
                UsageEventType::JobProcessing => {
                    job_minutes += event.duration_minutes.unwrap_or(0);
                }
            }
        }
        
        // Calculate costs
        let storage_cost = (storage_size_bytes as f64 / 1_000_000_000.0) * self.cost_config.storage_cost_per_gb_month;
        let egress_cost = (egress_bytes as f64 / 1_000_000_000.0) * self.cost_config.egress_cost_per_gb;
        let api_cost = api_requests as f64 * self.cost_config.api_cost_per_request;
        let job_cost = job_minutes as f64 * self.cost_config.job_cost_per_minute;
        
        let total_cost = storage_cost + egress_cost + api_cost + job_cost;
        
        // Create cost estimate
        let cost_estimate = CostEstimate {
            repo_id,
            repo_name: self.get_repo_name(repo_id).await.unwrap_or_else(|| "Unknown".to_string()),
            storage_cost,
            egress_cost,
            total_cost,
            cost_breakdown: CostBreakdown {
                storage_size_bytes,
                storage_cost_per_gb: self.cost_config.storage_cost_per_gb_month,
                egress_bytes,
                egress_cost_per_gb: self.cost_config.egress_cost_per_gb,
                api_requests,
                api_cost_per_request: self.cost_config.api_cost_per_request,
                job_minutes,
                job_cost_per_minute: self.cost_config.job_cost_per_minute,
            },
            estimated_monthly_cost: total_cost * 30.0, // Rough monthly estimate
            last_updated: Utc::now(),
        };
        
        // Update cost estimates
        let mut estimates = self.cost_estimates.write().await;
        estimates.insert(repo_id, cost_estimate);
        
        Ok(())
    }

    /// Get repository name (placeholder implementation)
    async fn get_repo_name(&self, repo_id: Uuid) -> Option<String> {
        // In a real implementation, this would query the database
        Some(format!("repo-{}", repo_id))
    }

    /// Check for budget alerts
    async fn check_budget_alerts(&self, repo_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let estimates = self.cost_estimates.read().await;
        
        if let Some(estimate) = estimates.get(&repo_id) {
            // Get budget limit for repository (placeholder)
            let budget_limit = self.get_budget_limit(repo_id).await.unwrap_or(1000.0);
            let current_spend = estimate.total_cost;
            let usage_percentage = (current_spend / budget_limit) * 100.0;
            
            // Check if we need to create an alert
            if usage_percentage >= 80.0 {
                let alert = BudgetAlert {
                    id: Uuid::new_v4(),
                    repo_id,
                    threshold_percentage: 80.0,
                    current_usage_percentage: usage_percentage,
                    budget_limit,
                    current_spend,
                    alert_type: if usage_percentage >= 100.0 {
                        AlertType::BudgetExceeded
                    } else {
                        AlertType::BudgetThreshold
                    },
                    created_at: Utc::now(),
                    resolved: false,
                };
                
                let mut alerts = self.budget_alerts.write().await;
                alerts.push(alert);
            }
        }
        
        Ok(())
    }

    /// Get budget limit for repository (placeholder implementation)
    async fn get_budget_limit(&self, repo_id: Uuid) -> Option<f64> {
        // In a real implementation, this would query the database
        Some(1000.0)
    }

    /// Get budget alerts
    pub async fn get_budget_alerts(&self) -> Result<Vec<BudgetAlert>, Box<dyn std::error::Error + Send + Sync>> {
        let alerts = self.budget_alerts.read().await;
        Ok(alerts.clone())
    }

    /// Get usage analytics
    pub async fn get_usage_analytics(&self, repo_id: Option<Uuid>, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> Result<UsageAnalytics, Box<dyn std::error::Error + Send + Sync>> {
        let events = self.usage_events.read().await;
        
        let start_date = start_date.unwrap_or_else(|| Utc::now() - Duration::days(30));
        let end_date = end_date.unwrap_or_else(|| Utc::now());
        
        // Filter events
        let filtered_events: Vec<&UsageEvent> = events.iter()
            .filter(|event| {
                if let Some(repo_id) = repo_id {
                    event.repo_id == repo_id
                } else {
                    true
                }
            })
            .filter(|event| event.timestamp >= start_date && event.timestamp <= end_date)
            .collect();
        
        // Calculate analytics
        let total_events = filtered_events.len();
        let total_storage_bytes: u64 = filtered_events.iter()
            .filter(|event| matches!(event.event_type, UsageEventType::Ingress | UsageEventType::StorageOperation))
            .map(|event| event.size_bytes)
            .sum();
        
        let total_egress_bytes: u64 = filtered_events.iter()
            .filter(|event| matches!(event.event_type, UsageEventType::Egress))
            .map(|event| event.size_bytes)
            .sum();
        
        let total_api_requests = filtered_events.iter()
            .filter(|event| matches!(event.event_type, UsageEventType::ApiRequest))
            .count();
        
        let total_job_minutes: u64 = filtered_events.iter()
            .filter(|event| matches!(event.event_type, UsageEventType::JobProcessing))
            .map(|event| event.duration_minutes.unwrap_or(0))
            .sum();
        
        // Calculate costs
        let storage_cost = (total_storage_bytes as f64 / 1_000_000_000.0) * self.cost_config.storage_cost_per_gb_month;
        let egress_cost = (total_egress_bytes as f64 / 1_000_000_000.0) * self.cost_config.egress_cost_per_gb;
        let api_cost = total_api_requests as f64 * self.cost_config.api_cost_per_request;
        let job_cost = total_job_minutes as f64 * self.cost_config.job_cost_per_minute;
        
        Ok(UsageAnalytics {
            total_events,
            total_storage_bytes,
            total_egress_bytes,
            total_api_requests,
            total_job_minutes,
            storage_cost,
            egress_cost,
            api_cost,
            job_cost,
            total_cost: storage_cost + egress_cost + api_cost + job_cost,
            period_start: start_date,
            period_end: end_date,
        })
    }

    /// Generate cost report
    pub async fn generate_cost_report(&self, repo_id: Option<Uuid>) -> Result<CostReport, Box<dyn std::error::Error + Send + Sync>> {
        let estimates = self.cost_estimates.read().await;
        let alerts = self.budget_alerts.read().await;
        
        let repo_estimates: Vec<CostEstimate> = if let Some(repo_id) = repo_id {
            estimates.get(&repo_id).cloned().into_iter().collect()
        } else {
            estimates.values().cloned().collect()
        };
        
        let total_storage_cost: f64 = repo_estimates.iter().map(|e| e.storage_cost).sum();
        let total_egress_cost: f64 = repo_estimates.iter().map(|e| e.egress_cost).sum();
        let total_cost: f64 = repo_estimates.iter().map(|e| e.total_cost).sum();
        
        let active_alerts = alerts.iter().filter(|alert| !alert.resolved).count();
        
        Ok(CostReport {
            repo_estimates,
            total_storage_cost,
            total_egress_cost,
            total_cost,
            active_alerts,
            currency: self.cost_config.currency.clone(),
            generated_at: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalytics {
    pub total_events: usize,
    pub total_storage_bytes: u64,
    pub total_egress_bytes: u64,
    pub total_api_requests: usize,
    pub total_job_minutes: u64,
    pub storage_cost: f64,
    pub egress_cost: f64,
    pub api_cost: f64,
    pub job_cost: f64,
    pub total_cost: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostReport {
    pub repo_estimates: Vec<CostEstimate>,
    pub total_storage_cost: f64,
    pub total_egress_cost: f64,
    pub total_cost: f64,
    pub active_alerts: usize,
    pub currency: String,
    pub generated_at: DateTime<Utc>,
}
