use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use media_gateway::{
    auth, config::Config, db::Database, middleware as mw, redis::RedisClient, routes,
    storage::S3Client, AppState,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "media_gateway=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting media-gateway service");

    let config = Config::from_env()?;
    let host = config.host.clone();
    let port = config.port;
    tracing::info!("Loaded configuration");

    let db = Database::new(&config.database_url).await?;
    tracing::info!("Connected to PostgreSQL");

    let redis = RedisClient::new(&config.redis_url).await?;
    tracing::info!("Connected to Redis");

    let s3 = S3Client::new(
        &config.s3.endpoint,
        &config.s3.region,
        &config.s3.access_key,
        &config.s3.secret_key,
        &config.s3.bucket,
    )?;
    tracing::info!("Initialized S3 client");

    db.run_migrations().await?;
    tracing::info!("Database migrations complete");

    let state = AppState {
        db,
        redis,
        s3,
        config,
    };

    let upload_router = Router::new()
        .route("/media/upload", post(routes::upload::handle_upload))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_required,
        ));

    let download_router = Router::new()
        .route("/media/{key}", get(routes::download::handle_download))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::optional_auth,
        ));

    let app = Router::new()
        .route("/health", get(health_check))
        .merge(upload_router)
        .merge(download_router)
        .route(
            "/media/{key}/info",
            get(routes::download::handle_media_info),
        )
        .route(
            "/admin/traffic/stats",
            get(routes::admin::handle_traffic_stats),
        )
        .route(
            "/admin/traffic/reset",
            post(routes::admin::handle_reset_counters),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            mw::rate_limit,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid address");
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
