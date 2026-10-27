use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub s3: S3Config,
    pub bunny_cdn_url: String,
    pub traffic: TrafficLimits,
    pub video: VideoConfig,
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone)]
pub struct TrafficLimits {
    pub max_daily_bytes: u64,
    pub max_monthly_bytes: u64,
    pub max_daily_downloads: u64,
    pub max_user_daily_bytes: u64,
    pub max_user_daily_downloads: u64,
    pub max_requests_per_hour: u32,
}

#[derive(Debug, Clone)]
pub struct VideoConfig {
    pub enabled: bool,
    pub max_upload_size: u64,
    pub qualities: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            // 3004, not 3003: Docker already maps the API to 3003 (backend
            // Dockerfile). The old 3003 default made both services collide on
            // a single host. Set PORT explicitly anyway.
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3004".to_string())
                .parse()
                .context("Invalid PORT")?,
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL is required")?,
            redis_url: std::env::var("REDIS_URL")
                .context("REDIS_URL is required")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET is required (must match backend)")?,
            s3: S3Config {
                endpoint: std::env::var("S3_ENDPOINT")
                    .context("S3_ENDPOINT is required")?,
                bucket: std::env::var("S3_BUCKET")
                    .context("S3_BUCKET is required")?,
                region: std::env::var("S3_REGION")
                    .context("S3_REGION is required")?,
                access_key: std::env::var("S3_ACCESS_KEY")
                    .context("S3_ACCESS_KEY is required")?,
                secret_key: std::env::var("S3_SECRET_KEY")
                    .context("S3_SECRET_KEY is required")?,
            },
            bunny_cdn_url: std::env::var("BUNNY_CDN_URL")
                .unwrap_or_else(|_| "https://clubify-media.b-cdn.net".to_string()),
            traffic: TrafficLimits {
                max_daily_bytes: std::env::var("TRAFFIC_MAX_DAILY_BYTES")
                    .unwrap_or_else(|_| "107374182400".to_string())
                    .parse()
                    .context("Invalid TRAFFIC_MAX_DAILY_BYTES")?,
                max_monthly_bytes: std::env::var("TRAFFIC_MAX_MONTHLY_BYTES")
                    .unwrap_or_else(|_| "2199023255552".to_string())
                    .parse()
                    .context("Invalid TRAFFIC_MAX_MONTHLY_BYTES")?,
                max_daily_downloads: std::env::var("TRAFFIC_MAX_DAILY_DOWNLOADS")
                    .unwrap_or_else(|_| "50000".to_string())
                    .parse()
                    .context("Invalid TRAFFIC_MAX_DAILY_DOWNLOADS")?,
                max_user_daily_bytes: std::env::var("TRAFFIC_MAX_USER_DAILY_BYTES")
                    .unwrap_or_else(|_| "10737418240".to_string())
                    .parse()
                    .context("Invalid TRAFFIC_MAX_USER_DAILY_BYTES")?,
                max_user_daily_downloads: std::env::var("TRAFFIC_MAX_USER_DAILY_DOWNLOADS")
                    .unwrap_or_else(|_| "500".to_string())
                    .parse()
                    .context("Invalid TRAFFIC_MAX_USER_DAILY_DOWNLOADS")?,
                max_requests_per_hour: std::env::var("TRAFFIC_MAX_REQUESTS_PER_HOUR")
                    .unwrap_or_else(|_| "5000".to_string())
                    .parse()
                    .context("Invalid TRAFFIC_MAX_REQUESTS_PER_HOUR")?,
            },
            video: VideoConfig {
                enabled: std::env::var("VIDEO_PROCESSING_ENABLED")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .context("Invalid VIDEO_PROCESSING_ENABLED")?,
                max_upload_size: std::env::var("VIDEO_MAX_UPLOAD_SIZE")
                    .unwrap_or_else(|_| "524288000".to_string())
                    .parse()
                    .context("Invalid VIDEO_MAX_UPLOAD_SIZE")?,
                qualities: std::env::var("VIDEO_QUALITIES")
                    .unwrap_or_else(|_| "360p,720p,1080p".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
        })
    }
}
