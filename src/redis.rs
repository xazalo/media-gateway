use anyhow::Result;
use chrono::{Datelike, NaiveDate, Timelike, Utc};
use redis::AsyncCommands;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }

    // Rate limiting: check if request is allowed
    pub async fn check_rate_limit(&self, key: &str, max_requests: u32, window_secs: u64) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let current: u64 = redis::cmd("INCR")
            .arg(key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        // Set expiration on first request
        if current == 1 {
            redis::cmd("EXPIRE")
                .arg(key)
                .arg(window_secs)
                .query_async::<_, ()>(&mut conn)
                .await?;
        }

        Ok(current <= max_requests as u64)
    }

    // Get current usage from Redis (fast cache)
    pub async fn get_daily_usage(&self, date: NaiveDate) -> Result<DailyUsage> {
        let mut conn = self.get_connection().await?;
        let key_prefix = format!("traffic:daily:{}", date.format("%Y-%m-%d"));

        let bytes: u64 = conn.get(format!("{}:bytes", key_prefix)).await.unwrap_or(0);
        let downloads: u64 = conn.get(format!("{}:downloads", key_prefix)).await.unwrap_or(0);
        let requests: u64 = conn.get(format!("{}:requests", key_prefix)).await.unwrap_or(0);

        Ok(DailyUsage {
            bytes,
            downloads,
            requests,
        })
    }

    // Increment daily usage in Redis
    pub async fn increment_daily_usage(&self, bytes: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let date = Utc::now().date_naive();
        let key_prefix = format!("traffic:daily:{}", date.format("%Y-%m-%d"));

        // Set TTL to end of day (24h from now, will be cleaned up)
        let ttl = 86400 * 2; // 2 days to be safe

        redis::pipe()
            .atomic()
            .incr(format!("{}:bytes", key_prefix), bytes)
            .expire(format!("{}:bytes", key_prefix), ttl)
            .incr(format!("{}:downloads", key_prefix), 1)
            .expire(format!("{}:downloads", key_prefix), ttl)
            .incr(format!("{}:requests", key_prefix), 1)
            .expire(format!("{}:requests", key_prefix), ttl)
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }

    // Get user daily usage
    pub async fn get_user_daily_usage(&self, user_id: Uuid) -> Result<UserDailyUsage> {
        let mut conn = self.get_connection().await?;
        let date = Utc::now().date_naive();
        let key_prefix = format!("traffic:user:{}:{}", user_id, date.format("%Y-%m-%d"));

        let bytes: u64 = conn.get(format!("{}:bytes", key_prefix)).await.unwrap_or(0);
        let downloads: u64 = conn.get(format!("{}:downloads", key_prefix)).await.unwrap_or(0);

        Ok(UserDailyUsage { bytes, downloads })
    }

    // Increment user daily usage
    pub async fn increment_user_daily_usage(&self, user_id: Uuid, bytes: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let date = Utc::now().date_naive();
        let key_prefix = format!("traffic:user:{}:{}", user_id, date.format("%Y-%m-%d"));

        let ttl = 86400 * 2;

        redis::pipe()
            .atomic()
            .incr(format!("{}:bytes", key_prefix), bytes)
            .expire(format!("{}:bytes", key_prefix), ttl)
            .incr(format!("{}:downloads", key_prefix), 1)
            .expire(format!("{}:downloads", key_prefix), ttl)
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }

    // Get monthly usage
    pub async fn get_monthly_usage(&self) -> Result<MonthlyUsage> {
        let mut conn = self.get_connection().await?;
        let now = Utc::now();
        let key_prefix = format!("traffic:monthly:{}-{:02}", now.year(), now.month());

        let bytes: u64 = conn.get(format!("{}:bytes", key_prefix)).await.unwrap_or(0);
        let downloads: u64 = conn.get(format!("{}:downloads", key_prefix)).await.unwrap_or(0);
        let requests: u64 = conn.get(format!("{}:requests", key_prefix)).await.unwrap_or(0);

        Ok(MonthlyUsage {
            bytes,
            downloads,
            requests,
        })
    }

    // Increment monthly usage
    pub async fn increment_monthly_usage(&self, bytes: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let now = Utc::now();
        let key_prefix = format!("traffic:monthly:{}-{:02}", now.year(), now.month());

        let ttl = 86400 * 32; // ~1 month

        redis::pipe()
            .atomic()
            .incr(format!("{}:bytes", key_prefix), bytes)
            .expire(format!("{}:bytes", key_prefix), ttl)
            .incr(format!("{}:downloads", key_prefix), 1)
            .expire(format!("{}:downloads", key_prefix), ttl)
            .incr(format!("{}:requests", key_prefix), 1)
            .expire(format!("{}:requests", key_prefix), ttl)
            .query_async::<_, ()>(&mut conn)
            .await?;

        Ok(())
    }

    // Rate limit per IP
    pub async fn check_ip_rate_limit(&self, ip: &str, max_per_hour: u32) -> Result<bool> {
        let now = Utc::now();
        let key = format!(
            "ratelimit:{}:{}-{:02}-{:02}-{:02}",
            ip,
            now.year(),
            now.month(),
            now.day(),
            now.hour()
        );
        self.check_rate_limit(&key, max_per_hour, 3600).await
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DailyUsage {
    pub bytes: u64,
    pub downloads: u64,
    pub requests: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserDailyUsage {
    pub bytes: u64,
    pub downloads: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MonthlyUsage {
    pub bytes: u64,
    pub downloads: u64,
    pub requests: u64,
}
