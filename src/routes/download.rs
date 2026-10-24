use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::auth::MaybeAuthUser;
use crate::traffic::TrafficTracker;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct MediaInfoResponse {
    pub key: String,
    pub cdn_url: String,
    pub size: u64,
    pub content_type: String,
}

pub async fn handle_download(
    Path(key): Path<String>,
    State(state): State<AppState>,
    MaybeAuthUser(auth_user): MaybeAuthUser,
) -> Result<Response, StatusCode> {
    let tracker = TrafficTracker::new(
        state.db.clone(),
        state.redis.clone(),
        state.config.traffic.clone(),
    );

    let user_id = auth_user.map(|u| u.id);
    let status = tracker.check_limits(user_id).await.map_err(|e| {
        tracing::error!("Failed to check limits: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if !status.can_serve {
        tracing::warn!(
            user_id = ?user_id,
            action = ?status.action,
            "Traffic limit reached, blocking download"
        );
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    let (data, content_type) = state.s3.get_object(&key).await.map_err(|e| {
        tracing::error!("Failed to get object: {}", e);
        StatusCode::NOT_FOUND
    })?;

    let size = data.len() as u64;

    if let Err(e) = tracker.record_download(user_id, size).await {
        tracing::error!("Failed to record download: {}", e);
    }

    let mut response = Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, size)
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .header("X-Traffic-Action", format!("{:?}", status.action));

    if status.throttled {
        response = response.header("X-RateLimit-Remaining", "0");
    }

    response
        .body(Body::from(data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn handle_media_info(
    Path(key): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<MediaInfoResponse>, StatusCode> {
    let exists = state
        .s3
        .exists(&key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }

    let (data, content_type) = state
        .s3
        .get_object(&key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let size = data.len() as u64;
    let cdn_url = state.s3.get_cdn_url(&key);

    Ok(Json(MediaInfoResponse {
        key,
        cdn_url,
        size,
        content_type,
    }))
}
