pub mod auth;
pub mod config;
pub mod db;
pub mod middleware;
pub mod redis;
pub mod routes;
pub mod storage;
pub mod traffic;

use config::Config;
use db::Database;
use redis::RedisClient;
use storage::S3Client;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: RedisClient,
    pub s3: S3Client,
    pub config: Config,
}
