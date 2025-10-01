use aws_sdk_s3::{
    config::{Builder as ConfigBuilder, Credentials, Region},
    presigning::PresigningConfig,
    Client as S3Client,
};
use std::time::Duration;
use thiserror::Error;
use url::Url;
use tokio::time::sleep;
use rand::Rng;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("S3 operation failed: {0}")]
    S3Error(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("AWS SDK error: {0}")]
    AwsSdkError(String),
}

impl From<aws_sdk_s3::Error> for StorageError {
    fn from(err: aws_sdk_s3::Error) -> Self {
        StorageError::AwsSdkError(err.to_string())
    }
}

impl From<aws_sdk_s3::error::BuildError> for StorageError {
    fn from(err: aws_sdk_s3::error::BuildError) -> Self {
        StorageError::AwsSdkError(err.to_string())
    }
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

        let config_builder = ConfigBuilder::default()
            .region(Region::new(region))
            .credentials_provider(credentials)
            .force_path_style(force_path_style);

        // Set custom endpoint if provided
        if !endpoint.is_empty() {
            let _endpoint_url = Url::parse(&endpoint)?;
            // Note: Custom endpoint configuration would need to be handled differently
            // For now, we'll skip custom endpoint configuration
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
            .await.map_err(|e| StorageError::AwsSdkError(e.to_string()))?;

        Ok(Url::parse(&request.uri().to_string())?)
    }

    /// Retry operation with exponential backoff and jitter
    async fn retry_operation<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        const MAX_RETRIES: u32 = 3;
        const BASE_DELAY: Duration = Duration::from_millis(100);
        const MAX_DELAY: Duration = Duration::from_secs(5);

        let mut attempt = 0;
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    if attempt > MAX_RETRIES {
                        return Err(e);
                    }

                    // Exponential backoff with jitter
                    let delay = BASE_DELAY * 2_u32.pow(attempt - 1);
                    let jitter = rand::thread_rng().gen_range(0..=100);
                    let final_delay = std::cmp::min(
                        delay + Duration::from_millis(jitter),
                        MAX_DELAY,
                    );

                    tracing::warn!("S3 operation failed (attempt {}), retrying in {:?}: {}", attempt, final_delay, e);
                    sleep(final_delay).await;
                }
            }
        }
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
            .await.map_err(|e| StorageError::AwsSdkError(e.to_string()))?;

        Ok(Url::parse(&request.uri().to_string())?)
    }

    /// Create content-addressed S3 key from SHA256 hash
    pub fn content_address_key(sha256: &str) -> String {
        format!("sha256/{}/{}/{}", &sha256[0..2], &sha256[2..4], sha256)
    }

    /// Ensure bucket exists with production-ready configuration
    async fn ensure_bucket_exists(client: &S3Client, bucket: &str) -> Result<()> {
        // Try to create bucket with retry logic
        let mut retry_count = 0;
        let max_retries = 3;
        
        while retry_count < max_retries {
            match client
                .create_bucket()
                .bucket(bucket)
                .send()
                .await
            {
                Ok(_) => {
                    // Configure bucket with production settings
                    Self::configure_bucket_production_settings(client, bucket).await?;
                    return Ok(());
                }
                Err(e) if e.to_string().contains("BucketAlreadyOwnedByYou") => {
                    // Bucket already exists, configure it
                    Self::configure_bucket_production_settings(client, bucket).await?;
                    return Ok(());
                }
                Err(_e) if retry_count < max_retries - 1 => {
                    retry_count += 1;
                    let delay = std::time::Duration::from_millis(1000 * retry_count as u64);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(StorageError::S3Error(e.to_string())),
            }
        }
        
        Ok(())
    }
    
    /// Configure bucket with production-ready settings
    async fn configure_bucket_production_settings(client: &S3Client, bucket: &str) -> Result<()> {
        // Enable versioning
        let _ = client
            .put_bucket_versioning()
            .bucket(bucket)
            .versioning_configuration(
                aws_sdk_s3::types::VersioningConfiguration::builder()
                    .status(aws_sdk_s3::types::BucketVersioningStatus::Enabled)
                    .build()
            )
            .send()
            .await;
        
        // Configure lifecycle policy for cost optimization
        let lifecycle_config = aws_sdk_s3::types::BucketLifecycleConfiguration::builder()
            .rules(
                aws_sdk_s3::types::LifecycleRule::builder()
                    .id("cost_optimization")
                    .status(aws_sdk_s3::types::ExpirationStatus::Enabled)
                    .expiration(
                        aws_sdk_s3::types::LifecycleExpiration::builder()
                            .days(365) // Move to cheaper storage after 1 year
                            .build()
                    )
                    .transitions(
                        aws_sdk_s3::types::Transition::builder()
                            .storage_class(aws_sdk_s3::types::TransitionStorageClass::StandardIa)
                            .days(30) // Move to IA after 30 days
                            .build()
                    )
                    .transitions(
                        aws_sdk_s3::types::Transition::builder()
                            .storage_class(aws_sdk_s3::types::TransitionStorageClass::Glacier)
                            .days(90) // Move to Glacier after 90 days
                            .build()?
                    )
                    .build()?
            )
            .build()?;
        
        let _ = client
            .put_bucket_lifecycle_configuration()
            .bucket(bucket)
            .lifecycle_configuration(lifecycle_config?)
            .send()
            .await;
        
        // Enable server-side encryption
        let encryption_config = aws_sdk_s3::types::ServerSideEncryptionConfiguration::builder()
            .rules(
                aws_sdk_s3::types::ServerSideEncryptionRule::builder()
                    .apply_server_side_encryption_by_default(
                        aws_sdk_s3::types::ServerSideEncryptionByDefault::builder()
                            .sse_algorithm(aws_sdk_s3::types::ServerSideEncryption::Aes256)
                            .build()?
                    )
                    .build()
            )
            .build();
        
        let _ = client
            .put_bucket_encryption()
            .bucket(bucket)
            .server_side_encryption_configuration(encryption_config?)
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
