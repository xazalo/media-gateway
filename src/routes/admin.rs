use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::traffic::TrafficTracker;
use crate::AppState;

pub async fn handle_traffic_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let tracker = TrafficTracker::new(
        state.db.clone(),
        state.redis.clone(),
        state.config.traffic.clone(),
    );
    let stats = tracker.get_stats().await.map_err(|e| {
        tracing::error!("Failed to get stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::to_value(stats).unwrap()))
}

pub async fn handle_reset_counters(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.db.reset_all_counters().await.map_err(|e| {
        tracing::error!("Failed to reset counters: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": "Counters reset successfully"
    })))
}
