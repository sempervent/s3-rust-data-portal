// AI-assisted embeddings and semantic search
// Week 8: AI-assisted metadata and search

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Embedding vector (384 dimensions for MiniLM)
pub type Embedding = Vec<f32>;

/// Embedding generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub text: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Embedding generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Embedding,
    pub model: String,
    pub dimensions: usize,
    pub processing_time_ms: u64,
}

/// Semantic search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub threshold: Option<f32>,
    pub filters: Option<HashMap<String, serde_json::Value>>,
}

/// Semantic search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub similarity_score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub source_type: String,
}

/// Semantic search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResponse {
    pub results: Vec<SemanticSearchResult>,
    pub total: usize,
    pub query_embedding: Option<Embedding>,
    pub processing_time_ms: u64,
}

/// Suggested tags from NER
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedTags {
    pub entity_tags: Vec<String>,
    pub keyword_tags: Vec<String>,
    pub confidence_scores: HashMap<String, f32>,
    pub processing_time_ms: u64,
}

/// Embedding model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModelConfig {
    pub model_name: String,
    pub dimensions: usize,
    pub max_text_length: usize,
    pub batch_size: usize,
}

/// Embedding service errors
#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("Model loading failed: {0}")]
    ModelLoadingError(String),
    
    #[error("Text processing failed: {0}")]
    TextProcessingError(String),
    
    #[error("Embedding generation failed: {0}")]
    EmbeddingGenerationError(String),
    
    #[error("Search failed: {0}")]
    SearchError(String),
    
    #[error("NER processing failed: {0}")]
    NerError(String),
    
    #[error("Text too long: {0} characters (max: {1})")]
    TextTooLong(usize, usize),
    
    #[error("Invalid embedding dimensions: expected {0}, got {1}")]
    InvalidDimensions(usize, usize),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Embedding service trait
#[async_trait::async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate embedding for text
    async fn generate_embedding(&self, request: &EmbeddingRequest) -> Result<EmbeddingResponse, EmbeddingError>;
    
    /// Generate embeddings for multiple texts (batch processing)
    async fn generate_embeddings_batch(&self, requests: &[EmbeddingRequest]) -> Result<Vec<EmbeddingResponse>, EmbeddingError>;
    
    /// Perform semantic search
    async fn semantic_search(&self, request: &SemanticSearchRequest) -> Result<SemanticSearchResponse, EmbeddingError>;
    
    /// Extract suggested tags using NER
    async fn extract_suggested_tags(&self, text: &str) -> Result<SuggestedTags, EmbeddingError>;
    
    /// Get model configuration
    fn get_model_config(&self) -> &EmbeddingModelConfig;
}

/// Mock embedding service for development
pub struct MockEmbeddingService {
    config: EmbeddingModelConfig,
}

impl MockEmbeddingService {
    pub fn new() -> Self {
        Self {
            config: EmbeddingModelConfig {
                model_name: "all-MiniLM-L6-v2".to_string(),
                dimensions: 384,
                max_text_length: 512,
                batch_size: 32,
            },
        }
    }
    
    /// Generate a mock embedding vector
    fn generate_mock_embedding(&self, text: &str) -> Embedding {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Generate deterministic but varied embeddings based on text hash
        (0..self.config.dimensions)
            .map(|i| {
                let seed = hash.wrapping_add(i as u64);
                let mut rng = (seed * 1103515245 + 12345) as u32;
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                (rng as f32) / (u32::MAX as f32) * 2.0 - 1.0
            })
            .collect()
    }
    
    /// Calculate cosine similarity between two embeddings
    fn cosine_similarity(&self, a: &Embedding, b: &Embedding) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }
}

#[async_trait::async_trait]
impl EmbeddingService for MockEmbeddingService {
    async fn generate_embedding(&self, request: &EmbeddingRequest) -> Result<EmbeddingResponse, EmbeddingError> {
        if request.text.len() > self.config.max_text_length {
            return Err(EmbeddingError::TextTooLong(request.text.len(), self.config.max_text_length));
        }
        
        let start_time = std::time::Instant::now();
        let embedding = self.generate_mock_embedding(&request.text);
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        Ok(EmbeddingResponse {
            embedding,
            model: self.config.model_name.clone(),
            dimensions: self.config.dimensions,
            processing_time_ms: processing_time,
        })
    }
    
    async fn generate_embeddings_batch(&self, requests: &[EmbeddingRequest]) -> Result<Vec<EmbeddingResponse>, EmbeddingError> {
        let mut responses = Vec::new();
        
        for request in requests {
            let response = self.generate_embedding(request).await?;
            responses.push(response);
        }
        
        Ok(responses)
    }
    
