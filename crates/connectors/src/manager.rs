// Connector manager for federation
// Week 8: Federation across data sources

use super::traits::*;
use super::{S3Connector, PostgresConnector, CkanConnector};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Connector manager implementation
pub struct ConnectorManager {
    connectors: Arc<RwLock<HashMap<Uuid, Box<dyn Connector>>>>,
    configs: Arc<RwLock<HashMap<Uuid, ConnectorConfig>>>,
    statuses: Arc<RwLock<HashMap<Uuid, ConnectorStatus>>>,
}

impl ConnectorManager {
    pub fn new() -> Self {
        Self {
            connectors: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create connector from configuration
    async fn create_connector(&self, id: Uuid, config: ConnectorConfig) -> Result<Box<dyn Connector>, ConnectorError> {
        match config.connector_type {
            ConnectorType::S3 => {
                let s3_config: super::s3::S3ConnectorConfig = serde_json::from_value(config.config)
                    .map_err(|e| ConnectorError::ConfigurationError(format!("Invalid S3 config: {}", e)))?;
                
                let connector = S3Connector::new(config.name.clone(), s3_config)?;
                Ok(Box::new(connector))
            }
            ConnectorType::Postgres => {
                let pg_config: super::postgres::PostgresConnectorConfig = serde_json::from_value(config.config)
                    .map_err(|e| ConnectorError::ConfigurationError(format!("Invalid Postgres config: {}", e)))?;
                
                let connector = PostgresConnector::new(config.name.clone(), pg_config).await?;
                Ok(Box::new(connector))
            }
            ConnectorType::Ckan => {
                let ckan_config: super::ckan::CkanConnectorConfig = serde_json::from_value(config.config)
                    .map_err(|e| ConnectorError::ConfigurationError(format!("Invalid CKAN config: {}", e)))?;
                
                let connector = CkanConnector::new(config.name.clone(), ckan_config);
                Ok(Box::new(connector))
            }
        }
    }
    
    /// Update connector status
    async fn update_status(&self, id: Uuid, status: ConnectorStatus) {
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, status);
    }
    
    /// Get connector status
    async fn get_status(&self, id: Uuid) -> Option<ConnectorStatus> {
        let statuses = self.statuses.read().await;
        statuses.get(&id).cloned()
    }
}

impl Default for ConnectorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConnectorRegistry for ConnectorManager {
    async fn register_connector(&self, config: ConnectorConfig) -> Result<Uuid, ConnectorError> {
        let id = Uuid::new_v4();
        
        // Create connector
        let connector = self.create_connector(id, config.clone()).await?;
        
        // Test connection
        connector.test_connection().await?;
        
        // Store connector and config
        {
            let mut connectors = self.connectors.write().await;
            connectors.insert(id, connector);
        }
        
        {
            let mut configs = self.configs.write().await;
            configs.insert(id, config.clone());
        }
        
        // Initialize status
        let status = ConnectorStatus {
            id,
            name: config.name,
            connector_type: config.connector_type,
            enabled: config.enabled,
            last_sync: None,
            last_error: None,
            entries_count: 0,
            sync_in_progress: false,
        };
        
        self.update_status(id, status).await;
        
        Ok(id)
    }
    
    async fn unregister_connector(&self, id: Uuid) -> Result<(), ConnectorError> {
        {
            let mut connectors = self.connectors.write().await;
            connectors.remove(&id);
        }
        
        {
            let mut configs = self.configs.write().await;
            configs.remove(&id);
        }
        
        {
            let mut statuses = self.statuses.write().await;
            statuses.remove(&id);
        }
        
        Ok(())
    }
    
    async fn get_connector(&self, id: Uuid) -> Result<Option<Box<dyn Connector>>, ConnectorError> {
        let connectors = self.connectors.read().await;
        Ok(connectors.get(&id).map(|c| {
            // This is a simplified approach - in production you'd need to clone the connector
            // or use a different approach to return owned connectors
            todo!("Implement connector cloning or use Arc<dyn Connector>")
        }))
    }
    
    async fn list_connectors(&self) -> Result<Vec<ConnectorStatus>, ConnectorError> {
        let statuses = self.statuses.read().await;
        Ok(statuses.values().cloned().collect())
    }
    
