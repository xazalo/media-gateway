use anyhow::{Context, Result};
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use bytes::Bytes;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
    bunny_cdn_url: String,
}

impl S3Client {
    pub fn new(
        endpoint: &str,
        region: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
    ) -> Result<Self> {
        let creds = Credentials::new(access_key, secret_key, None, None, "hetzner");
        let config = S3ConfigBuilder::new()
            .region(Region::new(region.to_string()))
            .endpoint_url(endpoint)
            .credentials_provider(creds)
            .force_path_style(true) // Required for S3-compatible services
            .build();

        let client = Client::from_conf(config);
        Ok(Self {
            client,
            bucket: bucket.to_string(),
            bunny_cdn_url: std::env::var("BUNNY_CDN_URL")
                .unwrap_or_else(|_| "https://clubify-media.b-cdn.net".to_string()),
        })
    }

    // Generate object key: uploads/{timestamp}/{uuid}-{filename}
    pub fn generate_key(&self, filename: &str) -> String {
        let timestamp = chrono::Utc::now().timestamp();
        let uuid = Uuid::new_v4();
        format!("uploads/{}/{}-{}", timestamp, uuid, filename)
    }

    // Upload file to S3
    pub async fn upload(
        &self,
        key: &str,
        data: Bytes,
        content_type: &str,
    ) -> Result<()> {
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .context("Failed to upload to S3")?;

        Ok(())
    }

    // Get object from S3
    pub async fn get_object(&self, key: &str) -> Result<(Bytes, String)> {
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .context("Failed to get object from S3")?;

        let content_type = result
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let data = result
            .body
            .collect()
            .await
            .context("Failed to read object body")?
            .into_bytes();

        Ok((data, content_type))
    }

    // Delete object from S3
    pub async fn delete(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .context("Failed to delete object from S3")?;

        Ok(())
    }

    // Check if object exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    // Generate Bunny CDN URL for a given key
    pub fn get_cdn_url(&self, key: &str) -> String {
        format!("{}/{}", self.bunny_cdn_url, key)
    }

    // Note: Presigned URLs are not implemented yet
    // For private files, use the download endpoint with authentication
    // or implement Bunny.net hotlink protection
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MediaInfo {
    pub key: String,
    pub size: u64,
    pub content_type: String,
    pub cdn_url: String,
}
