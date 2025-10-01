// Redis Caching System
// Week 5: Performance optimization with comprehensive caching

use redis::{Client as RedisClient, Connection, Commands};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub default_ttl: Duration,
    pub search_ttl: Duration,
    pub metadata_ttl: Duration,
    pub max_cache_size: usize,
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600), // 1 hour
            search_ttl: Duration::from_secs(1800),  // 30 minutes
            metadata_ttl: Duration::from_secs(7200),  // 2 hours
            max_cache_size: 1000,
            enable_compression: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub created_at: u64,
    pub ttl: u64,
    pub hit_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCacheKey {
    pub query: String,
    pub filters: Vec<String>,
    pub sort: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

impl SearchCacheKey {
    pub fn to_string(&self) -> String {
        format!(
            "search:{}:{}:{}:{}:{}",
            self.query,
            self.filters.join(","),
            self.sort.as_deref().unwrap_or(""),
            self.limit,
            self.offset
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataCacheKey {
    pub repo_id: Uuid,
    pub path: String,
    pub commit_id: Option<Uuid>,
}

impl MetadataCacheKey {
    pub fn to_string(&self) -> String {
        format!(
            "metadata:{}:{}:{}",
            self.repo_id,
            self.path,
            self.commit_id.map(|id| id.to_string()).unwrap_or_default()
        )
    }
}

pub struct CacheManager {
    redis_client: RedisClient,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_requests: u64,
    pub cache_size: usize,
}

impl CacheManager {
    pub fn new(redis_url: &str, config: CacheConfig) -> Result<Self, redis::RedisError> {
        let redis_client = RedisClient::open(redis_url)?;
        Ok(Self {
            redis_client,
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }

    /// Get a cached search result
    pub async fn get_search_result<T>(&self, key: &SearchCacheKey) -> Result<Option<T>, redis::RedisError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.redis_client.get_async_connection().await?;
        let cache_key = format!("search:{}", key.to_string());
        
        let result: Option<String> = conn.get(&cache_key).await?;
        
        if let Some(cached_data) = result {
            let entry: CacheEntry<T> = serde_json::from_str(&cached_data)
                .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string())))?;
            
            // Check if entry is still valid
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if now - entry.created_at < entry.ttl {
                // Update hit count
                let mut stats = self.stats.write().await;
                stats.hits += 1;
                stats.total_requests += 1;
                
                info!("Cache hit for search key: {}", cache_key);
                return Ok(Some(entry.data));
            } else {
                // Entry expired, remove it
                let _: () = conn.del(&cache_key).await?;
                let mut stats = self.stats.write().await;
                stats.evictions += 1;
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.misses += 1;
        stats.total_requests += 1;
        
        Ok(None)
    }

    /// Cache a search result
    pub async fn set_search_result<T>(&self, key: &SearchCacheKey, data: T) -> Result<(), redis::RedisError>
    where
        T: Serialize,
    {
        let mut conn = self.redis_client.get_async_connection().await?;
        let cache_key = format!("search:{}", key.to_string());
        
        let entry = CacheEntry {
            data,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            ttl: self.config.search_ttl.as_secs(),
            hit_count: 0,
        };
        
        let serialized = serde_json::to_string(&entry)
            .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;
        
        let _: () = conn.set_ex(&cache_key, &serialized, self.config.search_ttl.as_secs() as usize).await?;
        
        info!("Cached search result for key: {}", cache_key);
        Ok(())
    }

    /// Get cached metadata
    pub async fn get_metadata<T>(&self, key: &MetadataCacheKey) -> Result<Option<T>, redis::RedisError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.redis_client.get_async_connection().await?;
        let cache_key = format!("metadata:{}", key.to_string());
        
        let result: Option<String> = conn.get(&cache_key).await?;
        
        if let Some(cached_data) = result {
            let entry: CacheEntry<T> = serde_json::from_str(&cached_data)
                .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "Deserialization failed", e.to_string())))?;
            
            // Check if entry is still valid
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if now - entry.created_at < entry.ttl {
                let mut stats = self.stats.write().await;
                stats.hits += 1;
                stats.total_requests += 1;
                
                info!("Cache hit for metadata key: {}", cache_key);
                return Ok(Some(entry.data));
            } else {
                let _: () = conn.del(&cache_key).await?;
                let mut stats = self.stats.write().await;
                stats.evictions += 1;
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.misses += 1;
        stats.total_requests += 1;
        
        Ok(None)
    }

    /// Cache metadata
    pub async fn set_metadata<T>(&self, key: &MetadataCacheKey, data: T) -> Result<(), redis::RedisError>
    where
        T: Serialize,
    {
        let mut conn = self.redis_client.get_async_connection().await?;
        let cache_key = format!("metadata:{}", key.to_string());
        
        let entry = CacheEntry {
            data,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            ttl: self.config.metadata_ttl.as_secs(),
            hit_count: 0,
        };
        
        let serialized = serde_json::to_string(&entry)
            .map_err(|e| redis::RedisError::from((redis::ErrorKind::TypeError, "Serialization failed", e.to_string())))?;
        
        let _: () = conn.set_ex(&cache_key, &serialized, self.config.metadata_ttl.as_secs() as usize).await?;
        
        info!("Cached metadata for key: {}", cache_key);
        Ok(())
    }

    /// Invalidate cache entries
    pub async fn invalidate_search_cache(&self, pattern: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let search_pattern = format!("search:*{}*", pattern);
        
        let keys: Vec<String> = conn.keys(&search_pattern).await?;
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
            info!("Invalidated {} search cache entries", keys.len());
        }
        
        Ok(())
    }

    /// Invalidate metadata cache
    pub async fn invalidate_metadata_cache(&self, repo_id: Uuid) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let metadata_pattern = format!("metadata:{}:*", repo_id);
        
        let keys: Vec<String> = conn.keys(&metadata_pattern).await?;
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
            info!("Invalidated {} metadata cache entries for repo {}", keys.len(), repo_id);
        }
        
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Clear all cache
    pub async fn clear_all(&self) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let _: () = conn.flushdb().await?;
        
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
        
        info!("Cleared all cache entries");
        Ok(())
    }

    /// Get cache hit rate
    pub async fn get_hit_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        if stats.total_requests == 0 {
            0.0
        } else {
            stats.hits as f64 / stats.total_requests as f64
        }
    }

    /// Start cache cleanup task
    pub async fn start_cleanup_task(&self) {
        let redis_client = self.redis_client.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Run every hour
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::cleanup_expired_entries(&redis_client).await {
                    error!("Cache cleanup failed: {}", e);
                }
            }
        });
    }

    async fn cleanup_expired_entries(redis_client: &RedisClient) -> Result<(), redis::RedisError> {
        let mut conn = redis_client.get_async_connection().await?;
        
        // Get all cache keys
        let search_keys: Vec<String> = conn.keys("search:*").await?;
        let metadata_keys: Vec<String> = conn.keys("metadata:*").await?;
        
        let mut expired_keys = Vec::new();
        
        // Check search keys
        for key in search_keys {
            let entry_str: Option<String> = conn.get(&key).await?;
            if let Some(entry_str) = entry_str {
                if let Ok(entry) = serde_json::from_str::<CacheEntry<serde_json::Value>>(&entry_str) {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    if now - entry.created_at >= entry.ttl {
                        expired_keys.push(key);
                    }
                }
            }
        }
        
        // Check metadata keys
        for key in metadata_keys {
            let entry_str: Option<String> = conn.get(&key).await?;
            if let Some(entry_str) = entry_str {
                if let Ok(entry) = serde_json::from_str::<CacheEntry<serde_json::Value>>(&entry_str) {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    if now - entry.created_at >= entry.ttl {
                        expired_keys.push(key);
                    }
                }
            }
        }
        
        // Remove expired keys
        if !expired_keys.is_empty() {
            let _: () = conn.del(expired_keys).await?;
            info!("Cleaned up expired cache entries");
        }
        
        Ok(())
    }
}
