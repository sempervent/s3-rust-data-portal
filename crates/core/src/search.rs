// BlackLake Search Integration
// Week 6: Apache Solr integration for advanced search capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Solr configuration
#[derive(Debug, Clone)]
pub struct SolrConfig {
    pub url: String,
    pub collection: String,
    pub commit_within: u64, // milliseconds
    pub batch_size: u32,
    pub timeout: std::time::Duration,
}

impl Default for SolrConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8983/solr".to_string(),
            collection: "blacklake".to_string(),
            commit_within: 1500, // 1.5 seconds
            batch_size: 100,
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Solr document for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrDocument {
    pub id: String, // Composite key: {repo}:{ref}:{path}:{commit_id}
    pub repo: String,
    pub r#ref: String,
    pub path: String,
    pub commit_id: String,
    pub file_name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub org_lab: String,
    pub file_type: String,
    pub file_size: i64,
    pub creation_dt: String, // ISO 8601 format
    pub sha256: String,
    pub content: Option<String>, // Extracted text content
    pub meta: serde_json::Value,
}

/// Solr search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrSearchRequest {
    pub q: String,
    pub fq: Vec<String>, // Filter queries
    pub sort: Option<String>,
    pub start: Option<u32>,
    pub rows: Option<u32>,
    pub facet: Option<SolrFacetRequest>,
    pub suggest: Option<SolrSuggestRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrFacetRequest {
    pub field: Vec<String>,
    pub range: Option<SolrRangeFacet>,
    pub limit: Option<u32>,
    pub mincount: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrRangeFacet {
    pub field: String,
    pub start: String,
    pub end: String,
    pub gap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrSuggestRequest {
    pub q: String,
    pub count: Option<u32>,
}

/// Solr search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrSearchResponse {
    pub response: SolrResponse,
    pub facets: Option<SolrFacets>,
    pub suggest: Option<SolrSuggestResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrResponse {
    pub num_found: u32,
    pub start: u32,
    pub docs: Vec<SolrDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrFacets {
    pub facet_fields: Option<HashMap<String, Vec<serde_json::Value>>>,
    pub facet_ranges: Option<HashMap<String, SolrRangeFacetResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrRangeFacetResult {
    pub counts: Vec<serde_json::Value>,
    pub gap: String,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrSuggestResponse {
    pub suggest: HashMap<String, SolrSuggestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrSuggestResult {
    pub num_found: u32,
    pub suggestions: Vec<SolrSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrSuggestion {
    pub term: String,
    pub weight: u32,
    pub payload: Option<String>,
}

/// Solr client for BlackLake
pub struct SolrClient {
    config: SolrConfig,
    client: reqwest::Client,
}

/// Solr errors
#[derive(Debug, Error)]
pub enum SolrError {
    #[error("Solr request failed: {0}")]
    Request(String),
    #[error("Solr response error: {0}")]
    Response(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Timeout error: {0}")]
    Timeout(String),
}

impl SolrClient {
    /// Create a new Solr client
    pub fn new(config: SolrConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }
    
    /// Index a document
    pub async fn index_document(&self, doc: &SolrDocument) -> Result<(), SolrError> {
        let url = format!("{}/{}/update", self.config.url, self.config.collection);
        
        let payload = serde_json::json!({
            "add": {
                "doc": doc,
                "commitWithin": self.config.commit_within
            }
        });
        
        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| SolrError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SolrError::Response(error_text));
        }
        
        Ok(())
    }
    
    /// Index multiple documents in batch
    pub async fn index_documents(&self, docs: &[SolrDocument]) -> Result<(), SolrError> {
        if docs.is_empty() {
            return Ok(());
        }
        
        let url = format!("{}/{}/update", self.config.url, self.config.collection);
        
        let mut payload = serde_json::Map::new();
        let docs_json: Vec<serde_json::Value> = docs.iter()
            .map(|doc| serde_json::to_value(doc).unwrap())
            .collect();
        
        payload.insert("add".to_string(), serde_json::json!({
            "docs": docs_json,
            "commitWithin": self.config.commit_within
        }));
        
        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| SolrError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SolrError::Response(error_text));
        }
        
        Ok(())
    }
    
    /// Delete documents by query
    pub async fn delete_by_query(&self, query: &str) -> Result<(), SolrError> {
        let url = format!("{}/{}/update", self.config.url, self.config.collection);
        
        let payload = serde_json::json!({
            "delete": {
                "query": query
            },
            "commitWithin": self.config.commit_within
        });
        
        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| SolrError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SolrError::Response(error_text));
        }
        
        Ok(())
    }
    
    /// Search documents
    pub async fn search(&self, request: &SolrSearchRequest) -> Result<SolrSearchResponse, SolrError> {
        let url = format!("{}/{}/select", self.config.url, self.config.collection);
        
        let mut params = vec![
            ("q", request.q.clone()),
            ("wt", "json".to_string()),
        ];
        
        if let Some(sort) = &request.sort {
            params.push(("sort", sort.clone()));
        }
        
        if let Some(start) = request.start {
            params.push(("start", start.to_string()));
        }
        
        if let Some(rows) = request.rows {
            params.push(("rows", rows.to_string()));
        }
        
        // Add filter queries
        for fq in &request.fq {
            params.push(("fq", fq.clone()));
        }
        
        // Add facets
        if let Some(facet) = &request.facet {
            params.push(("facet", "true".to_string()));
            for field in &facet.field {
                params.push(("facet.field", field.clone()));
            }
            if let Some(limit) = facet.limit {
                params.push(("facet.limit", limit.to_string()));
            }
            if let Some(mincount) = facet.mincount {
                params.push(("facet.mincount", mincount.to_string()));
            }
            
            // Add range facets
            if let Some(range) = &facet.range {
                params.push(("facet.range", range.field.clone()));
                params.push(("f.range.facet.range.start", range.start.clone()));
                params.push(("f.range.facet.range.end", range.end.clone()));
                params.push(("f.range.facet.range.gap", range.gap.clone()));
            }
        }
        
        // Add suggest
        if let Some(suggest) = &request.suggest {
            params.push(("suggest", "true".to_string()));
            params.push(("suggest.q", suggest.q.clone()));
            if let Some(count) = suggest.count {
                params.push(("suggest.count", count.to_string()));
            }
        }
        
        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| SolrError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SolrError::Response(error_text));
        }
        
        let search_response: SolrSearchResponse = response
            .json()
            .await
            .map_err(|e| SolrError::Serialization(e.to_string()))?;
        
        Ok(search_response)
    }
    
    /// Get suggestions
    pub async fn suggest(&self, query: &str, count: Option<u32>) -> Result<Vec<SolrSuggestion>, SolrError> {
        let url = format!("{}/{}/suggest", self.config.url, self.config.collection);
        
        let mut params = vec![
            ("suggest", "true".to_string()),
            ("suggest.q", query.to_string()),
            ("suggest.dictionary", "file_name_suggest".to_string()),
        ];
        
        if let Some(count) = count {
            params.push(("suggest.count", count.to_string()));
        }
        
        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| SolrError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SolrError::Response(error_text));
        }
        
        let suggest_response: SolrSuggestResponse = response
            .json()
            .await
            .map_err(|e| SolrError::Serialization(e.to_string()))?;
        
        // Extract suggestions from response
        let suggestions = suggest_response
            .suggest
            .values()
            .flat_map(|result| result.suggestions.clone())
            .collect();
        
        Ok(suggestions)
    }
    
    /// Commit changes
    pub async fn commit(&self) -> Result<(), SolrError> {
        let url = format!("{}/{}/update", self.config.url, self.config.collection);
        
        let payload = serde_json::json!({
            "commit": {}
        });
        
        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| SolrError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SolrError::Response(error_text));
        }
        
        Ok(())
    }
    
    /// Get collection status
    pub async fn get_status(&self) -> Result<SolrStatus, SolrError> {
        let url = format!("{}/admin/collections", self.config.url);
        
        let params = vec![
            ("action", "CLUSTERSTATUS"),
            ("collection", self.config.collection.clone()),
        ];
        
        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| SolrError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SolrError::Response(error_text));
        }
        
        let status_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SolrError::Serialization(e.to_string()))?;
        
        // Parse status response
        let doc_count = status_response
            .get("cluster")
            .and_then(|c| c.get("collections"))
            .and_then(|collections| collections.get(&self.config.collection))
            .and_then(|collection| collection.get("docs"))
            .and_then(|docs| docs.as_u64())
            .unwrap_or(0);
        
        Ok(SolrStatus {
            collection: self.config.collection.clone(),
            doc_count,
            status: "active".to_string(),
        })
    }
}

