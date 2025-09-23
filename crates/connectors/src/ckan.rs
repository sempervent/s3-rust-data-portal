// CKAN connector for open data federation
// Week 8: Federation across data sources

use super::traits::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// CKAN connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CkanConnectorConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub organization: Option<String>,
    pub tags: Vec<String>,
    pub limit: Option<u32>,
}

/// CKAN package response
#[derive(Debug, Deserialize)]
struct CkanPackage {
    id: String,
    name: String,
    title: String,
    notes: Option<String>,
    resources: Vec<CkanResource>,
    tags: Vec<CkanTag>,
    organization: Option<CkanOrganization>,
    metadata_created: Option<String>,
    metadata_modified: Option<String>,
}

/// CKAN resource
#[derive(Debug, Deserialize)]
struct CkanResource {
    id: String,
    name: String,
    description: Option<String>,
    url: String,
    format: Option<String>,
    size: Option<u64>,
    created: Option<String>,
    last_modified: Option<String>,
}

/// CKAN tag
#[derive(Debug, Deserialize)]
struct CkanTag {
    name: String,
    display_name: String,
}

/// CKAN organization
#[derive(Debug, Deserialize)]
struct CkanOrganization {
    id: String,
    name: String,
    title: String,
}

/// CKAN API response
#[derive(Debug, Deserialize)]
struct CkanResponse<T> {
    success: bool,
    result: T,
}

/// CKAN connector implementation
pub struct CkanConnector {
    config: CkanConnectorConfig,
    client: Client,
    name: String,
}

impl CkanConnector {
    pub fn new(name: String, config: CkanConnectorConfig) -> Self {
        let client = Client::new();
        
        Self {
            config,
            client,
            name,
        }
    }
    
    /// Build API URL
    fn build_api_url(&self, endpoint: &str) -> String {
        format!("{}/api/3/action/{}", self.config.base_url.trim_end_matches('/'), endpoint)
    }
    
    /// Make authenticated request
    async fn make_request<T>(&self, endpoint: &str, params: Option<HashMap<String, String>>) -> Result<T, ConnectorError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.build_api_url(endpoint);
        
        let mut request = self.client.get(&url);
        
        // Add API key if provided
        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", api_key);
        }
        
        // Add query parameters
        if let Some(params) = params {
            request = request.query(&params);
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(ConnectorError::HttpError(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ));
        }
        
        let ckan_response: CkanResponse<T> = response.json().await?;
        
        if !ckan_response.success {
            return Err(ConnectorError::SyncError("CKAN API returned success=false".to_string()));
        }
        
        Ok(ckan_response.result)
    }
    
    /// Convert CKAN package to ExternalEntry
    fn package_to_entry(&self, package: &CkanPackage, source_id: Uuid) -> Vec<ExternalEntry> {
        let mut entries = Vec::new();
        
        for resource in &package.resources {
            let mut metadata = HashMap::new();
            metadata.insert("package_id".to_string(), serde_json::Value::String(package.id.clone()));
            metadata.insert("package_name".to_string(), serde_json::Value::String(package.name.clone()));
            metadata.insert("resource_id".to_string(), serde_json::Value::String(resource.id.clone()));
            
            if let Some(ref org) = package.organization {
                metadata.insert("organization".to_string(), serde_json::Value::String(org.name.clone()));
            }
            
            let tags: Vec<String> = package.tags.iter().map(|t| t.name.clone()).collect();
            
            let entry = ExternalEntry {
                id: format!("ckan:{}:{}", package.id, resource.id),
                title: resource.name.clone(),
                description: resource.description.clone().or_else(|| package.notes.clone()),
                url: resource.url.clone(),
                content_type: resource.format.clone(),
                size: resource.size,
                modified_at: resource.last_modified
                    .as_ref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                tags,
                metadata,
                source_id,
                source_type: "ckan".to_string(),
            };
            
            entries.push(entry);
        }
        
        entries
    }
}

