use anyhow::Result;
use chrono::{NaiveDate, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        let migration_sql = include_str!("../sql/001_traffic_tracking.sql");
        sqlx::query(migration_sql).execute(&self.pool).await?;
        Ok(())
    }

    // Daily traffic counters
    pub async fn increment_daily_traffic(
        &self,
        date: NaiveDate,
        metric_type: &str,
        amount: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO traffic_daily (date, metric_type, count, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (date, metric_type)
            DO UPDATE SET count = traffic_daily.count + $3, updated_at = NOW()
            "#,
        )
        .bind(date)
        .bind(metric_type)
        .bind(amount)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Monthly traffic counters
    pub async fn increment_monthly_traffic(
        &self,
        month: NaiveDate,
        metric_type: &str,
        amount: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO traffic_monthly (month, metric_type, count, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (month, metric_type)
            DO UPDATE SET count = traffic_monthly.count + $3, updated_at = NOW()
            "#,
        )
        .bind(month)
        .bind(metric_type)
        .bind(amount)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Per-user daily traffic counters
    pub async fn increment_user_daily_traffic(
        &self,
        date: NaiveDate,
        user_id: Uuid,
        metric_type: &str,
        amount: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO traffic_user_daily (date, user_id, metric_type, count, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (date, user_id, metric_type)
            DO UPDATE SET count = traffic_user_daily.count + $4, updated_at = NOW()
            "#,
        )
        .bind(date)
        .bind(user_id)
        .bind(metric_type)
        .bind(amount)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Get daily traffic stats
    pub async fn get_daily_stats(&self, date: NaiveDate) -> Result<DailyStats> {
        let rows: Vec<(String, i64)> = sqlx::query_scalar(
            "SELECT metric_type, count FROM traffic_daily WHERE date = $1",
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;

        let mut stats = DailyStats::default();
        for (metric, count) in rows {
            match metric.as_str() {
                "bytes" => stats.bytes = count as u64,
                "downloads" => stats.downloads = count as u64,
                "requests" => stats.requests = count as u64,
                _ => {}
            }
        }
        Ok(stats)
    }

    // Get monthly traffic stats
    pub async fn get_monthly_stats(&self, month: NaiveDate) -> Result<MonthlyStats> {
        let rows: Vec<(String, i64)> = sqlx::query_scalar(
            "SELECT metric_type, count FROM traffic_monthly WHERE month = $1",
        )
        .bind(month)
        .fetch_all(&self.pool)
        .await?;

        let mut stats = MonthlyStats::default();
        for (metric, count) in rows {
            match metric.as_str() {
                "bytes" => stats.bytes = count as u64,
                "downloads" => stats.downloads = count as u64,
                "requests" => stats.requests = count as u64,
                _ => {}
            }
        }
        Ok(stats)
    }

    // Get user daily stats
    pub async fn get_user_daily_stats(
        &self,
        date: NaiveDate,
        user_id: Uuid,
    ) -> Result<UserDailyStats> {
        let rows: Vec<(String, i64)> = sqlx::query_scalar(
            "SELECT metric_type, count FROM traffic_user_daily WHERE date = $1 AND user_id = $2",
        )
        .bind(date)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut stats = UserDailyStats::default();
        for (metric, count) in rows {
            match metric.as_str() {
                "bytes" => stats.bytes = count as u64,
                "downloads" => stats.downloads = count as u64,
                _ => {}
            }
        }
        Ok(stats)
    }

    // Reset counters (for testing)
    pub async fn reset_all_counters(&self) -> Result<()> {
        sqlx::query("DELETE FROM traffic_daily").execute(&self.pool).await?;
        sqlx::query("DELETE FROM traffic_monthly").execute(&self.pool).await?;
        sqlx::query("DELETE FROM traffic_user_daily").execute(&self.pool).await?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct DailyStats {
    pub bytes: u64,
    pub downloads: u64,
    pub requests: u64,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct MonthlyStats {
    pub bytes: u64,
    pub downloads: u64,
    pub requests: u64,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct UserDailyStats {
    pub bytes: u64,
    pub downloads: u64,
}