    async fn semantic_search(&self, request: &SemanticSearchRequest) -> Result<SemanticSearchResponse, EmbeddingError> {
        let start_time = std::time::Instant::now();
        
        // Generate query embedding
        let query_request = EmbeddingRequest {
            text: request.query.clone(),
            metadata: None,
        };
        let query_response = self.generate_embedding(&query_request).await?;
        let query_embedding = query_response.embedding;
        
        // Mock search results - in production, this would query the database
        let mock_results = vec![
            SemanticSearchResult {
                id: "mock-1".to_string(),
                title: "Sample Document 1".to_string(),
                description: Some("This is a sample document for testing semantic search".to_string()),
                url: "/documents/sample-1".to_string(),
                similarity_score: 0.85,
                metadata: HashMap::new(),
                source_type: "internal".to_string(),
            },
            SemanticSearchResult {
                id: "mock-2".to_string(),
                title: "Sample Document 2".to_string(),
                description: Some("Another sample document with related content".to_string()),
                url: "/documents/sample-2".to_string(),
                similarity_score: 0.72,
                metadata: HashMap::new(),
                source_type: "internal".to_string(),
            },
        ];
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        Ok(SemanticSearchResponse {
            results: mock_results,
            total: 2,
            query_embedding: Some(query_embedding),
            processing_time_ms: processing_time,
        })
    }
    
    async fn extract_suggested_tags(&self, text: &str) -> Result<SuggestedTags, EmbeddingError> {
        let start_time = std::time::Instant::now();
        
        // Mock NER processing - in production, this would use a real NER model
        let entity_tags = vec![
            "PERSON".to_string(),
            "ORGANIZATION".to_string(),
            "LOCATION".to_string(),
        ];
        
        let keyword_tags = vec![
            "data".to_string(),
            "analysis".to_string(),
            "machine learning".to_string(),
        ];
        
        let mut confidence_scores = HashMap::new();
        confidence_scores.insert("PERSON".to_string(), 0.95);
        confidence_scores.insert("ORGANIZATION".to_string(), 0.87);
        confidence_scores.insert("LOCATION".to_string(), 0.73);
        confidence_scores.insert("data".to_string(), 0.91);
        confidence_scores.insert("analysis".to_string(), 0.84);
        confidence_scores.insert("machine learning".to_string(), 0.78);
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        Ok(SuggestedTags {
            entity_tags,
            keyword_tags,
            confidence_scores,
            processing_time_ms: processing_time,
        })
    }
    
    fn get_model_config(&self) -> &EmbeddingModelConfig {
        &self.config
    }
}

impl Default for MockEmbeddingService {
    fn default() -> Self {
        Self::new()
    }
}

/// Embedding job for background processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingJob {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub path: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub content_type: Option<String>,
    pub source_type: String, // "internal" or "external"
}

/// Suggested tags job for background processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedTagsJob {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub path: String,
    pub description: String,
    pub existing_tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_embedding_service() {
        let service = MockEmbeddingService::new();
        let config = service.get_model_config();
        
        assert_eq!(config.model_name, "all-MiniLM-L6-v2");
        assert_eq!(config.dimensions, 384);
    }
    
    #[tokio::test]
    async fn test_embedding_generation() {
        let service = MockEmbeddingService::new();
        let request = EmbeddingRequest {
            text: "This is a test document".to_string(),
            metadata: None,
        };
        
        let response = service.generate_embedding(&request).await.unwrap();
        assert_eq!(response.embedding.len(), 384);
        assert_eq!(response.model, "all-MiniLM-L6-v2");
    }
    
    #[tokio::test]
    async fn test_semantic_search() {
        let service = MockEmbeddingService::new();
        let request = SemanticSearchRequest {
            query: "machine learning data analysis".to_string(),
            limit: Some(10),
            threshold: Some(0.5),
            filters: None,
        };
        
        let response = service.semantic_search(&request).await.unwrap();
        assert!(!response.results.is_empty());
        assert!(response.processing_time_ms > 0);
    }
    
    #[tokio::test]
    async fn test_suggested_tags() {
        let service = MockEmbeddingService::new();
        let text = "John Smith works at Acme Corporation in New York. He specializes in data analysis and machine learning.";
        
        let tags = service.extract_suggested_tags(text).await.unwrap();
        assert!(!tags.entity_tags.is_empty());
        assert!(!tags.keyword_tags.is_empty());
        assert!(!tags.confidence_scores.is_empty());
    }
    
    #[test]
    fn test_cosine_similarity() {
        let service = MockEmbeddingService::new();
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];
        
        assert_eq!(service.cosine_similarity(&a, &b), 1.0);
        assert_eq!(service.cosine_similarity(&a, &c), 0.0);
    }
}
