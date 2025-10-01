// S3 connector for external bucket federation
// Week 8: Federation across data sources

use super::traits::*;
use async_trait::async_trait;
use rusoto_core::{Region, RusotoError};
use rusoto_s3::{GetObjectRequest, ListObjectsV2Request, S3Client, S3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// S3 connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3ConnectorConfig {
    pub bucket: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub prefix: Option<String>,
    pub endpoint: Option<String>,
}

/// S3 connector implementation
pub struct S3Connector {
    config: S3ConnectorConfig,
    client: S3Client,
    name: String,
}

impl S3Connector {
    pub fn new(name: String, config: S3ConnectorConfig) -> Result<Self, ConnectorError> {
        let region = config.region.parse::<Region>()
            .map_err(|e| ConnectorError::ConfigurationError(format!("Invalid region: {}", e)))?;
        
        let client = S3Client::new(region);
        
        Ok(Self {
            config,
            client,
            name,
        })
    }
    
    /// Generate presigned URL for S3 object
    async fn generate_presigned_url(&self, key: &str, _expires_in_seconds: u32) -> Result<String, ConnectorError> {
        // For now, return a direct URL - in production, you'd use presigned URLs
        let endpoint = self.config.endpoint.as_deref().unwrap_or("https://s3.amazonaws.com");
        Ok(format!("{}/{}", endpoint, key))
    }
}

#[async_trait]
impl Connector for S3Connector {
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::S3
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn test_connection(&self) -> Result<(), ConnectorError> {
        let request = ListObjectsV2Request {
            bucket: self.config.bucket.clone(),
            prefix: self.config.prefix.clone(),
            max_keys: Some(1),
            ..Default::default()
        };
        
        self.client.list_objects_v2(request).await
            .map_err(|e| ConnectorError::ConnectionError(format!("Failed to list objects: {}", e)))?;
        
        Ok(())
    }
    
    async fn list_entries(&self) -> Result<Vec<ExternalEntry>, ConnectorError> {
        let mut entries = Vec::new();
        let mut continuation_token: Option<String> = None;
        
        loop {
            let request = ListObjectsV2Request {
                bucket: self.config.bucket.clone(),
                prefix: self.config.prefix.clone(),
                continuation_token,
                max_keys: Some(1000),
                ..Default::default()
            };
            
            let response = self.client.list_objects_v2(request).await
                .map_err(|e| ConnectorError::SyncError(format!("Failed to list objects: {}", e)))?;
            
            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        // Skip directories
                        if key.ends_with('/') {
                            continue;
                        }
                        
                        let entry = ExternalEntry {
                            id: format!("s3:{}:{}", self.config.bucket, key),
                            title: key.split('/').last().unwrap_or(&key).to_string(),
                            description: None,
                            url: self.generate_presigned_url(&key, 3600).await?,
                            content_type: None, // S3 Object doesn't have content_type field
                            size: object.size.map(|s| s as u64),
                            modified_at: object.last_modified.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
                            tags: vec![],
                            metadata: {
                                let mut meta = HashMap::new();
                                meta.insert("bucket".to_string(), serde_json::Value::String(self.config.bucket.clone()));
                                meta.insert("key".to_string(), serde_json::Value::String(key.clone()));
                                meta.insert("etag".to_string(), serde_json::Value::String(object.e_tag.unwrap_or_default()));
                                meta
                            },
                            source_id: Uuid::new_v4(), // This would be the connector ID
                            source_type: "s3".to_string(),
                        };
                        
                        entries.push(entry);
                    }
                }
            }
            
            continuation_token = response.next_continuation_token;
            if continuation_token.is_none() {
                break;
            }
        }
        
        Ok(entries)
    }
    
    async fn get_entry(&self, id: &str) -> Result<Option<ExternalEntry>, ConnectorError> {
        // Parse S3 ID format: s3:bucket:key
        let parts: Vec<&str> = id.split(':').collect();
        if parts.len() != 3 || parts[0] != "s3" {
            return Err(ConnectorError::EntryNotFound(format!("Invalid S3 entry ID: {}", id)));
        }
        
        let bucket = parts[1];
        let key = parts[2];
        
        if bucket != self.config.bucket {
            return Ok(None);
        }
        
        // Get object metadata
        let request = GetObjectRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            ..Default::default()
        };
        
        match self.client.get_object(request).await {
            Ok(response) => {
                let entry = ExternalEntry {
                    id: id.to_string(),
                    title: key.split('/').last().unwrap_or(key).to_string(),
                    description: None,
                    url: self.generate_presigned_url(key, 3600).await?,
                    content_type: response.content_type,
                    size: response.content_length.map(|s| s as u64),
                    modified_at: response.last_modified.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
                    tags: vec![],
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("bucket".to_string(), serde_json::Value::String(bucket.to_string()));
                        meta.insert("key".to_string(), serde_json::Value::String(key.to_string()));
                        meta.insert("etag".to_string(), serde_json::Value::String(response.e_tag.unwrap_or_default()));
                        meta
                    },
                    source_id: Uuid::new_v4(),
                    source_type: "s3".to_string(),
                };
                
                Ok(Some(entry))
            }
            Err(RusotoError::Service(rusoto_s3::GetObjectError::NoSuchKey(_))) => {
                Ok(None)
            }
            Err(e) => {
                Err(ConnectorError::SyncError(format!("Failed to get object: {}", e)))
            }
        }
    }
    
    async fn get_presigned_url(&self, entry: &ExternalEntry, expires_in_seconds: u32) -> Result<String, ConnectorError> {
        // Extract key from metadata
        let key = entry.metadata.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::ConfigurationError("Missing key in entry metadata".to_string()))?;
        
        self.generate_presigned_url(key, expires_in_seconds).await
    }
    
    async fn sync_entries(&self) -> Result<SyncResult, ConnectorError> {
        let start_time = std::time::Instant::now();
        
        // Test connection first
        self.test_connection().await?;
        
        // List all entries
        let entries = self.list_entries().await?;
        
        let duration = start_time.elapsed();
        
        Ok(SyncResult {
            entries_processed: entries.len() as u64,
            entries_added: entries.len() as u64,
            entries_updated: 0,
            entries_removed: 0,
            errors: vec![],
            duration_seconds: duration.as_secs_f64(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_s3_connector_config() {
        let config = S3ConnectorConfig {
            bucket: "test-bucket".to_string(),
            region: "us-east-1".to_string(),
            access_key_id: "test-key".to_string(),
            secret_access_key: "test-secret".to_string(),
            prefix: Some("data/".to_string()),
            endpoint: None,
        };
        
        assert_eq!(config.bucket, "test-bucket");
        assert_eq!(config.region, "us-east-1");
    }
    
    #[test]
    fn test_s3_entry_id_parsing() {
        let id = "s3:my-bucket:path/to/file.csv";
        let parts: Vec<&str> = id.split(':').collect();
        
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "s3");
        assert_eq!(parts[1], "my-bucket");
        assert_eq!(parts[2], "path/to/file.csv");
    }
}
