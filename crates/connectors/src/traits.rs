// Connector traits for federation
// Week 8: Federation across data sources

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// External data source entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEntry {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub content_type: Option<String>,
    pub size: Option<u64>,
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub source_id: Uuid,
    pub source_type: String,
}

/// Connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub name: String,
    pub description: Option<String>,
    pub connector_type: ConnectorType,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub sync_interval_minutes: u32,
}

/// Supported connector types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectorType {
    S3,
    Postgres,
    Ckan,
}

/// Connector status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorStatus {
    pub id: Uuid,
    pub name: String,
    pub connector_type: ConnectorType,
    pub enabled: bool,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    pub entries_count: u64,
    pub sync_in_progress: bool,
}

/// Connector sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub entries_processed: u64,
    pub entries_added: u64,
    pub entries_updated: u64,
    pub entries_removed: u64,
    pub errors: Vec<String>,
    pub duration_seconds: f64,
}

/// Connector trait for external data sources
#[async_trait]
pub trait Connector: Send + Sync {
    /// Get connector type
    fn connector_type(&self) -> ConnectorType;
    
    /// Get connector name
    fn name(&self) -> &str;
    
    /// Test connector connectivity
    async fn test_connection(&self) -> Result<(), ConnectorError>;
    
    /// List all entries from the external source
    async fn list_entries(&self) -> Result<Vec<ExternalEntry>, ConnectorError>;
    
    /// Get a specific entry by ID
    async fn get_entry(&self, id: &str) -> Result<Option<ExternalEntry>, ConnectorError>;
    
    /// Generate a presigned URL for accessing an entry
    async fn get_presigned_url(&self, entry: &ExternalEntry, expires_in_seconds: u32) -> Result<String, ConnectorError>;
    
    /// Sync entries from the external source
    async fn sync_entries(&self) -> Result<SyncResult, ConnectorError>;
}

/// Connector errors
#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Permission denied: {0}")]
    PermissionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Sync error: {0}")]
    SyncError(String),
    
    #[error("Entry not found: {0}")]
    EntryNotFound(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),
    
    #[error("AWS error: {0}")]
    AwsError(#[from] rusoto_core::RusotoError<rusoto_s3::GetObjectError>),
}

/// Connector factory trait
#[async_trait]
pub trait ConnectorFactory: Send + Sync {
    /// Create a connector from configuration
    async fn create_connector(&self, config: ConnectorConfig) -> Result<Box<dyn Connector>, ConnectorError>;
    
    /// Get supported connector types
    fn supported_types(&self) -> Vec<ConnectorType>;
}

/// Connector registry for managing multiple connectors
#[async_trait]
pub trait ConnectorRegistry: Send + Sync {
    /// Register a new connector
    async fn register_connector(&self, config: ConnectorConfig) -> Result<Uuid, ConnectorError>;
    
    /// Unregister a connector
    async fn unregister_connector(&self, id: Uuid) -> Result<(), ConnectorError>;
    
    /// Get connector by ID
    async fn get_connector(&self, id: Uuid) -> Result<Option<Box<dyn Connector>>, ConnectorError>;
    
    /// List all registered connectors
    async fn list_connectors(&self) -> Result<Vec<ConnectorStatus>, ConnectorError>;
    
    /// Test connector connectivity
    async fn test_connector(&self, id: Uuid) -> Result<(), ConnectorError>;
    
    /// Sync all enabled connectors
    async fn sync_all_connectors(&self) -> Result<Vec<SyncResult>, ConnectorError>;
    
    /// Sync a specific connector
    async fn sync_connector(&self, id: Uuid) -> Result<SyncResult, ConnectorError>;
}
