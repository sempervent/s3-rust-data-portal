use sqlx::{PgPool, Postgres, Row};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, warn, info};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure_time: Option<std::time::Instant>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_time: None,
            config,
        }
    }

    pub async fn call<F, T>(&mut self, operation: F) -> Result<T, sqlx::Error>
    where
        F: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        match self.state {
            CircuitState::Closed => {
                match operation.await {
                    Ok(result) => {
                        self.on_success();
                        Ok(result)
                    }
                    Err(e) => {
                        self.on_failure();
                        Err(e)
                    }
                }
            }
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitState::HalfOpen;
                    self.call(operation).await
                } else {
                    error!("Circuit breaker is open, operation rejected");
                    Err(sqlx::Error::Configuration("Circuit breaker is open".into()))
                }
            }
            CircuitState::HalfOpen => {
                match operation.await {
                    Ok(result) => {
                        self.on_success();
                        Ok(result)
                    }
                    Err(e) => {
                        self.on_failure();
                        Err(e)
                    }
                }
            }
        }
    }

    fn on_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.last_failure_time = None;
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitState::Open;
            warn!("Circuit breaker opened after {} failures", self.failure_count);
        }
    }

    fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = self.last_failure_time {
            last_failure.elapsed() >= self.config.recovery_timeout
        } else {
            false
        }
    }
}

pub struct ResilientIndexClient {
    pool: PgPool,
    circuit_breaker: std::sync::Mutex<CircuitBreaker>,
    query_timeout: Duration,
}

impl ResilientIndexClient {
    pub fn new(pool: PgPool, query_timeout: Option<Duration>) -> Self {
        Self {
            pool,
            circuit_breaker: std::sync::Mutex::new(CircuitBreaker::new(CircuitBreakerConfig::default())),
            query_timeout: query_timeout.unwrap_or(Duration::from_secs(30)),
        }
    }

    pub async fn execute_with_timeout_and_circuit_breaker<F, T>(
        &self,
        operation: F,
    ) -> Result<T, sqlx::Error>
    where
        F: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        let timeout_future = timeout(self.query_timeout, operation);
        
        match timeout_future.await {
            Ok(result) => {
                let mut cb = self.circuit_breaker.lock().unwrap();
                cb.call(async { result }).await
            }
            Err(_) => {
                error!("Database operation timed out after {:?}", self.query_timeout);
                Err(sqlx::Error::Configuration("Operation timed out".into()))
            }
        }
    }

    pub async fn test_connection(&self) -> Result<(), sqlx::Error> {
        self.execute_with_timeout_and_circuit_breaker(async {
            sqlx::query("SELECT 1")
                .fetch_one(&self.pool)
                .await
                .map(|_| ())
        }).await
    }

    pub async fn get_connection_health(&self) -> Result<ConnectionHealth, sqlx::Error> {
        self.execute_with_timeout_and_circuit_breaker(async {
            let start = std::time::Instant::now();
            
            // Test basic connectivity
            sqlx::query("SELECT 1")
                .fetch_one(&self.pool)
                .await?;
            
            let response_time = start.elapsed();
            
            // Get connection pool stats
            let pool_size = self.pool.size();
            let idle_connections = self.pool.num_idle();
            let active_connections = pool_size - idle_connections;
            
            Ok(ConnectionHealth {
                is_healthy: true,
                response_time,
                pool_size,
                active_connections,
                idle_connections,
            })
        }).await
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    pub is_healthy: bool,
    pub response_time: Duration,
    pub pool_size: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.recovery_timeout, Duration::from_secs(60));
        assert_eq!(config.half_open_max_calls, 3);
    }

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        assert!(matches!(cb.state, CircuitState::Closed));
        assert_eq!(cb.failure_count, 0);
        assert!(cb.last_failure_time.is_none());
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        
        let result = cb.call(async { Ok::<i32, sqlx::Error>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert!(matches!(cb.state, CircuitState::Closed));
        assert_eq!(cb.failure_count, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_secs(1),
            half_open_max_calls: 1,
        });
        
        // First failure
        let result = cb.call(async { Err::<i32, sqlx::Error>(sqlx::Error::Configuration("test error".into())) }).await;
        assert!(result.is_err());
        assert!(matches!(cb.state, CircuitState::Closed));
        assert_eq!(cb.failure_count, 1);
        
        // Second failure - should open circuit
        let result = cb.call(async { Err::<i32, sqlx::Error>(sqlx::Error::Configuration("test error".into())) }).await;
        assert!(result.is_err());
        assert!(matches!(cb.state, CircuitState::Open));
        assert_eq!(cb.failure_count, 2);
    }
}
