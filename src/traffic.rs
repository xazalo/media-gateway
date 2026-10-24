use crate::config::TrafficLimits;
use crate::db::Database;
use crate::redis::RedisClient;
use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TrafficTracker {
    db: Database,
    redis: RedisClient,
    limits: TrafficLimits,
}

#[derive(Debug, Clone)]
pub struct LimitStatus {
    pub can_serve: bool,
    pub throttled: bool,
    pub action: LimitAction,
    pub daily_usage: DailyUsage,
    pub monthly_usage: MonthlyUsage,
    pub user_daily_usage: Option<UserDailyUsage>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LimitAction {
    Allow,
    Warning,
    Throttle,
    BlockNew,
    BlockAll,
}

#[derive(Debug, Clone)]
pub struct DailyUsage {
    pub bytes: u64,
    pub downloads: u64,
    pub requests: u64,
}

#[derive(Debug, Clone)]
pub struct MonthlyUsage {
    pub bytes: u64,
    pub downloads: u64,
    pub requests: u64,
}

#[derive(Debug, Clone)]
pub struct UserDailyUsage {
    pub bytes: u64,
    pub downloads: u64,
}

impl TrafficTracker {
    pub fn new(db: Database, redis: RedisClient, limits: TrafficLimits) -> Self {
        Self { db, redis, limits }
    }

    // Check if we can serve a request
    pub async fn check_limits(&self, user_id: Option<Uuid>) -> Result<LimitStatus> {
        let daily_usage = self.redis.get_daily_usage(Utc::now().date_naive()).await?;
        let monthly_usage = self.redis.get_monthly_usage().await?;
        let user_daily_usage = if let Some(uid) = user_id {
            Some(self.redis.get_user_daily_usage(uid).await?)
        } else {
            None
        };

        let action = self.decide_action(&daily_usage, &monthly_usage, &user_daily_usage);

        let can_serve = action != LimitAction::BlockAll && action != LimitAction::BlockNew;
        let throttled = action == LimitAction::Throttle;

        Ok(LimitStatus {
            can_serve,
            throttled,
            action,
            daily_usage: DailyUsage {
                bytes: daily_usage.bytes,
                downloads: daily_usage.downloads,
                requests: daily_usage.requests,
            },
            monthly_usage: MonthlyUsage {
                bytes: monthly_usage.bytes,
                downloads: monthly_usage.downloads,
                requests: monthly_usage.requests,
            },
            user_daily_usage: user_daily_usage.map(|u| UserDailyUsage {
                bytes: u.bytes,
                downloads: u.downloads,
            }),
        })
    }

    // Record a download
    pub async fn record_download(&self, user_id: Option<Uuid>, bytes: u64) -> Result<()> {
        // Update Redis (fast)
        self.redis.increment_daily_usage(bytes).await?;
        self.redis.increment_monthly_usage(bytes).await?;
        if let Some(uid) = user_id {
            self.redis.increment_user_daily_usage(uid, bytes).await?;
        }

        // Update PostgreSQL (persistent)
        let today = Utc::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

        self.db.increment_daily_traffic(today, "bytes", bytes as i64).await?;
        self.db.increment_daily_traffic(today, "downloads", 1).await?;
        self.db.increment_daily_traffic(today, "requests", 1).await?;

        self.db.increment_monthly_traffic(month_start, "bytes", bytes as i64).await?;
        self.db.increment_monthly_traffic(month_start, "downloads", 1).await?;
        self.db.increment_monthly_traffic(month_start, "requests", 1).await?;

        if let Some(uid) = user_id {
            self.db.increment_user_daily_traffic(today, uid, "bytes", bytes as i64).await?;
            self.db.increment_user_daily_traffic(today, uid, "downloads", 1).await?;
        }

        Ok(())
    }

    // Decide action based on usage
    fn decide_action(
        &self,
        daily: &crate::redis::DailyUsage,
        monthly: &crate::redis::MonthlyUsage,
        user_daily: &Option<crate::redis::UserDailyUsage>,
    ) -> LimitAction {
        let daily_bytes_pct = (daily.bytes as f64 / self.limits.max_daily_bytes as f64) * 100.0;
        let monthly_bytes_pct = (monthly.bytes as f64 / self.limits.max_monthly_bytes as f64) * 100.0;
        let daily_downloads_pct = (daily.downloads as f64 / self.limits.max_daily_downloads as f64) * 100.0;

        // Check user limits
        if let Some(user) = user_daily {
            let user_bytes_pct = (user.bytes as f64 / self.limits.max_user_daily_bytes as f64) * 100.0;
            let user_downloads_pct = (user.downloads as f64 / self.limits.max_user_daily_downloads as f64) * 100.0;

            if user_bytes_pct >= 100.0 || user_downloads_pct >= 100.0 {
                return LimitAction::BlockNew; // Block this user
            }
        }

        // Check global limits
        let max_pct = daily_bytes_pct.max(monthly_bytes_pct).max(daily_downloads_pct);

        if max_pct >= 100.0 {
            LimitAction::BlockAll
        } else if max_pct >= 95.0 {
            LimitAction::BlockNew
        } else if max_pct >= 90.0 {
            LimitAction::Throttle
        } else if max_pct >= 75.0 {
            LimitAction::Warning
        } else {
            LimitAction::Allow
        }
    }

    // Get traffic statistics
    pub async fn get_stats(&self) -> Result<TrafficStats> {
        let today = Utc::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

        let daily = self.db.get_daily_stats(today).await?;
        let monthly = self.db.get_monthly_stats(month_start).await?;

        Ok(TrafficStats {
            daily,
            monthly,
            limits: TrafficLimitsInfo {
                max_daily_bytes: self.limits.max_daily_bytes,
                max_monthly_bytes: self.limits.max_monthly_bytes,
                max_daily_downloads: self.limits.max_daily_downloads,
            },
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TrafficStats {
    pub daily: crate::db::DailyStats,
    pub monthly: crate::db::MonthlyStats,
    pub limits: TrafficLimitsInfo,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TrafficLimitsInfo {
    pub max_daily_bytes: u64,
    pub max_monthly_bytes: u64,
    pub max_daily_downloads: u64,
}