#[async_trait]
impl Connector for CkanConnector {
    fn connector_type(&self) -> ConnectorType {
        ConnectorType::Ckan
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    async fn test_connection(&self) -> Result<(), ConnectorError> {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), "1".to_string());
        
        if let Some(ref org) = self.config.organization {
            params.insert("fq".to_string(), format!("organization:{}", org));
        }
        
        self.make_request::<Vec<CkanPackage>>("package_list", Some(params)).await?;
        Ok(())
    }
    
    async fn list_entries(&self) -> Result<Vec<ExternalEntry>, ConnectorError> {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), self.config.limit.unwrap_or(1000).to_string());
        
        // Add organization filter
        if let Some(ref org) = self.config.organization {
            params.insert("fq".to_string(), format!("organization:{}", org));
        }
        
        // Add tag filters
        if !self.config.tags.is_empty() {
            let tag_filter = self.config.tags.join(" OR ");
            params.insert("fq".to_string(), format!("tags:({})", tag_filter));
        }
        
        let package_names: Vec<String> = self.make_request("package_list", Some(params)).await?;
        
        let mut all_entries = Vec::new();
        let source_id = Uuid::new_v4(); // This would be the connector ID
        
        // Fetch details for each package
        for package_name in package_names {
            let package_name_clone = package_name.clone();
            let mut params = HashMap::new();
            params.insert("id".to_string(), package_name);
            
            match self.make_request::<CkanPackage>("package_show", Some(params)).await {
                Ok(package) => {
                    let entries = self.package_to_entry(&package, source_id);
                    all_entries.extend(entries);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch package {}: {}", package_name_clone, e);
                }
            }
        }
        
        Ok(all_entries)
    }
    
    async fn get_entry(&self, id: &str) -> Result<Option<ExternalEntry>, ConnectorError> {
        // Parse CKAN ID format: ckan:package_id:resource_id
        let parts: Vec<&str> = id.split(':').collect();
        if parts.len() != 3 || parts[0] != "ckan" {
            return Err(ConnectorError::EntryNotFound(format!("Invalid CKAN entry ID: {}", id)));
        }
        
        let package_id = parts[1];
        let resource_id = parts[2];
        
        let mut params = HashMap::new();
        params.insert("id".to_string(), package_id.to_string());
        
        let package: CkanPackage = self.make_request("package_show", Some(params)).await?;
        
        // Find the specific resource
        for resource in &package.resources {
            if resource.id == resource_id {
                let source_id = Uuid::new_v4();
                let entries = self.package_to_entry(&package, source_id);
                
                return Ok(entries.into_iter().find(|e| e.id == id));
            }
        }
        
        Ok(None)
    }
    
    async fn get_presigned_url(&self, entry: &ExternalEntry, _expires_in_seconds: u32) -> Result<String, ConnectorError> {
        // For CKAN, return the URL from the entry
        Ok(entry.url.clone())
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
    fn test_ckan_connector_config() {
        let config = CkanConnectorConfig {
            base_url: "https://data.gov".to_string(),
            api_key: Some("test-key".to_string()),
            organization: Some("test-org".to_string()),
            tags: vec!["data".to_string(), "open".to_string()],
            limit: Some(100),
        };
        
        assert_eq!(config.base_url, "https://data.gov");
        assert_eq!(config.organization, Some("test-org".to_string()));
        assert_eq!(config.tags.len(), 2);
    }
    
    #[test]
    fn test_ckan_entry_id_parsing() {
        let id = "ckan:package-123:resource-456";
        let parts: Vec<&str> = id.split(':').collect();
        
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "ckan");
        assert_eq!(parts[1], "package-123");
        assert_eq!(parts[2], "resource-456");
    }
    
    #[test]
    fn test_api_url_building() {
        let config = CkanConnectorConfig {
            base_url: "https://data.gov".to_string(),
            api_key: None,
            organization: None,
            tags: vec![],
            limit: None,
        };
        
        let connector = CkanConnector::new("test".to_string(), config);
        let url = connector.build_api_url("package_list");
        
        assert_eq!(url, "https://data.gov/api/3/action/package_list");
    }
}
