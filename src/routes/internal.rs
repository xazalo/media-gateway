use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};

use crate::AppState;

/// Internal endpoint for backend to fetch media from Object Storage.
/// Protected by API key (not JWT), so only the backend can access it.
pub async fn handle_internal_download(
    Path(key): Path<String>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let (data, content_type) = state.s3.get_object(&key).await.map_err(|e| {
        tracing::error!(key = %key, error = %e, "Failed to get object from S3");
        StatusCode::NOT_FOUND
    })?;

    let size = data.len() as u64;

    // Private storage: no caching, short TTL
    let response = Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, size)
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .body(Body::from(data))
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to build response");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(response)
}
