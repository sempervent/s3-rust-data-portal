use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::operation::put_object::PutObjectOutput;
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::primitives::ByteStream;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, warn, info};
use url::Url;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

pub struct ResilientS3Client {
    client: S3Client,
    retry_config: RetryConfig,
}

impl ResilientS3Client {
    pub fn new(client: S3Client, retry_config: Option<RetryConfig>) -> Self {
        Self {
            client,
            retry_config: retry_config.unwrap_or_default(),
        }
    }

    pub async fn put_object_with_retry(
        &self,
        bucket: &str,
        key: &str,
        body: ByteStream,
        content_type: Option<&str>,
    ) -> Result<PutObjectOutput, aws_sdk_s3::Error> {
        let mut delay = self.retry_config.initial_delay;
        
        for attempt in 0..=self.retry_config.max_retries {
            match self.put_object_single(bucket, key, body.clone(), content_type).await {
                Ok(output) => {
                    if attempt > 0 {
                        info!("S3 put_object succeeded after {} retries", attempt);
                    }
                    return Ok(output);
                }
                Err(e) => {
                    if attempt == self.retry_config.max_retries {
                        error!("S3 put_object failed after {} retries: {}", attempt + 1, e);
                        return Err(e);
                    }
                    
                    warn!("S3 put_object attempt {} failed: {}, retrying in {:?}", attempt + 1, e, delay);
                    sleep(delay).await;
                    
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * self.retry_config.backoff_multiplier) as u64),
                        self.retry_config.max_delay,
                    );
                }
            }
        }
        
        unreachable!()
    }

    async fn put_object_single(
        &self,
        bucket: &str,
        key: &str,
        body: ByteStream,
        content_type: Option<&str>,
    ) -> Result<PutObjectOutput, aws_sdk_s3::Error> {
        let mut request = self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(body);

        if let Some(ct) = content_type {
            request = request.content_type(ct);
        }

        request.send().await
    }

    pub async fn get_object_with_retry(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<GetObjectOutput, aws_sdk_s3::Error> {
        let mut delay = self.retry_config.initial_delay;
        
        for attempt in 0..=self.retry_config.max_retries {
            match self.get_object_single(bucket, key).await {
                Ok(output) => {
                    if attempt > 0 {
                        info!("S3 get_object succeeded after {} retries", attempt);
                    }
                    return Ok(output);
                }
                Err(e) => {
                    if attempt == self.retry_config.max_retries {
                        error!("S3 get_object failed after {} retries: {}", attempt + 1, e);
                        return Err(e);
                    }
                    
                    warn!("S3 get_object attempt {} failed: {}, retrying in {:?}", attempt + 1, e, delay);
                    sleep(delay).await;
                    
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * self.retry_config.backoff_multiplier) as u64),
                        self.retry_config.max_delay,
                    );
                }
            }
        }
        
        unreachable!()
    }

    async fn get_object_single(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<GetObjectOutput, aws_sdk_s3::Error> {
        self.client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
    }

    pub async fn ensure_bucket_with_versioning_and_encryption(
        &self,
        bucket: &str,
    ) -> Result<(), aws_sdk_s3::Error> {
        // Check if bucket exists
        match self.client.head_bucket().bucket(bucket).send().await {
            Ok(_) => {
                info!("Bucket {} already exists", bucket);
            }
            Err(_) => {
                info!("Creating bucket {}", bucket);
                self.client
                    .create_bucket()
                    .bucket(bucket)
                    .send()
                    .await?;
            }
        }

        // Enable versioning
        info!("Enabling versioning for bucket {}", bucket);
        self.client
            .put_bucket_versioning()
            .bucket(bucket)
            .versioning_configuration(
                aws_sdk_s3::types::VersioningConfiguration::builder()
                    .status(aws_sdk_s3::types::BucketVersioningStatus::Enabled)
                    .build(),
            )
            .send()
            .await?;

        // Enable server-side encryption
        info!("Enabling server-side encryption for bucket {}", bucket);
        self.client
            .put_bucket_encryption()
            .bucket(bucket)
            .server_side_encryption_configuration(
                aws_sdk_s3::types::ServerSideEncryptionConfiguration::builder()
                    .rules(
                        aws_sdk_s3::types::ServerSideEncryptionRule::builder()
                            .apply_server_side_encryption_by_default(
                                aws_sdk_s3::types::ServerSideEncryptionByDefault::builder()
                                    .sse_algorithm(aws_sdk_s3::types::ServerSideEncryption::Aes256)
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 1.5,
        };
        
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(200));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.backoff_multiplier, 1.5);
    }
}
