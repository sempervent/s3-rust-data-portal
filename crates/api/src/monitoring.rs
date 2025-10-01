// Comprehensive Monitoring and Metrics
// Week 5: Performance optimization with monitoring

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkMetrics,
    pub database_metrics: DatabaseMetrics,
    pub cache_metrics: CacheMetrics,
    pub api_metrics: ApiMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_active: u32,
    pub connections_total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_connections: u32,
    pub query_count: u64,
    pub slow_queries: u64,
    pub avg_query_time_ms: f64,
    pub connection_pool_utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub eviction_count: u64,
    pub total_requests: u64,
    pub cache_size: usize,
    pub memory_usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub requests_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub resolved: bool,
    pub metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighCpuUsage,
    HighMemoryUsage,
    HighDiskUsage,
    DatabaseSlowQueries,
    CacheLowHitRate,
    ApiHighLatency,
    DatabaseConnectionPoolExhausted,
    CacheMemoryExhausted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct MonitoringService {
    metrics: Arc<RwLock<SystemMetrics>>,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    request_times: Arc<RwLock<Vec<Duration>>>,
    is_monitoring: Arc<RwLock<bool>>,
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(SystemMetrics {
                timestamp: chrono::Utc::now(),
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_io: NetworkMetrics {
                    bytes_sent: 0,
                    bytes_received: 0,
                    connections_active: 0,
                    connections_total: 0,
                },
                database_metrics: DatabaseMetrics {
                    active_connections: 0,
                    idle_connections: 0,
                    total_connections: 0,
                    query_count: 0,
                    slow_queries: 0,
                    avg_query_time_ms: 0.0,
                    connection_pool_utilization: 0.0,
                },
                cache_metrics: CacheMetrics {
                    hit_rate: 0.0,
                    miss_rate: 0.0,
                    eviction_count: 0,
                    total_requests: 0,
                    cache_size: 0,
                    memory_usage: 0.0,
                },
                api_metrics: ApiMetrics {
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                    avg_response_time_ms: 0.0,
                    p95_response_time_ms: 0.0,
                    p99_response_time_ms: 0.0,
                    requests_per_second: 0.0,
                },
            })),
            alerts: Arc::new(RwLock::new(Vec::new())),
            request_times: Arc::new(RwLock::new(Vec::new())),
            is_monitoring: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start_monitoring(&self) {
        let mut is_monitoring = self.is_monitoring.write().await;
        if *is_monitoring {
            return;
        }
        *is_monitoring = true;
        drop(is_monitoring);

        let metrics = self.metrics.clone();
        let alerts = self.alerts.clone();
        let request_times = self.request_times.clone();

        // Start metrics collection
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::collect_system_metrics(&metrics).await {
                    error!("Failed to collect system metrics: {}", e);
                }
                
                if let Err(e) = Self::check_performance_alerts(&metrics, &alerts).await {
                    error!("Failed to check performance alerts: {}", e);
                }
            }
        });

        // Start request time tracking cleanup
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Clean every 5 minutes
            
            loop {
                interval.tick().await;
                
                let mut times = request_times.write().await;
                if times.len() > 10000 { // Keep only last 10k requests
                    times.drain(0..times.len() - 10000);
                }
            }
        });

        info!("Monitoring service started");
    }

    async fn collect_system_metrics(metrics: &Arc<RwLock<SystemMetrics>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut metrics_guard = metrics.write().await;
        
        // Update timestamp
        metrics_guard.timestamp = chrono::Utc::now();
        
        // Collect CPU usage (simplified)
        metrics_guard.cpu_usage = Self::get_cpu_usage().await?;
        
        // Collect memory usage (simplified)
        metrics_guard.memory_usage = Self::get_memory_usage().await?;
        
        // Collect disk usage (simplified)
        metrics_guard.disk_usage = Self::get_disk_usage().await?;
        
        // Update network metrics
        metrics_guard.network_io = Self::get_network_metrics().await?;
        
        // Update database metrics
        metrics_guard.database_metrics = Self::get_database_metrics().await?;
        
        // Update cache metrics
        metrics_guard.cache_metrics = Self::get_cache_metrics().await?;
        
        // Update API metrics
        metrics_guard.api_metrics = Self::get_api_metrics(&metrics_guard).await?;
        
        Ok(())
    }

    async fn get_cpu_usage() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified CPU usage calculation
        // In production, you would use system monitoring libraries
        Ok(25.0) // Placeholder
    }

    async fn get_memory_usage() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified memory usage calculation
        // In production, you would use system monitoring libraries
        Ok(60.0) // Placeholder
    }

    async fn get_disk_usage() -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified disk usage calculation
        // In production, you would use system monitoring libraries
        Ok(45.0) // Placeholder
    }

    async fn get_network_metrics() -> Result<NetworkMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified network metrics
        // In production, you would use system monitoring libraries
        Ok(NetworkMetrics {
            bytes_sent: 1024 * 1024 * 100, // 100MB
            bytes_received: 1024 * 1024 * 200, // 200MB
            connections_active: 50,
            connections_total: 100,
        })
    }

    async fn get_database_metrics() -> Result<DatabaseMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified database metrics
        // In production, you would query the actual database
        Ok(DatabaseMetrics {
            active_connections: 10,
            idle_connections: 5,
            total_connections: 15,
            query_count: 1000,
            slow_queries: 5,
            avg_query_time_ms: 25.0,
            connection_pool_utilization: 0.67,
        })
    }

    async fn get_cache_metrics() -> Result<CacheMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Simplified cache metrics
        // In production, you would query the actual cache
        Ok(CacheMetrics {
            hit_rate: 0.85,
            miss_rate: 0.15,
            eviction_count: 10,
            total_requests: 1000,
            cache_size: 500,
            memory_usage: 0.3,
        })
    }

    async fn get_api_metrics(system_metrics: &SystemMetrics) -> Result<ApiMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Calculate API metrics based on request times
        let total_requests = system_metrics.api_metrics.total_requests + 1;
        let successful_requests = system_metrics.api_metrics.successful_requests + 1;
        let failed_requests = system_metrics.api_metrics.failed_requests;
        
        Ok(ApiMetrics {
            total_requests,
            successful_requests,
            failed_requests,
            avg_response_time_ms: 150.0, // Placeholder
            p95_response_time_ms: 300.0, // Placeholder
            p99_response_time_ms: 500.0, // Placeholder
            requests_per_second: 10.0, // Placeholder
        })
    }

    async fn check_performance_alerts(
        metrics: &Arc<RwLock<SystemMetrics>>,
        alerts: &Arc<RwLock<Vec<PerformanceAlert>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metrics_guard = metrics.read().await;
        let mut alerts_guard = alerts.write().await;
        
        // Check CPU usage
        if metrics_guard.cpu_usage > 80.0 {
            Self::create_alert(
                &mut alerts_guard,
                AlertType::HighCpuUsage,
                AlertSeverity::High,
                format!("High CPU usage: {:.1}%", metrics_guard.cpu_usage),
            ).await;
        }
        
        // Check memory usage
        if metrics_guard.memory_usage > 85.0 {
            Self::create_alert(
                &mut alerts_guard,
                AlertType::HighMemoryUsage,
                AlertSeverity::High,
                format!("High memory usage: {:.1}%", metrics_guard.memory_usage),
            ).await;
        }
        
        // Check disk usage
        if metrics_guard.disk_usage > 90.0 {
            Self::create_alert(
                &mut alerts_guard,
                AlertType::HighDiskUsage,
                AlertSeverity::Critical,
                format!("High disk usage: {:.1}%", metrics_guard.disk_usage),
            ).await;
        }
        
        // Check database slow queries
        if metrics_guard.database_metrics.slow_queries > 10 {
            Self::create_alert(
                &mut alerts_guard,
                AlertType::DatabaseSlowQueries,
                AlertSeverity::Medium,
                format!("High number of slow queries: {}", metrics_guard.database_metrics.slow_queries),
            ).await;
        }
        
        // Check cache hit rate
        if metrics_guard.cache_metrics.hit_rate < 0.7 {
            Self::create_alert(
                &mut alerts_guard,
                AlertType::CacheLowHitRate,
                AlertSeverity::Medium,
                format!("Low cache hit rate: {:.1}%", metrics_guard.cache_metrics.hit_rate * 100.0),
            ).await;
        }
        
        // Check API latency
        if metrics_guard.api_metrics.avg_response_time_ms > 1000.0 {
            Self::create_alert(
                &mut alerts_guard,
                AlertType::ApiHighLatency,
                AlertSeverity::High,
                format!("High API latency: {:.1}ms", metrics_guard.api_metrics.avg_response_time_ms),
            ).await;
        }
        
        Ok(())
    }

    async fn create_alert(
        alerts: &mut Vec<PerformanceAlert>,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: String,
    ) {
        let alert = PerformanceAlert {
            id: Uuid::new_v4(),
            alert_type,
            severity,
            message,
            timestamp: chrono::Utc::now(),
            resolved: false,
            metrics: None,
        };
        
        alerts.push(alert);
        warn!("Performance alert created: {}", message);
    }

    pub async fn record_request_time(&self, duration: Duration) {
        let mut request_times = self.request_times.write().await;
        request_times.push(duration);
    }

    pub async fn get_metrics(&self) -> SystemMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    pub async fn get_alerts(&self) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.read().await;
        alerts.clone()
    }

    pub async fn get_performance_summary(&self) -> serde_json::Value {
        let metrics = self.get_metrics().await;
        let alerts = self.get_alerts().await;
        
        serde_json::json!({
            "timestamp": metrics.timestamp,
            "system_health": {
                "cpu_usage": metrics.cpu_usage,
                "memory_usage": metrics.memory_usage,
                "disk_usage": metrics.disk_usage,
            },
            "database_health": {
                "active_connections": metrics.database_metrics.active_connections,
                "connection_pool_utilization": metrics.database_metrics.connection_pool_utilization,
                "avg_query_time_ms": metrics.database_metrics.avg_query_time_ms,
            },
            "cache_health": {
                "hit_rate": metrics.cache_metrics.hit_rate,
                "cache_size": metrics.cache_metrics.cache_size,
                "memory_usage": metrics.cache_metrics.memory_usage,
            },
            "api_health": {
                "total_requests": metrics.api_metrics.total_requests,
                "success_rate": metrics.api_metrics.successful_requests as f64 / metrics.api_metrics.total_requests as f64,
                "avg_response_time_ms": metrics.api_metrics.avg_response_time_ms,
            },
            "alerts": {
                "total": alerts.len(),
                "unresolved": alerts.iter().filter(|a| !a.resolved).count(),
                "critical": alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Critical)).count(),
            }
        })
    }
}
