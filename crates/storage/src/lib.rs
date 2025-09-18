use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{
    config::{Builder as ConfigBuilder, Credentials, Region},
    presigning::{PresigningConfig, PresigningRequest},
    Client as S3Client, Endpoint, Operation,
};
use aws_smithy_http::endpoint::Endpoint as SmithyEndpoint;
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("S3 operation failed: {0}")]
    S3Error(#[from] aws_sdk_s3::Error),
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// S3 client wrapper with presigned URL generation
pub struct StorageClient {
    client: S3Client,
    bucket: String,
}

impl StorageClient {
    /// Create a new S3 client from environment variables
    pub async fn from_env() -> Result<Self> {
        let bucket = std::env::var("S3_BUCKET")
            .map_err(|_| StorageError::ConfigError("S3_BUCKET not set".to_string()))?;

        let region = std::env::var("S3_REGION")
            .map_err(|_| StorageError::ConfigError("S3_REGION not set".to_string()))?;

        let access_key = std::env::var("S3_ACCESS_KEY")
            .map_err(|_| StorageError::ConfigError("S3_ACCESS_KEY not set".to_string()))?;

        let secret_key = std::env::var("S3_SECRET_KEY")
            .map_err(|_| StorageError::ConfigError("S3_SECRET_KEY not set".to_string()))?;

        let endpoint = std::env::var("S3_ENDPOINT")
            .map_err(|_| StorageError::ConfigError("S3_ENDPOINT not set".to_string()))?;

        let force_path_style = std::env::var("S3_FORCE_PATH_STYLE")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let credentials = Credentials::new(&access_key, &secret_key, None, None, "env");

        let mut config_builder = ConfigBuilder::default()
            .region(Region::new(region))
            .credentials_provider(credentials)
            .force_path_style(force_path_style);

        // Set custom endpoint if provided
        if !endpoint.is_empty() {
            let endpoint_url = Url::parse(&endpoint)?;
            let smithy_endpoint = SmithyEndpoint::immutable(endpoint_url);
            config_builder = config_builder.endpoint_resolver(smithy_endpoint);
        }

        let config = config_builder.build();
        let client = S3Client::from_conf(config);

        // Ensure bucket exists (dev only)
        Self::ensure_bucket_exists(&client, &bucket).await?;

        Ok(Self { client, bucket })
    }

    /// Generate a presigned PUT URL for uploading content
    pub async fn presign_put(
        &self,
        key: &str,
        size: u64,
        content_type: &str,
        expires: Duration,
    ) -> Result<Url> {
        let presigning_config = PresigningConfig::expires_in(expires)
            .map_err(|e| StorageError::ConfigError(format!("Invalid presigning config: {}", e)))?;

        let request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_length(size as i64)
            .content_type(content_type)
            .presigned(presigning_config)
            .await?;

        Ok(Url::parse(&request.uri().to_string())?)
    }

    /// Generate a presigned GET URL for downloading content
    pub async fn presign_get(&self, key: &str, expires: Duration) -> Result<Url> {
        let presigning_config = PresigningConfig::expires_in(expires)
            .map_err(|e| StorageError::ConfigError(format!("Invalid presigning config: {}", e)))?;

        let request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await?;

        Ok(Url::parse(&request.uri().to_string())?)
    }

    /// Create content-addressed S3 key from SHA256 hash
    pub fn content_address_key(sha256: &str) -> String {
        format!("sha256/{}/{}/{}", &sha256[0..2], &sha256[2..4], sha256)
    }

    /// Ensure bucket exists (for development)
    async fn ensure_bucket_exists(client: &S3Client, bucket: &str) -> Result<()> {
        // TODO: Add retry logic with exponential backoff
        // TODO: Implement bucket lifecycle policies for object cleanup
        // TODO: Add bucket versioning and encryption configuration
        // TODO: Implement cross-region replication for disaster recovery
        
        // Try to create bucket, ignore if it already exists
        let _ = client
            .create_bucket()
            .bucket(bucket)
            .send()
            .await;

        Ok(())
    }

    /// Get the bucket name
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_address_key() {
        let sha256 = "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3";
        let key = StorageClient::content_address_key(sha256);
        assert_eq!(key, "sha256/a6/65/a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3");
    }
}
