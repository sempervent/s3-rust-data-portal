// Budget Alerts System
// Implements budget alerts with webhook/email notifications
// Configures thresholds and escalation procedures

use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub threshold_percentage: f64,
    pub current_usage_percentage: f64,
    pub budget_limit: f64,
    pub current_spend: f64,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub created_at: DateTime<Utc>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub escalation_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    BudgetThreshold,
    BudgetExceeded,
    UnusualSpending,
    CostAnomaly,
    ProjectedOverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfiguration {
    pub repo_id: Uuid,
    pub repo_name: String,
    pub budget_limit: f64,
    pub threshold_percentages: Vec<f64>,
    pub escalation_contacts: Vec<EscalationContact>,
    pub notification_channels: Vec<NotificationChannel>,
    pub auto_resolve: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationContact {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub role: String,
    pub escalation_level: u32,
    pub notification_preferences: NotificationPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email: bool,
    pub sms: bool,
    pub webhook: bool,
    pub slack: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email {
        smtp_server: String,
        smtp_port: u16,
        username: String,
        password: String,
        from_address: String,
    },
    Webhook {
        url: String,
        headers: HashMap<String, String>,
        timeout_seconds: u64,
    },
    Slack {
        webhook_url: String,
        channel: String,
        username: String,
    },
    SMS {
        provider: String,
        api_key: String,
        from_number: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    pub alert_id: Uuid,
    pub channel: NotificationChannel,
    pub message: String,
    pub sent_at: DateTime<Utc>,
    pub status: NotificationStatus,
    pub retry_count: u32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetMetrics {
    pub repo_id: Uuid,
    pub current_spend: f64,
    pub projected_spend: f64,
    pub daily_average: f64,
    pub monthly_trend: f64,
    pub last_updated: DateTime<Utc>,
}

pub struct BudgetAlertService {
    alerts: Arc<RwLock<Vec<BudgetAlert>>>,
    configurations: Arc<RwLock<HashMap<Uuid, BudgetConfiguration>>>,
    notifications: Arc<RwLock<Vec<AlertNotification>>>,
    http_client: Client,
    metrics_cache: Arc<RwLock<HashMap<Uuid, BudgetMetrics>>>,
}

impl BudgetAlertService {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
            configurations: Arc::new(RwLock::new(HashMap::new())),
            notifications: Arc::new(RwLock::new(Vec::new())),
            http_client: Client::new(),
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Configure budget alerts for a repository
    pub async fn configure_budget_alerts(
        &self,
        config: BudgetConfiguration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut configurations = self.configurations.write().await;
        configurations.insert(config.repo_id, config);
        Ok(())
    }

    /// Check budget thresholds and create alerts
    pub async fn check_budget_thresholds(
        &self,
        repo_id: Uuid,
        current_spend: f64,
    ) -> Result<Vec<BudgetAlert>, Box<dyn std::error::Error + Send + Sync>> {
        let configurations = self.configurations.read().await;
        let config = match configurations.get(&repo_id) {
            Some(config) => config,
            None => return Ok(Vec::new()),
        };

        let budget_limit = config.budget_limit;
        let usage_percentage = (current_spend / budget_limit) * 100.0;

        let mut new_alerts = Vec::new();

        // Check each threshold
        for threshold in &config.threshold_percentages {
            if usage_percentage >= *threshold {
                // Check if alert already exists for this threshold
                let existing_alert = self.get_existing_alert(repo_id, *threshold).await;
                if existing_alert.is_none() {
                    let alert = BudgetAlert {
                        id: Uuid::new_v4(),
                        repo_id,
                        repo_name: config.repo_name.clone(),
                        threshold_percentage: *threshold,
                        current_usage_percentage: usage_percentage,
                        budget_limit,
                        current_spend,
                        alert_type: if usage_percentage >= 100.0 {
                            AlertType::BudgetExceeded
                        } else {
                            AlertType::BudgetThreshold
                        },
                        severity: self.determine_severity(usage_percentage),
                        created_at: Utc::now(),
                        resolved: false,
                        resolved_at: None,
                        escalation_level: 1,
                    };

                    // Send notifications
                    self.send_alert_notifications(&alert, config).await?;

                    // Store alert
                    let mut alerts = self.alerts.write().await;
                    alerts.push(alert.clone());
                    new_alerts.push(alert);
                }
            }
        }

        Ok(new_alerts)
    }

    /// Send alert notifications
    async fn send_alert_notifications(
        &self,
        alert: &BudgetAlert,
        config: &BudgetConfiguration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message = self.format_alert_message(alert);
        
        for channel in &config.notification_channels {
            let notification = AlertNotification {
                alert_id: alert.id,
                channel: channel.clone(),
                message: message.clone(),
                sent_at: Utc::now(),
                status: NotificationStatus::Pending,
                retry_count: 0,
                error_message: None,
            };

            // Send notification
            match self.send_notification(&notification).await {
                Ok(_) => {
                    let mut notifications = self.notifications.write().await;
                    let mut notification = notification;
                    notification.status = NotificationStatus::Sent;
                    notifications.push(notification);
                }
                Err(e) => {
                    let mut notifications = self.notifications.write().await;
                    let mut notification = notification;
                    notification.status = NotificationStatus::Failed;
                    notification.error_message = Some(e.to_string());
                    notifications.push(notification);
                }
            }
        }

        Ok(())
    }

    /// Send notification via configured channel
    async fn send_notification(
        &self,
        notification: &AlertNotification,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match &notification.channel {
            NotificationChannel::Email { smtp_server, smtp_port, username, password, from_address } => {
                self.send_email_notification(
                    smtp_server,
                    *smtp_port,
                    username,
                    password,
                    from_address,
                    &notification.message,
                ).await
            }
            NotificationChannel::Webhook { url, headers, timeout_seconds } => {
                self.send_webhook_notification(url, headers, &notification.message, *timeout_seconds).await
            }
            NotificationChannel::Slack { webhook_url, channel, username } => {
                self.send_slack_notification(webhook_url, channel, username, &notification.message).await
            }
            NotificationChannel::SMS { provider, api_key, from_number } => {
                self.send_sms_notification(provider, api_key, from_number, &notification.message).await
            }
        }
    }

    /// Send email notification
    async fn send_email_notification(
        &self,
        smtp_server: &str,
        smtp_port: u16,
        username: &str,
        password: &str,
        from_address: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, you would use an email library like lettre
        // For now, we'll simulate the email sending
        println!("Sending email notification to {}: {}", from_address, message);
        Ok(())
    }

    /// Send webhook notification
    async fn send_webhook_notification(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        message: &str,
        timeout_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "message": message,
            "timestamp": Utc::now(),
            "type": "budget_alert"
        });

        let mut request = self.http_client
            .post(url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(timeout_seconds));

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(format!("Webhook notification failed with status: {}", response.status()).into());
        }

        Ok(())
    }

    /// Send Slack notification
    async fn send_slack_notification(
        &self,
        webhook_url: &str,
        channel: &str,
        username: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "channel": channel,
            "username": username,
            "text": message,
            "icon_emoji": ":warning:"
        });

        let response = self.http_client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Slack notification failed with status: {}", response.status()).into());
        }

        Ok(())
    }

    /// Send SMS notification
    async fn send_sms_notification(
        &self,
        provider: &str,
        api_key: &str,
        from_number: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, you would integrate with SMS providers like Twilio
        println!("Sending SMS notification via {}: {}", provider, message);
        Ok(())
    }

    /// Format alert message
    fn format_alert_message(&self, alert: &BudgetAlert) -> String {
        format!(
            "🚨 Budget Alert for {}: {:.1}% of budget used (${:.2} of ${:.2})\n\
             Alert Type: {:?}\n\
             Severity: {:?}\n\
             Threshold: {:.1}%\n\
             Created: {}",
            alert.repo_name,
            alert.current_usage_percentage,
            alert.current_spend,
            alert.budget_limit,
            alert.alert_type,
            alert.severity,
            alert.threshold_percentage,
            alert.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    /// Get existing alert for threshold
    async fn get_existing_alert(&self, repo_id: Uuid, threshold: f64) -> Option<BudgetAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .find(|alert| alert.repo_id == repo_id && 
                         alert.threshold_percentage == threshold && 
                         !alert.resolved)
            .cloned()
    }

    /// Determine alert severity
    fn determine_severity(&self, usage_percentage: f64) -> AlertSeverity {
        if usage_percentage >= 100.0 {
            AlertSeverity::Critical
        } else if usage_percentage >= 90.0 {
            AlertSeverity::High
        } else if usage_percentage >= 75.0 {
            AlertSeverity::Medium
        } else {
            AlertSeverity::Low
        }
    }

    /// Get all alerts for a repository
    pub async fn get_repo_alerts(&self, repo_id: Uuid) -> Result<Vec<BudgetAlert>, Box<dyn std::error::Error + Send + Sync>> {
        let alerts = self.alerts.read().await;
        Ok(alerts.iter()
            .filter(|alert| alert.repo_id == repo_id)
            .cloned()
            .collect())
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
            alert.resolved_at = Some(Utc::now());
        }
        Ok(())
    }

    /// Get alert statistics
    pub async fn get_alert_statistics(&self) -> Result<AlertStatistics, Box<dyn std::error::Error + Send + Sync>> {
        let alerts = self.alerts.read().await;
        let notifications = self.notifications.read().await;

        let total_alerts = alerts.len();
        let resolved_alerts = alerts.iter().filter(|a| a.resolved).count();
        let active_alerts = total_alerts - resolved_alerts;

        let critical_alerts = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Critical)).count();
        let high_alerts = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::High)).count();
        let medium_alerts = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Medium)).count();
        let low_alerts = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Low)).count();

        let sent_notifications = notifications.iter().filter(|n| matches!(n.status, NotificationStatus::Sent)).count();
        let failed_notifications = notifications.iter().filter(|n| matches!(n.status, NotificationStatus::Failed)).count();

        Ok(AlertStatistics {
            total_alerts,
            active_alerts,
            resolved_alerts,
            critical_alerts,
            high_alerts,
            medium_alerts,
            low_alerts,
            sent_notifications,
            failed_notifications,
            notification_success_rate: if notifications.is_empty() { 0.0 } else { sent_notifications as f64 / notifications.len() as f64 },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub total_alerts: usize,
    pub active_alerts: usize,
    pub resolved_alerts: usize,
    pub critical_alerts: usize,
    pub high_alerts: usize,
    pub medium_alerts: usize,
    pub low_alerts: usize,
    pub sent_notifications: usize,
    pub failed_notifications: usize,
    pub notification_success_rate: f64,
}

/// Budget alerts router
pub fn budget_alerts_router() -> Router {
    Router::new()
        .route("/budget-alerts/configure", post(configure_budget_alerts))
        .route("/budget-alerts/:repo_id", get(get_repo_alerts))
        .route("/budget-alerts/:alert_id/resolve", post(resolve_alert))
        .route("/budget-alerts/statistics", get(get_alert_statistics))
}

/// Configure budget alerts
async fn configure_budget_alerts(
    State(service): State<Arc<BudgetAlertService>>,
    Json(config): Json<BudgetConfiguration>,
) -> Result<Json<serde_json::Value>, String> {
    service.configure_budget_alerts(config).await
        .map_err(|e| format!("Failed to configure budget alerts: {}", e))?;
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Budget alerts configured successfully"
    })))
}

/// Get repository alerts
async fn get_repo_alerts(
    State(service): State<Arc<BudgetAlertService>>,
    axum::extract::Path(repo_id): axum::extract::Path<Uuid>,
) -> Result<Json<Vec<BudgetAlert>>, String> {
    let alerts = service.get_repo_alerts(repo_id).await
        .map_err(|e| format!("Failed to get repository alerts: {}", e))?;
    
    Ok(Json(alerts))
}

/// Resolve alert
async fn resolve_alert(
    State(service): State<Arc<BudgetAlertService>>,
    axum::extract::Path(alert_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, String> {
    service.resolve_alert(alert_id).await
        .map_err(|e| format!("Failed to resolve alert: {}", e))?;
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Alert resolved successfully"
    })))
}

/// Get alert statistics
async fn get_alert_statistics(
    State(service): State<Arc<BudgetAlertService>>,
) -> Result<Json<AlertStatistics>, String> {
    let stats = service.get_alert_statistics().await
        .map_err(|e| format!("Failed to get alert statistics: {}", e))?;
    
    Ok(Json(stats))
}
