use anyhow::{anyhow, Result};
use blacklake_core::{CanonicalMeta, Change, ChangeOp, CommitRequest, CommitResponse, SearchRequest, SearchResponse, TreeResponse, UploadInitResponse};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadInitRequest {
    pub path: String,
    pub size: u64,
    pub media_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            token: None,
        }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn request(&self) -> reqwest::RequestBuilder {
        let mut req = self.client.get(&self.base_url);
        
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        
        req
    }

    pub fn post_request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self.client.post(url);
        
        if let Some(ref token) = self.token {
            req = req.bearer_auth(token);
        }
        
        req
    }

    pub async fn upload_init(&self, repo: &str, request: &UploadInitRequest) -> Result<UploadInitResponse> {
        let url = format!("{}/v1/repos/{}/upload-init", self.base_url, repo);
        let response = self.post_request(&url)
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Upload init failed: {}", error_text));
        }

        let upload_response: UploadInitResponse = response.json().await?;
        Ok(upload_response)
    }

    pub async fn upload_file(&self, upload_url: &str, file_path: &Path) -> Result<()> {
        let file_size = std::fs::metadata(file_path)?.len();
        let file_content = std::fs::read(file_path)?;

        let pb = ProgressBar::new(file_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        let response = self.client
            .put(upload_url)
            .body(file_content)
            .send()
            .await?;

        pb.finish_with_message("Upload complete");

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Upload failed: {}", error_text));
        }

        Ok(())
    }

    pub async fn commit(&self, repo: &str, request: &CommitRequest, merge: bool) -> Result<CommitResponse> {
        let url = format!("{}/v1/repos/{}/commit", self.base_url, repo);
        
        let mut req_builder = self.post_request(&url);
        
        if merge {
            req_builder = req_builder.header("X-Blacklake-Merge", "true");
        }

        let response = req_builder
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Commit failed: {}", error_text));
        }

        let commit_response: CommitResponse = response.json().await?;
        Ok(commit_response)
    }

    pub async fn get_tree(&self, repo: &str, r#ref: &str, path: Option<&str>) -> Result<TreeResponse> {
        let mut url = format!("{}/v1/repos/{}/tree/{}", self.base_url, repo, r#ref);
        
        if let Some(path) = path {
            url.push_str(&format!("?path={}", urlencoding::encode(path)));
        }

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Get tree failed: {}", error_text));
        }

        let tree_response: TreeResponse = response.json().await?;
        Ok(tree_response)
    }

    pub async fn search(&self, repo: &str, request: &SearchRequest) -> Result<SearchResponse> {
        let mut url = format!("{}/v1/repos/{}/search", self.base_url, repo);
        
        let mut query_params = Vec::new();
        // Add filters to query params
        for (key, value) in &request.filters {
            if let Some(value_str) = value.as_str() {
                query_params.push(format!("{}={}", key, urlencoding::encode(value_str)));
            }
        }
        if let Some(limit) = request.limit {
            query_params.push(format!("limit={}", limit));
        }
        if let Some(offset) = request.offset {
            query_params.push(format!("offset={}", offset));
        }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Search failed: {}", error_text));
        }

        let search_response: SearchResponse = response.json().await?;
        Ok(search_response)
    }

    pub async fn get_blob(&self, repo: &str, r#ref: &str, path: &str) -> Result<String> {
        let url = format!("{}/v1/repos/{}/blob/{}/{}", 
            self.base_url, repo, r#ref, urlencoding::encode(path));
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Get blob failed: {}", error_text));
        }

        let blob_response: Value = response.json().await?;
        let download_url = blob_response["download_url"]
            .as_str()
            .ok_or_else(|| anyhow!("No download URL in response"))?;

        Ok(download_url.to_string())
    }

    pub async fn get_rdf(&self, repo: &str, r#ref: &str, path: &str, format: &str) -> Result<String> {
        let url = format!("{}/v1/repos/{}/rdf/{}/{}?format={}", 
            self.base_url, repo, r#ref, urlencoding::encode(path), format);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Get RDF failed: {}", error_text));
        }

        let rdf_content = response.text().await?;
        Ok(rdf_content)
    }

    pub async fn get_schema(&self, collection: Option<&str>) -> Result<Value> {
        let url = if let Some(collection) = collection {
            format!("{}/v1/schemas/{}", self.base_url, collection)
        } else {
            format!("{}/v1/schemas/default", self.base_url)
        };

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Get schema failed: {}", error_text));
        }

        let schema: Value = response.json().await?;
        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = ApiClient::new("http://localhost:8080".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
        assert!(client.token.is_none());
    }

    #[test]
    fn test_api_client_with_token() {
        let client = ApiClient::new("http://localhost:8080".to_string())
            .with_token("test-token".to_string());
        assert_eq!(client.token, Some("test-token".to_string()));
    }
}
