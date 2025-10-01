// Access Review System
// Creates quarterly access review job to export user permissions
// Builds admin UI to acknowledge and manage access reviews

use axum::{
    extract::State,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration, NaiveDate};
use chrono_tz::Tz;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessReview {
    pub id: Uuid,
    pub review_period: ReviewPeriod,
    pub status: ReviewStatus,
    pub created_at: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub reviewers: Vec<Reviewer>,
    pub permissions: Vec<UserPermission>,
    pub review_summary: Option<ReviewSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewPeriod {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub quarter: u8,
    pub year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    InProgress,
    Completed,
    Overdue,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reviewer {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
    pub assigned_permissions: Vec<Uuid>,
    pub review_status: ReviewerStatus,
    pub review_notes: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewerStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermission {
    pub id: Uuid,
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub resource_name: String,
    pub permission: Permission,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub justification: Option<String>,
    pub review_decision: Option<ReviewDecision>,
    pub review_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Repository,
    Organization,
    Project,
    Dataset,
    Model,
    Job,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Admin,
    Execute,
    Delete,
    Share,
    Export,
    Import,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewDecision {
    Approved,
    Revoked,
    Modified,
    RequiresJustification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total_permissions: usize,
    pub approved_permissions: usize,
    pub revoked_permissions: usize,
    pub modified_permissions: usize,
    pub pending_permissions: usize,
    pub completion_percentage: f64,
    pub risk_score: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessReviewJob {
    pub id: Uuid,
    pub review_id: Uuid,
    pub job_type: JobType,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub progress_percentage: f64,
    pub result_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    ExportPermissions,
    GenerateReport,
    SendNotifications,
    UpdatePermissions,
    ArchiveReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessReviewConfiguration {
    pub id: Uuid,
    pub review_frequency: ReviewFrequency,
    pub auto_create_reviews: bool,
    pub notification_settings: NotificationSettings,
    pub review_assignments: Vec<ReviewAssignment>,
    pub risk_thresholds: RiskThresholds,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewFrequency {
    Quarterly,
    Monthly,
    Annually,
    Custom(Duration),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub slack_notifications: bool,
    pub webhook_notifications: bool,
    pub reminder_days: Vec<u8>,
    pub escalation_days: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAssignment {
    pub reviewer_id: Uuid,
    pub reviewer_name: String,
    pub reviewer_email: String,
    pub assigned_permissions: Vec<String>,
    pub review_deadline: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    pub high_risk_threshold: f64,
    pub medium_risk_threshold: f64,
    pub low_risk_threshold: f64,
    pub auto_revoke_threshold: f64,
}

pub struct AccessReviewService {
    reviews: Arc<RwLock<Vec<AccessReview>>>,
    jobs: Arc<RwLock<Vec<AccessReviewJob>>>,
    configurations: Arc<RwLock<Vec<AccessReviewConfiguration>>>,
    permissions_cache: Arc<RwLock<HashMap<String, Vec<UserPermission>>>>,
}

impl AccessReviewService {
    pub fn new() -> Self {
        Self {
            reviews: Arc::new(RwLock::new(Vec::new())),
            jobs: Arc::new(RwLock::new(Vec::new())),
            configurations: Arc::new(RwLock::new(Vec::new())),
            permissions_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new access review
    pub async fn create_access_review(
        &self,
        review_period: ReviewPeriod,
        reviewers: Vec<Reviewer>,
    ) -> Result<AccessReview, Box<dyn std::error::Error + Send + Sync>> {
        let review = AccessReview {
            id: Uuid::new_v4(),
            review_period: review_period.clone(),
            status: ReviewStatus::Pending,
            created_at: Utc::now(),
            due_date: Utc::now() + Duration::days(30), // 30 days from creation
            completed_at: None,
            reviewers,
            permissions: Vec::new(),
            review_summary: None,
        };

        // Store review
        let mut reviews = self.reviews.write().await;
        reviews.push(review.clone());

        // Create export job
        self.create_export_job(review.id).await?;

        Ok(review)
    }

    /// Create quarterly access review job
    pub async fn create_quarterly_review_job(&self) -> Result<AccessReviewJob, Box<dyn std::error::Error + Send + Sync>> {
        let current_date = Utc::now().date_naive();
        let quarter = ((current_date.month() - 1) / 3) + 1;
        let year = current_date.year();

        let review_period = ReviewPeriod {
            start_date: NaiveDate::from_ymd_opt(year, (quarter - 1) * 3 + 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year, quarter * 3, 1).unwrap(),
            quarter: quarter as u8,
            year,
        };

        // Get all user permissions
        let permissions = self.export_user_permissions().await?;

        // Create reviewers (simplified - in real implementation, get from config)
        let reviewers = self.get_reviewers_for_quarter().await?;

        // Create access review
        let review = self.create_access_review(review_period, reviewers).await?;

        // Create job
        let job = AccessReviewJob {
            id: Uuid::new_v4(),
            review_id: review.id,
            job_type: JobType::ExportPermissions,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            progress_percentage: 0.0,
            result_data: None,
        };

        let mut jobs = self.jobs.write().await;
        jobs.push(job.clone());

        Ok(job)
    }

    /// Export user permissions
    pub async fn export_user_permissions(&self) -> Result<Vec<UserPermission>, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would query the database
        // For now, we'll return mock data
        let permissions = vec![
            UserPermission {
                id: Uuid::new_v4(),
                user_id: "user1".to_string(),
                user_name: "John Doe".to_string(),
                user_email: "john.doe@example.com".to_string(),
                resource_type: ResourceType::Repository,
                resource_id: "repo1".to_string(),
                resource_name: "ml-experiments".to_string(),
                permission: Permission::Admin,
                granted_by: "admin".to_string(),
                granted_at: Utc::now() - Duration::days(90),
                last_used: Some(Utc::now() - Duration::days(7)),
                justification: Some("Project lead for ML experiments".to_string()),
                review_decision: None,
                review_notes: None,
            },
            UserPermission {
                id: Uuid::new_v4(),
                user_id: "user2".to_string(),
                user_name: "Jane Smith".to_string(),
                user_email: "jane.smith@example.com".to_string(),
                resource_type: ResourceType::Repository,
                resource_id: "repo2".to_string(),
                resource_name: "data-analysis".to_string(),
                permission: Permission::Read,
                granted_by: "admin".to_string(),
                granted_at: Utc::now() - Duration::days(60),
                last_used: Some(Utc::now() - Duration::days(30)),
                justification: Some("Data analyst access".to_string()),
                review_decision: None,
                review_notes: None,
            },
        ];

        Ok(permissions)
    }

    /// Get reviewers for quarter
    async fn get_reviewers_for_quarter(&self) -> Result<Vec<Reviewer>, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would get from configuration
        Ok(vec![
            Reviewer {
                id: Uuid::new_v4(),
                name: "Admin User".to_string(),
                email: "admin@example.com".to_string(),
                role: "Security Admin".to_string(),
                assigned_permissions: Vec::new(),
                review_status: ReviewerStatus::Pending,
                review_notes: None,
                reviewed_at: None,
            },
        ])
    }

    /// Create export job
    async fn create_export_job(&self, review_id: Uuid) -> Result<AccessReviewJob, Box<dyn std::error::Error + Send + Sync>> {
        let job = AccessReviewJob {
            id: Uuid::new_v4(),
            review_id,
            job_type: JobType::ExportPermissions,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            progress_percentage: 0.0,
            result_data: None,
        };

        let mut jobs = self.jobs.write().await;
        jobs.push(job.clone());

        Ok(job)
    }

    /// Get access review by ID
    pub async fn get_access_review(&self, review_id: Uuid) -> Result<Option<AccessReview>, Box<dyn std::error::Error + Send + Sync>> {
        let reviews = self.reviews.read().await;
        Ok(reviews.iter().find(|r| r.id == review_id).cloned())
    }

    /// Get all access reviews
    pub async fn get_all_access_reviews(&self) -> Result<Vec<AccessReview>, Box<dyn std::error::Error + Send + Sync>> {
        let reviews = self.reviews.read().await;
        Ok(reviews.clone())
    }

    /// Update review status
    pub async fn update_review_status(
        &self,
        review_id: Uuid,
        status: ReviewStatus,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut reviews = self.reviews.write().await;
        if let Some(review) = reviews.iter_mut().find(|r| r.id == review_id) {
            review.status = status;
            if matches!(status, ReviewStatus::Completed) {
                review.completed_at = Some(Utc::now());
            }
        }
        Ok(())
    }

    /// Submit reviewer decision
    pub async fn submit_reviewer_decision(
        &self,
        review_id: Uuid,
        reviewer_id: Uuid,
        permission_id: Uuid,
        decision: ReviewDecision,
        notes: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut reviews = self.reviews.write().await;
        if let Some(review) = reviews.iter_mut().find(|r| r.id == review_id) {
            // Update permission decision
            if let Some(permission) = review.permissions.iter_mut().find(|p| p.id == permission_id) {
                permission.review_decision = Some(decision);
                permission.review_notes = notes;
            }

            // Update reviewer status
            if let Some(reviewer) = review.reviewers.iter_mut().find(|r| r.id == reviewer_id) {
                reviewer.review_status = ReviewerStatus::Completed;
                reviewer.reviewed_at = Some(Utc::now());
            }
        }
        Ok(())
    }

    /// Generate review summary
    pub async fn generate_review_summary(&self, review_id: Uuid) -> Result<ReviewSummary, Box<dyn std::error::Error + Send + Sync>> {
        let reviews = self.reviews.read().await;
        let review = reviews.iter().find(|r| r.id == review_id)
            .ok_or("Review not found")?;

        let total_permissions = review.permissions.len();
        let approved_permissions = review.permissions.iter()
            .filter(|p| matches!(p.review_decision, Some(ReviewDecision::Approved)))
            .count();
        let revoked_permissions = review.permissions.iter()
            .filter(|p| matches!(p.review_decision, Some(ReviewDecision::Revoked)))
            .count();
        let modified_permissions = review.permissions.iter()
            .filter(|p| matches!(p.review_decision, Some(ReviewDecision::Modified)))
            .count();
        let pending_permissions = total_permissions - approved_permissions - revoked_permissions - modified_permissions;

        let completion_percentage = if total_permissions > 0 {
            (approved_permissions + revoked_permissions + modified_permissions) as f64 / total_permissions as f64 * 100.0
        } else {
            0.0
        };

        let risk_score = self.calculate_risk_score(&review.permissions).await;

        let recommendations = self.generate_recommendations(&review.permissions).await;

        let summary = ReviewSummary {
            total_permissions,
            approved_permissions,
            revoked_permissions,
            modified_permissions,
            pending_permissions,
            completion_percentage,
            risk_score,
            recommendations,
        };

        // Update review with summary
        let mut reviews = self.reviews.write().await;
        if let Some(review) = reviews.iter_mut().find(|r| r.id == review_id) {
            review.review_summary = Some(summary.clone());
        }

        Ok(summary)
    }

    /// Calculate risk score
    async fn calculate_risk_score(&self, permissions: &[UserPermission]) -> f64 {
        let mut risk_score = 0.0;
        let total_permissions = permissions.len() as f64;

        for permission in permissions {
            let mut permission_risk = 0.0;

            // Risk based on permission type
            match permission.permission {
                Permission::Admin => permission_risk += 0.8,
                Permission::Delete => permission_risk += 0.7,
                Permission::Write => permission_risk += 0.5,
                Permission::Read => permission_risk += 0.2,
                _ => permission_risk += 0.3,
            }

            // Risk based on last usage
            if let Some(last_used) = permission.last_used {
                let days_since_use = (Utc::now() - last_used).num_days();
                if days_since_use > 90 {
                    permission_risk += 0.3;
                } else if days_since_use > 30 {
                    permission_risk += 0.1;
                }
            } else {
                permission_risk += 0.5; // Never used
            }

            // Risk based on justification
            if permission.justification.is_none() {
                permission_risk += 0.2;
            }

            risk_score += permission_risk;
        }

        if total_permissions > 0.0 {
            risk_score / total_permissions
        } else {
            0.0
        }
    }

    /// Generate recommendations
    async fn generate_recommendations(&self, permissions: &[UserPermission]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let unused_permissions = permissions.iter()
            .filter(|p| p.last_used.is_none() || 
                       (Utc::now() - p.last_used.unwrap()).num_days() > 90)
            .count();

        if unused_permissions > 0 {
            recommendations.push(format!("Consider revoking {} unused permissions", unused_permissions));
        }

        let admin_permissions = permissions.iter()
            .filter(|p| matches!(p.permission, Permission::Admin))
            .count();

        if admin_permissions > 5 {
            recommendations.push("High number of admin permissions detected - consider reviewing necessity".to_string());
        }

        let permissions_without_justification = permissions.iter()
            .filter(|p| p.justification.is_none())
            .count();

        if permissions_without_justification > 0 {
            recommendations.push(format!("{} permissions lack justification - require documentation", permissions_without_justification));
        }

        if recommendations.is_empty() {
            recommendations.push("No specific recommendations at this time".to_string());
        }

        recommendations
    }

    /// Get review statistics
    pub async fn get_review_statistics(&self) -> Result<ReviewStatistics, Box<dyn std::error::Error + Send + Sync>> {
        let reviews = self.reviews.read().await;
        let jobs = self.jobs.read().await;

        let total_reviews = reviews.len();
        let completed_reviews = reviews.iter().filter(|r| matches!(r.status, ReviewStatus::Completed)).count();
        let pending_reviews = reviews.iter().filter(|r| matches!(r.status, ReviewStatus::Pending)).count();
        let overdue_reviews = reviews.iter().filter(|r| matches!(r.status, ReviewStatus::Overdue)).count();

        let total_jobs = jobs.len();
        let completed_jobs = jobs.iter().filter(|j| matches!(j.status, JobStatus::Completed)).count();
        let failed_jobs = jobs.iter().filter(|j| matches!(j.status, JobStatus::Failed)).count();

        Ok(ReviewStatistics {
            total_reviews,
            completed_reviews,
            pending_reviews,
            overdue_reviews,
            total_jobs,
            completed_jobs,
            failed_jobs,
            completion_rate: if total_reviews > 0 { completed_reviews as f64 / total_reviews as f64 } else { 0.0 },
            job_success_rate: if total_jobs > 0 { completed_jobs as f64 / total_jobs as f64 } else { 0.0 },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStatistics {
    pub total_reviews: usize,
    pub completed_reviews: usize,
    pub pending_reviews: usize,
    pub overdue_reviews: usize,
    pub total_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
    pub completion_rate: f64,
    pub job_success_rate: f64,
}

/// Access review router
pub fn access_review_router() -> Router {
    Router::new()
        .route("/access-reviews", post(create_access_review))
        .route("/access-reviews/quarterly", post(create_quarterly_review))
        .route("/access-reviews", get(get_all_access_reviews))
        .route("/access-reviews/:review_id", get(get_access_review))
        .route("/access-reviews/:review_id/status", put(update_review_status))
        .route("/access-reviews/:review_id/decision", post(submit_reviewer_decision))
        .route("/access-reviews/:review_id/summary", get(generate_review_summary))
        .route("/access-reviews/statistics", get(get_review_statistics))
}

/// Create access review
async fn create_access_review(
    State(service): State<Arc<AccessReviewService>>,
    Json(request): Json<CreateAccessReviewRequest>,
) -> Result<Json<AccessReview>, String> {
    let review = service.create_access_review(request.review_period, request.reviewers).await
        .map_err(|e| format!("Failed to create access review: {}", e))?;
    
    Ok(Json(review))
}

/// Create quarterly review
async fn create_quarterly_review(
    State(service): State<Arc<AccessReviewService>>,
) -> Result<Json<AccessReviewJob>, String> {
    let job = service.create_quarterly_review_job().await
        .map_err(|e| format!("Failed to create quarterly review: {}", e))?;
    
    Ok(Json(job))
}

/// Get all access reviews
async fn get_all_access_reviews(
    State(service): State<Arc<AccessReviewService>>,
) -> Result<Json<Vec<AccessReview>>, String> {
    let reviews = service.get_all_access_reviews().await
        .map_err(|e| format!("Failed to get access reviews: {}", e))?;
    
    Ok(Json(reviews))
}

/// Get access review
async fn get_access_review(
    State(service): State<Arc<AccessReviewService>>,
    axum::extract::Path(review_id): axum::extract::Path<Uuid>,
) -> Result<Json<AccessReview>, String> {
    let review = service.get_access_review(review_id).await
        .map_err(|e| format!("Failed to get access review: {}", e))?
        .ok_or("Review not found")?;
    
    Ok(Json(review))
}

/// Update review status
async fn update_review_status(
    State(service): State<Arc<AccessReviewService>>,
    axum::extract::Path(review_id): axum::extract::Path<Uuid>,
    Json(request): Json<UpdateReviewStatusRequest>,
) -> Result<Json<serde_json::Value>, String> {
    service.update_review_status(review_id, request.status).await
        .map_err(|e| format!("Failed to update review status: {}", e))?;
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Review status updated successfully"
    })))
}

/// Submit reviewer decision
async fn submit_reviewer_decision(
    State(service): State<Arc<AccessReviewService>>,
    axum::extract::Path(review_id): axum::extract::Path<Uuid>,
    Json(request): Json<SubmitReviewerDecisionRequest>,
) -> Result<Json<serde_json::Value>, String> {
    service.submit_reviewer_decision(
        review_id,
        request.reviewer_id,
        request.permission_id,
        request.decision,
        request.notes,
    ).await
        .map_err(|e| format!("Failed to submit reviewer decision: {}", e))?;
    
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Reviewer decision submitted successfully"
    })))
}

/// Generate review summary
async fn generate_review_summary(
    State(service): State<Arc<AccessReviewService>>,
    axum::extract::Path(review_id): axum::extract::Path<Uuid>,
) -> Result<Json<ReviewSummary>, String> {
    let summary = service.generate_review_summary(review_id).await
        .map_err(|e| format!("Failed to generate review summary: {}", e))?;
    
    Ok(Json(summary))
}

/// Get review statistics
async fn get_review_statistics(
    State(service): State<Arc<AccessReviewService>>,
) -> Result<Json<ReviewStatistics>, String> {
    let stats = service.get_review_statistics().await
        .map_err(|e| format!("Failed to get review statistics: {}", e))?;
    
    Ok(Json(stats))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccessReviewRequest {
    pub review_period: ReviewPeriod,
    pub reviewers: Vec<Reviewer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReviewStatusRequest {
    pub status: ReviewStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitReviewerDecisionRequest {
    pub reviewer_id: Uuid,
    pub permission_id: Uuid,
    pub decision: ReviewDecision,
    pub notes: Option<String>,
}
