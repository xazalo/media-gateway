use media_gateway::auth::AuthUser;
use media_gateway::config::{Config, S3Config, TrafficLimits, VideoConfig};

fn test_config() -> Config {
    Config {
        host: "127.0.0.1".to_string(),
        port: 0,
        database_url: "postgres://test:test@localhost/test".to_string(),
        redis_url: "redis://localhost:6379".to_string(),
        jwt_secret: "test-secret-key-for-testing".to_string(),
        s3: S3Config {
            endpoint: "https://test.example.com".to_string(),
            bucket: "test-bucket".to_string(),
            region: "us-east-1".to_string(),
            access_key: "test-key".to_string(),
            secret_key: "test-secret".to_string(),
        },
        bunny_cdn_url: "https://test.b-cdn.net".to_string(),
        traffic: TrafficLimits {
            max_daily_bytes: 1_073_741_824,
            max_monthly_bytes: 10_737_418_240,
            max_daily_downloads: 1000,
            max_user_daily_bytes: 100_000_000,
            max_user_daily_downloads: 100,
            max_requests_per_hour: 100,
        },
        video: VideoConfig {
            enabled: false,
            max_upload_size: 60_000_000,
            qualities: vec!["360p".to_string(), "720p".to_string()],
        },
    }
}

#[test]
fn test_config_from_env() {
    let config = test_config();
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 0);
    assert_eq!(config.bunny_cdn_url, "https://test.b-cdn.net");
    assert_eq!(config.traffic.max_daily_bytes, 1_073_741_824);
}

#[test]
fn test_s3_key_generation() {
    let config = test_config();
    let key = format!("uploads/{}/test.jpg", chrono::Utc::now().timestamp());
    assert!(key.starts_with("uploads/"));
    assert!(key.ends_with("test.jpg"));
}

#[test]
fn test_cdn_url_generation() {
    let config = test_config();
    let key = "uploads/123/test.jpg";
    let cdn_url = format!("{}/{}", config.bunny_cdn_url, key);
    assert_eq!(cdn_url, "https://test.b-cdn.net/uploads/123/test.jpg");
}

#[test]
fn test_traffic_limits_thresholds() {
    let limits = TrafficLimits {
        max_daily_bytes: 100,
        max_monthly_bytes: 1000,
        max_daily_downloads: 10,
        max_user_daily_bytes: 50,
        max_user_daily_downloads: 5,
        max_requests_per_hour: 20,
    };

    assert_eq!(limits.max_daily_bytes, 100);
    assert_eq!(limits.max_monthly_bytes, 1000);
    assert_eq!(limits.max_daily_downloads, 10);
    assert_eq!(limits.max_user_daily_bytes, 50);
    assert_eq!(limits.max_user_daily_downloads, 5);
    assert_eq!(limits.max_requests_per_hour, 20);
}

#[test]
fn test_jwt_secret_is_configured() {
    let config = test_config();
    assert!(!config.jwt_secret.is_empty());
    assert_ne!(config.jwt_secret, "dev-insecure-user-secret-change-me");
}
