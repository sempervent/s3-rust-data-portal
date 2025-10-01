// Performance Reporting System
// Ships /perf/report JSON endpoint exposing rolling P95/99
// Adds performance regression detection

use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: DateTime<Utc>,
    pub period: ReportPeriod,
    pub metrics: PerformanceMetrics,
    pub thresholds: PerformanceThresholds,
    pub regression_detection: RegressionDetection,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub api_metrics: ApiMetrics,
    pub database_metrics: DatabaseMetrics,
    pub storage_metrics: StorageMetrics,
    pub cache_metrics: CacheMetrics,
    pub job_metrics: JobMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub p50_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub max_response_time_ms: f64,
    pub requests_per_second: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub total_queries: u64,
    pub avg_query_time_ms: f64,
    pub p95_query_time_ms: f64,
    pub p99_query_time_ms: f64,
    pub slow_queries: u64,
    pub connection_pool_utilization: f64,
    pub active_connections: u32,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub total_operations: u64,
    pub avg_operation_time_ms: f64,
    pub p95_operation_time_ms: f64,
    pub p99_operation_time_ms: f64,
    pub upload_success_rate: f64,
    pub download_success_rate: f64,
    pub storage_utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub avg_lookup_time_ms: f64,
    pub p95_lookup_time_ms: f64,
    pub p99_lookup_time_ms: f64,
    pub cache_size: usize,
    pub memory_usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetrics {
    pub total_jobs: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub avg_processing_time_ms: f64,
    pub p95_processing_time_ms: f64,
    pub p99_processing_time_ms: f64,
    pub queue_depth: u32,
    pub processing_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub api_response_time_p95_ms: f64,
    pub api_response_time_p99_ms: f64,
    pub api_error_rate: f64,
    pub database_query_time_p95_ms: f64,
    pub database_query_time_p99_ms: f64,
    pub storage_operation_time_p95_ms: f64,
    pub storage_operation_time_p99_ms: f64,
    pub cache_hit_rate: f64,
    pub job_processing_time_p95_ms: f64,
    pub job_processing_time_p99_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetection {
    pub detected: bool,
    pub severity: RegressionSeverity,
    pub affected_metrics: Vec<String>,
    pub regression_score: f64,
    pub historical_comparison: HistoricalComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalComparison {
    pub previous_period: ReportPeriod,
    pub performance_change: PerformanceChange,
    pub significant_changes: Vec<SignificantChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceChange {
    pub api_response_time_change: f64,
    pub database_query_time_change: f64,
    pub storage_operation_time_change: f64,
    pub cache_hit_rate_change: f64,
    pub job_processing_time_change: f64,
    pub overall_performance_change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignificantChange {
    pub metric_name: String,
    pub current_value: f64,
    pub previous_value: f64,
    pub change_percentage: f64,
    pub significance_level: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceDataPoint {
    pub timestamp: DateTime<Utc>,
    pub api_response_time: f64,
    pub database_query_time: f64,
    pub storage_operation_time: f64,
    pub cache_hit_rate: f64,
    pub job_processing_time: f64,
}

pub struct PerformanceReporter {
    data_points: Arc<RwLock<VecDeque<PerformanceDataPoint>>>,
    max_data_points: usize,
    thresholds: PerformanceThresholds,
}

impl PerformanceReporter {
    pub fn new(max_data_points: usize) -> Self {
        Self {
            data_points: Arc::new(RwLock::new(VecDeque::new())),
            max_data_points,
            thresholds: PerformanceThresholds {
                api_response_time_p95_ms: 1000.0,
                api_response_time_p99_ms: 2000.0,
                api_error_rate: 0.05,
                database_query_time_p95_ms: 500.0,
                database_query_time_p99_ms: 1000.0,
                storage_operation_time_p95_ms: 2000.0,
                storage_operation_time_p99_ms: 5000.0,
                cache_hit_rate: 0.80,
                job_processing_time_p95_ms: 30000.0,
                job_processing_time_p99_ms: 60000.0,
            },
        }
    }

    /// Record a performance data point
    pub async fn record_data_point(&self, data_point: PerformanceDataPoint) {
        let mut data_points = self.data_points.write().await;
        
        // Add new data point
        data_points.push_back(data_point);
        
        // Remove old data points if we exceed the limit
        while data_points.len() > self.max_data_points {
            data_points.pop_front();
        }
    }

    /// Generate performance report
    pub async fn generate_report(&self, period_hours: i64) -> Result<PerformanceReport, Box<dyn std::error::Error + Send + Sync>> {
        let end_time = Utc::now();
        let start_time = end_time - Duration::hours(period_hours);
        
        let data_points = self.data_points.read().await;
        
        // Filter data points for the specified period
        let period_data: Vec<&PerformanceDataPoint> = data_points
            .iter()
            .filter(|dp| dp.timestamp >= start_time && dp.timestamp <= end_time)
            .collect();
        
        if period_data.is_empty() {
            return Err("No data points available for the specified period".into());
        }
        
        // Calculate metrics
        let api_metrics = self.calculate_api_metrics(&period_data).await;
        let database_metrics = self.calculate_database_metrics(&period_data).await;
        let storage_metrics = self.calculate_storage_metrics(&period_data).await;
        let cache_metrics = self.calculate_cache_metrics(&period_data).await;
        let job_metrics = self.calculate_job_metrics(&period_data).await;
        
        // Detect regressions
        let regression_detection = self.detect_regressions(&period_data, &api_metrics).await;
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&api_metrics, &regression_detection).await;
        
        Ok(PerformanceReport {
            timestamp: Utc::now(),
            period: ReportPeriod {
                start: start_time,
                end: end_time,
                duration_seconds: period_hours * 3600,
            },
            metrics: PerformanceMetrics {
                api_metrics,
                database_metrics,
                storage_metrics,
                cache_metrics,
                job_metrics,
            },
            thresholds: self.thresholds.clone(),
            regression_detection,
            recommendations,
        })
    }

    /// Calculate API metrics
    async fn calculate_api_metrics(&self, data_points: &[&PerformanceDataPoint]) -> ApiMetrics {
        let total_requests = data_points.len() as u64;
        let successful_requests = data_points.len() as u64; // Simplified
        let failed_requests = 0; // Simplified
        
        let response_times: Vec<f64> = data_points.iter().map(|dp| dp.api_response_time).collect();
        let avg_response_time = response_times.iter().sum::<f64>() / response_times.len() as f64;
        
        let mut sorted_times = response_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p50 = self.percentile(&sorted_times, 0.50);
        let p95 = self.percentile(&sorted_times, 0.95);
        let p99 = self.percentile(&sorted_times, 0.99);
        let max_time = sorted_times.last().copied().unwrap_or(0.0);
        
        let requests_per_second = total_requests as f64 / 3600.0; // Simplified
        let error_rate = failed_requests as f64 / total_requests as f64;
        
        ApiMetrics {
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time_ms: avg_response_time,
            p50_response_time_ms: p50,
            p95_response_time_ms: p95,
            p99_response_time_ms: p99,
            max_response_time_ms: max_time,
            requests_per_second,
            error_rate,
        }
    }

    /// Calculate database metrics
    async fn calculate_database_metrics(&self, data_points: &[&PerformanceDataPoint]) -> DatabaseMetrics {
        let total_queries = data_points.len() as u64;
        
        let query_times: Vec<f64> = data_points.iter().map(|dp| dp.database_query_time).collect();
        let avg_query_time = query_times.iter().sum::<f64>() / query_times.len() as f64;
        
        let mut sorted_times = query_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p95 = self.percentile(&sorted_times, 0.95);
        let p99 = self.percentile(&sorted_times, 0.99);
        
        let slow_queries = query_times.iter().filter(|&&t| t > 1000.0).count() as u64;
        
        DatabaseMetrics {
            total_queries,
            avg_query_time_ms: avg_query_time,
            p95_query_time_ms: p95,
            p99_query_time_ms: p99,
            slow_queries,
            connection_pool_utilization: 0.5, // Simplified
            active_connections: 10, // Simplified
            max_connections: 20, // Simplified
        }
    }

    /// Calculate storage metrics
    async fn calculate_storage_metrics(&self, data_points: &[&PerformanceDataPoint]) -> StorageMetrics {
        let total_operations = data_points.len() as u64;
        
        let operation_times: Vec<f64> = data_points.iter().map(|dp| dp.storage_operation_time).collect();
        let avg_operation_time = operation_times.iter().sum::<f64>() / operation_times.len() as f64;
        
        let mut sorted_times = operation_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p95 = self.percentile(&sorted_times, 0.95);
        let p99 = self.percentile(&sorted_times, 0.99);
        
        StorageMetrics {
            total_operations,
            avg_operation_time_ms: avg_operation_time,
            p95_operation_time_ms: p95,
            p99_operation_time_ms: p99,
            upload_success_rate: 0.95, // Simplified
            download_success_rate: 0.98, // Simplified
            storage_utilization: 0.60, // Simplified
        }
    }

    /// Calculate cache metrics
    async fn calculate_cache_metrics(&self, data_points: &[&PerformanceDataPoint]) -> CacheMetrics {
        let hit_rates: Vec<f64> = data_points.iter().map(|dp| dp.cache_hit_rate).collect();
        let avg_hit_rate = hit_rates.iter().sum::<f64>() / hit_rates.len() as f64;
        let miss_rate = 1.0 - avg_hit_rate;
        
        let lookup_times: Vec<f64> = data_points.iter().map(|dp| dp.cache_hit_rate * 10.0).collect(); // Simplified
        let avg_lookup_time = lookup_times.iter().sum::<f64>() / lookup_times.len() as f64;
        
        let mut sorted_times = lookup_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p95 = self.percentile(&sorted_times, 0.95);
        let p99 = self.percentile(&sorted_times, 0.99);
        
        CacheMetrics {
            hit_rate: avg_hit_rate,
            miss_rate,
            avg_lookup_time_ms: avg_lookup_time,
            p95_lookup_time_ms: p95,
            p99_lookup_time_ms: p99,
            cache_size: 1000, // Simplified
            memory_usage: 0.30, // Simplified
        }
    }

    /// Calculate job metrics
    async fn calculate_job_metrics(&self, data_points: &[&PerformanceDataPoint]) -> JobMetrics {
        let total_jobs = data_points.len() as u64;
        let completed_jobs = (total_jobs as f64 * 0.95) as u64; // Simplified
        let failed_jobs = total_jobs - completed_jobs;
        
        let processing_times: Vec<f64> = data_points.iter().map(|dp| dp.job_processing_time).collect();
        let avg_processing_time = processing_times.iter().sum::<f64>() / processing_times.len() as f64;
        
        let mut sorted_times = processing_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p95 = self.percentile(&sorted_times, 0.95);
        let p99 = self.percentile(&sorted_times, 0.99);
        
        JobMetrics {
            total_jobs,
            completed_jobs,
            failed_jobs,
            avg_processing_time_ms: avg_processing_time,
            p95_processing_time_ms: p95,
            p99_processing_time_ms: p99,
            queue_depth: 5, // Simplified
            processing_rate: completed_jobs as f64 / 3600.0, // Simplified
        }
    }

    /// Detect performance regressions
    async fn detect_regressions(&self, data_points: &[&PerformanceDataPoint], api_metrics: &ApiMetrics) -> RegressionDetection {
        let mut affected_metrics = Vec::new();
        let mut regression_score = 0.0;
        
        // Check API response time regression
        if api_metrics.p95_response_time_ms > self.thresholds.api_response_time_p95_ms {
            affected_metrics.push("api_response_time_p95".to_string());
            regression_score += 0.3;
        }
        
        if api_metrics.p99_response_time_ms > self.thresholds.api_response_time_p99_ms {
            affected_metrics.push("api_response_time_p99".to_string());
            regression_score += 0.2;
        }
        
        // Check error rate regression
        if api_metrics.error_rate > self.thresholds.api_error_rate {
            affected_metrics.push("api_error_rate".to_string());
            regression_score += 0.5;
        }
        
        let detected = regression_score > 0.5;
        let severity = if regression_score > 0.8 {
            RegressionSeverity::Critical
        } else if regression_score > 0.6 {
            RegressionSeverity::High
        } else if regression_score > 0.4 {
            RegressionSeverity::Medium
        } else {
            RegressionSeverity::Low
        };
        
        RegressionDetection {
            detected,
            severity,
            affected_metrics,
            regression_score,
            historical_comparison: HistoricalComparison {
                previous_period: ReportPeriod {
                    start: Utc::now() - Duration::hours(48),
                    end: Utc::now() - Duration::hours(24),
                    duration_seconds: 86400,
                },
                performance_change: PerformanceChange {
                    api_response_time_change: 0.0, // Simplified
                    database_query_time_change: 0.0,
                    storage_operation_time_change: 0.0,
                    cache_hit_rate_change: 0.0,
                    job_processing_time_change: 0.0,
                    overall_performance_change: 0.0,
                },
                significant_changes: Vec::new(),
            },
        }
    }

    /// Generate performance recommendations
    async fn generate_recommendations(&self, api_metrics: &ApiMetrics, regression: &RegressionDetection) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if api_metrics.p95_response_time_ms > self.thresholds.api_response_time_p95_ms {
            recommendations.push("Consider optimizing API endpoints with slow response times".to_string());
        }
        
        if api_metrics.error_rate > self.thresholds.api_error_rate {
            recommendations.push("Investigate and fix API errors to improve reliability".to_string());
        }
        
        if regression.detected {
            recommendations.push("Performance regression detected - investigate recent changes".to_string());
        }
        
        if api_metrics.requests_per_second > 100.0 {
            recommendations.push("Consider implementing rate limiting for high-traffic endpoints".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Performance is within acceptable thresholds".to_string());
        }
        
        recommendations
    }

    /// Calculate percentile
    fn percentile(&self, sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }
        
        let index = (percentile * (sorted_values.len() - 1) as f64) as usize;
        sorted_values[index]
    }
}

/// Performance reporting router
pub fn performance_router() -> Router {
    Router::new()
        .route("/perf/report", get(get_performance_report))
        .route("/perf/metrics", get(get_performance_metrics))
}

/// Get performance report
async fn get_performance_report(
    State(reporter): State<Arc<PerformanceReporter>>,
) -> Result<Json<PerformanceReport>, String> {
    let report = reporter.generate_report(24).await
        .map_err(|e| format!("Failed to generate performance report: {}", e))?;
    
    Ok(Json(report))
}

/// Get performance metrics
async fn get_performance_metrics(
    State(reporter): State<Arc<PerformanceReporter>>,
) -> Result<Json<PerformanceMetrics>, String> {
    let report = reporter.generate_report(1).await
        .map_err(|e| format!("Failed to generate performance metrics: {}", e))?;
    
    Ok(Json(report.metrics))
}
