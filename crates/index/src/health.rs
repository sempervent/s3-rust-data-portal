// Database Health Monitoring
// Week 4: Production-ready database operations with health checks

use sqlx::{PgPool, Row};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{info, warn, error};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub is_healthy: bool,
    pub connection_count: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub last_check: Instant,
    pub response_time_ms: u64,
    pub error_count: u64,
    pub circuit_breaker_state: CircuitBreakerState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,    // Normal operation
    Open,      // Circuit is open, requests are failing
    HalfOpen,  // Testing if service is back
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub test_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            test_timeout: Duration::from_secs(5),
        }
    }
}

pub struct DatabaseHealthMonitor {
    pool: PgPool,
    health: Arc<RwLock<DatabaseHealth>>,
    config: CircuitBreakerConfig,
    consecutive_failures: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl DatabaseHealthMonitor {
    pub fn new(pool: PgPool, config: CircuitBreakerConfig) -> Self {
        Self {
            pool,
            health: Arc::new(RwLock::new(DatabaseHealth {
                is_healthy: true,
                connection_count: 0,
                active_connections: 0,
                idle_connections: 0,
                last_check: Instant::now(),
                response_time_ms: 0,
                error_count: 0,
                circuit_breaker_state: CircuitBreakerState::Closed,
            })),
            config,
            consecutive_failures: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start_monitoring(&self) {
        let health = self.health.clone();
        let pool = self.pool.clone();
        let config = self.config.clone();
        let consecutive_failures = self.consecutive_failures.clone();
        let last_failure_time = self.last_failure_time.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                let start = Instant::now();
                let mut is_healthy = true;
                let mut error_count = 0;

                // Check database connectivity
                match Self::check_database_health(&pool).await {
                    Ok(connection_stats) => {
                        let response_time = start.elapsed().as_millis() as u64;
                        
                        let mut health_guard = health.write().await;
                        health_guard.is_healthy = true;
                        health_guard.connection_count = connection_stats.connection_count;
                        health_guard.active_connections = connection_stats.active_connections;
                        health_guard.idle_connections = connection_stats.idle_connections;
                        health_guard.last_check = Instant::now();
                        health_guard.response_time_ms = response_time;
                        health_guard.error_count = error_count;
                        health_guard.circuit_breaker_state = CircuitBreakerState::Closed;
                        
                        // Reset failure count on successful check
                        let mut failures_guard = consecutive_failures.write().await;
                        *failures_guard = 0;
                        
                        info!(
                            "Database health check passed: {} connections, {}ms response time",
                            connection_stats.connection_count,
                            response_time
                        );
                    }
                    Err(e) => {
                        is_healthy = false;
                        error_count += 1;
                        
                        let mut health_guard = health.write().await;
                        health_guard.is_healthy = false;
                        health_guard.error_count = error_count;
                        health_guard.last_check = Instant::now();
                        
                        // Update failure count and circuit breaker state
                        let mut failures_guard = consecutive_failures.write().await;
                        *failures_guard += 1;
                        
                        let mut last_failure_guard = last_failure_time.write().await;
                        *last_failure_guard = Some(Instant::now());
                        
                        // Check if circuit breaker should open
                        if *failures_guard >= config.failure_threshold {
                            health_guard.circuit_breaker_state = CircuitBreakerState::Open;
                            warn!(
                                "Circuit breaker opened after {} consecutive failures",
                                *failures_guard
                            );
                        }
                        
                        error!("Database health check failed: {}", e);
                    }
                }
            }
        });
    }

    async fn check_database_health(pool: &PgPool) -> Result<ConnectionStats, sqlx::Error> {
        // Get connection pool statistics
        let connection_count = pool.size();
        let idle_connections = pool.num_idle();
        let active_connections = connection_count - idle_connections;

        // Test database connectivity with a simple query
        let _: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(pool)
            .await?;

        Ok(ConnectionStats {
            connection_count: connection_count as u32,
            active_connections: active_connections as u32,
            idle_connections: idle_connections as u32,
        })
    }

    pub async fn get_health(&self) -> DatabaseHealth {
        self.health.read().await.clone()
    }

    pub async fn is_circuit_breaker_open(&self) -> bool {
        let health = self.health.read().await;
        health.circuit_breaker_state == CircuitBreakerState::Open
    }

    pub async fn can_execute_query(&self) -> bool {
        let health = self.health.read().await;
        let consecutive_failures = self.consecutive_failures.read().await;
        let last_failure_time = self.last_failure_time.read().await;

        match health.circuit_breaker_state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if we should try to recover
                if let Some(last_failure) = *last_failure_time {
                    if last_failure.elapsed() >= self.config.recovery_timeout {
                        // Transition to half-open state
                        let mut health_guard = self.health.write().await;
                        health_guard.circuit_breaker_state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Allow one test request
                if *consecutive_failures == 0 {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub async fn record_success(&self) {
        let mut health = self.health.write().await;
        health.circuit_breaker_state = CircuitBreakerState::Closed;
        
        let mut failures = self.consecutive_failures.write().await;
        *failures = 0;
    }

    pub async fn record_failure(&self) {
        let mut failures = self.consecutive_failures.write().await;
        *failures += 1;
        
        let mut last_failure = self.last_failure_time.write().await;
        *last_failure = Some(Instant::now());
        
        if *failures >= self.config.failure_threshold {
            let mut health = self.health.write().await;
            health.circuit_breaker_state = CircuitBreakerState::Open;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub connection_count: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
}

impl DatabaseHealthMonitor {
    pub async fn get_connection_stats(&self) -> Result<ConnectionStats, sqlx::Error> {
        Self::check_database_health(&self.pool).await
    }
}