    async fn test_connector(&self, id: Uuid) -> Result<(), ConnectorError> {
        let connectors = self.connectors.read().await;
        let connector = connectors.get(&id)
            .ok_or_else(|| ConnectorError::EntryNotFound(format!("Connector {} not found", id)))?;
        
        connector.test_connection().await
    }
    
    async fn sync_all_connectors(&self) -> Result<Vec<SyncResult>, ConnectorError> {
        let statuses = self.statuses.read().await;
        let enabled_connectors: Vec<Uuid> = statuses
            .iter()
            .filter(|(_, status)| status.enabled)
            .map(|(id, _)| *id)
            .collect();
        
        let mut results = Vec::new();
        
        for id in enabled_connectors {
            match self.sync_connector(id).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::error!("Failed to sync connector {}: {}", id, e);
                    // Update status with error
                    if let Some(mut status) = self.get_status(id).await {
                        status.last_error = Some(e.to_string());
                        self.update_status(id, status).await;
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    async fn sync_connector(&self, id: Uuid) -> Result<SyncResult, ConnectorError> {
        // Update status to indicate sync in progress
        if let Some(mut status) = self.get_status(id).await {
            status.sync_in_progress = true;
            status.last_error = None;
            self.update_status(id, status).await;
        }
        
        let connectors = self.connectors.read().await;
        let connector = connectors.get(&id)
            .ok_or_else(|| ConnectorError::EntryNotFound(format!("Connector {} not found", id)))?;
        
        let result = connector.sync_entries().await;
        
        // Update status with result
        if let Some(mut status) = self.get_status(id).await {
            status.sync_in_progress = false;
            status.last_sync = Some(chrono::Utc::now());
            
            match &result {
                Ok(sync_result) => {
                    status.entries_count = sync_result.entries_processed;
                    status.last_error = None;
                }
                Err(e) => {
                    status.last_error = Some(e.to_string());
                }
            }
            
            self.update_status(id, status).await;
        }
        
        result
    }
}

/// Connector factory implementation
pub struct ConnectorFactory;

#[async_trait]
impl super::traits::ConnectorFactory for ConnectorFactory {
    async fn create_connector(&self, config: ConnectorConfig) -> Result<Box<dyn Connector>, ConnectorError> {
        match config.connector_type {
            ConnectorType::S3 => {
                let s3_config: super::s3::S3ConnectorConfig = serde_json::from_value(config.config)
                    .map_err(|e| ConnectorError::ConfigurationError(format!("Invalid S3 config: {}", e)))?;
                
                let connector = S3Connector::new(config.name.clone(), s3_config)?;
                Ok(Box::new(connector))
            }
            ConnectorType::Postgres => {
                let pg_config: super::postgres::PostgresConnectorConfig = serde_json::from_value(config.config)
                    .map_err(|e| ConnectorError::ConfigurationError(format!("Invalid Postgres config: {}", e)))?;
                
                let connector = PostgresConnector::new(config.name.clone(), pg_config).await?;
                Ok(Box::new(connector))
            }
            ConnectorType::Ckan => {
                let ckan_config: super::ckan::CkanConnectorConfig = serde_json::from_value(config.config)
                    .map_err(|e| ConnectorError::ConfigurationError(format!("Invalid CKAN config: {}", e)))?;
                
                let connector = CkanConnector::new(config.name.clone(), ckan_config);
                Ok(Box::new(connector))
            }
        }
    }
    
    fn supported_types(&self) -> Vec<ConnectorType> {
        vec![
            ConnectorType::S3,
            ConnectorType::Postgres,
            ConnectorType::Ckan,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connector_manager_creation() {
        let manager = ConnectorManager::new();
        let connectors = manager.list_connectors().await.unwrap();
        assert_eq!(connectors.len(), 0);
    }
    
    #[test]
    fn test_connector_factory_supported_types() {
        let factory = ConnectorFactory;
        let types = factory.supported_types();
        assert_eq!(types.len(), 3);
        assert!(types.contains(&ConnectorType::S3));
        assert!(types.contains(&ConnectorType::Postgres));
        assert!(types.contains(&ConnectorType::Ckan));
    }
}
