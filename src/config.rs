use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub media_gateway_api_key: String,
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
        let mut errors = Vec::new();

        let host = match std::env::var("HOST") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => {
                errors.push("HOST is missing or empty");
                String::new()
            }
        };

        let port = match std::env::var("PORT") {
            Ok(v) if !v.trim().is_empty() => v.parse().context("Invalid PORT")?,
            _ => {
                errors.push("PORT is missing or empty");
                0
            }
        };

        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL is required")?;
        let redis_url = std::env::var("REDIS_URL")
            .context("REDIS_URL is required")?;
        let jwt_secret = std::env::var("JWT_SECRET")
            .context("JWT_SECRET is required (must match backend)")?;

        let media_gateway_api_key = std::env::var("MEDIA_GATEWAY_API_KEY")
            .context("MEDIA_GATEWAY_API_KEY is required (for internal backend communication)")?;

        let s3 = S3Config {
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
        };

        let bunny_cdn_url = match std::env::var("BUNNY_CDN_URL") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => {
                errors.push("BUNNY_CDN_URL is missing or empty");
                String::new()
            }
        };

        let max_daily_bytes = match std::env::var("TRAFFIC_MAX_DAILY_BYTES") {
            Ok(v) if !v.trim().is_empty() => v.parse().context("Invalid TRAFFIC_MAX_DAILY_BYTES")?,
            _ => {
                errors.push("TRAFFIC_MAX_DAILY_BYTES is missing or empty");
                0
            }
        };

        let max_monthly_bytes = match std::env::var("TRAFFIC_MAX_MONTHLY_BYTES") {
            Ok(v) if !v.trim().is_empty() => v.parse().context("Invalid TRAFFIC_MAX_MONTHLY_BYTES")?,
            _ => {
                errors.push("TRAFFIC_MAX_MONTHLY_BYTES is missing or empty");
                0
            }
        };

        let max_daily_downloads = match std::env::var("TRAFFIC_MAX_DAILY_DOWNLOADS") {
            Ok(v) if !v.trim().is_empty() => v.parse().context("Invalid TRAFFIC_MAX_DAILY_DOWNLOADS")?,
            _ => {
                errors.push("TRAFFIC_MAX_DAILY_DOWNLOADS is missing or empty");
                0
            }
        };

        let max_user_daily_bytes = match std::env::var("TRAFFIC_MAX_USER_DAILY_BYTES") {
            Ok(v) if !v.trim().is_empty() => v.parse().context("Invalid TRAFFIC_MAX_USER_DAILY_BYTES")?,
            _ => {
                errors.push("TRAFFIC_MAX_USER_DAILY_BYTES is missing or empty");
                0
            }
        };

        let max_user_daily_downloads = match std::env::var("TRAFFIC_MAX_USER_DAILY_DOWNLOADS") {
            Ok(v) if !v.trim().is_empty() => v.parse().context("Invalid TRAFFIC_MAX_USER_DAILY_DOWNLOADS")?,
            _ => {
                errors.push("TRAFFIC_MAX_USER_DAILY_DOWNLOADS is missing or empty");
                0
            }
        };

        let max_requests_per_hour = match std::env::var("TRAFFIC_MAX_REQUESTS_PER_HOUR") {
            Ok(v) if !v.trim().is_empty() => v.parse().context("Invalid TRAFFIC_MAX_REQUESTS_PER_HOUR")?,
            _ => {
                errors.push("TRAFFIC_MAX_REQUESTS_PER_HOUR is missing or empty");
                0
            }
        };

        let video_enabled = match std::env::var("VIDEO_PROCESSING_ENABLED") {
            Ok(v) if !v.trim().is_empty() => v.parse().context("Invalid VIDEO_PROCESSING_ENABLED")?,
            _ => {
                errors.push("VIDEO_PROCESSING_ENABLED is missing or empty");
                false
            }
        };

        let video_max_upload_size = match std::env::var("VIDEO_MAX_UPLOAD_SIZE") {
            Ok(v) if !v.trim().is_empty() => v.parse().context("Invalid VIDEO_MAX_UPLOAD_SIZE")?,
            _ => {
                errors.push("VIDEO_MAX_UPLOAD_SIZE is missing or empty");
                0
            }
        };

        let video_qualities = match std::env::var("VIDEO_QUALITIES") {
            Ok(v) if !v.trim().is_empty() => v.split(',').map(|s| s.trim().to_string()).collect(),
            _ => {
                errors.push("VIDEO_QUALITIES is missing or empty");
                Vec::new()
            }
        };

        if !errors.is_empty() {
            eprintln!("\n❌ Configuration validation failed:");
            for err in &errors {
                eprintln!("  • {err}");
            }
            anyhow::bail!("Configuration validation failed. See errors above.");
        }

        Ok(Self {
            host,
            port,
            database_url,
            redis_url,
            jwt_secret,
            media_gateway_api_key,
            s3,
            bunny_cdn_url,
            traffic: TrafficLimits {
                max_daily_bytes,
                max_monthly_bytes,
                max_daily_downloads,
                max_user_daily_bytes,
                max_user_daily_downloads,
                max_requests_per_hour,
            },
            video: VideoConfig {
                enabled: video_enabled,
                max_upload_size: video_max_upload_size,
                qualities: video_qualities,
            },
        })
    }
}