/// Solr collection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolrStatus {
    pub collection: String,
    pub doc_count: u64,
    pub status: String,
}

/// Convert BlackLake entry to Solr document
pub fn entry_to_solr_document(
    repo_name: &str,
    ref_name: &str,
    path: &str,
    commit_id: Uuid,
    meta: &serde_json::Value,
    sha256: &str,
) -> SolrDocument {
    let id = format!("{}:{}:{}:{}", repo_name, ref_name, path, commit_id);
    
    SolrDocument {
        id,
        repo: repo_name.to_string(),
        r#ref: ref_name.to_string(),
        path: path.to_string(),
        commit_id: commit_id.to_string(),
        file_name: meta.get("file_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        title: meta.get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: meta.get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        tags: meta.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default(),
        org_lab: meta.get("org_lab")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        file_type: meta.get("file_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        file_size: meta.get("file_size")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        creation_dt: meta.get("creation_dt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        sha256: sha256.to_string(),
        content: None, // Will be populated by content extraction jobs
        meta: meta.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_solr_config_default() {
        let config = SolrConfig::default();
        assert_eq!(config.url, "http://localhost:8983/solr");
        assert_eq!(config.collection, "blacklake");
        assert_eq!(config.commit_within, 1500);
        assert_eq!(config.batch_size, 100);
    }
    
    #[test]
    fn test_entry_to_solr_document() {
        let meta = serde_json::json!({
            "file_name": "test.csv",
            "description": "Test dataset",
            "tags": ["test", "csv"],
            "org_lab": "ORNL",
            "file_type": "text/csv",
            "file_size": 1234,
            "creation_dt": "2025-01-17T18:28:00Z"
        });
        
        let commit_id = Uuid::new_v4();
        let doc = entry_to_solr_document(
            "test-repo",
            "main",
            "data/test.csv",
            commit_id,
            &meta,
            "abc123",
        );
        
        assert_eq!(doc.repo, "test-repo");
        assert_eq!(doc.r#ref, "main");
        assert_eq!(doc.path, "data/test.csv");
        assert_eq!(doc.file_name, "test.csv");
        assert_eq!(doc.description, Some("Test dataset".to_string()));
        assert_eq!(doc.tags, vec!["test", "csv"]);
        assert_eq!(doc.org_lab, "ORNL");
        assert_eq!(doc.file_type, "text/csv");
        assert_eq!(doc.file_size, 1234);
        assert_eq!(doc.sha256, "abc123");
    }
    
    #[test]
    fn test_solr_document_serialization() {
        let doc = SolrDocument {
            id: "test:main:data/test.csv:123".to_string(),
            repo: "test".to_string(),
            r#ref: "main".to_string(),
            path: "data/test.csv".to_string(),
            commit_id: "123".to_string(),
            file_name: "test.csv".to_string(),
            title: None,
            description: Some("Test dataset".to_string()),
            tags: vec!["test".to_string()],
            org_lab: "ORNL".to_string(),
            file_type: "text/csv".to_string(),
            file_size: 1234,
            creation_dt: "2025-01-17T18:28:00Z".to_string(),
            sha256: "abc123".to_string(),
            content: None,
            meta: serde_json::json!({}),
        };
        
        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: SolrDocument = serde_json::from_str(&json).unwrap();
        
        assert_eq!(doc.id, deserialized.id);
        assert_eq!(doc.repo, deserialized.repo);
        assert_eq!(doc.file_name, deserialized.file_name);
        assert_eq!(doc.tags, deserialized.tags);
    }
}