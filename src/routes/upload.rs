use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;
use crate::AppState;

const MAX_IMAGE_BYTES: u64 = 15 * 1024 * 1024;
const MAX_VIDEO_BYTES: u64 = 60 * 1024 * 1024;

const IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/svg+xml",
];

const VIDEO_TYPES: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/quicktime",
    "video/x-m4v",
    "video/ogg",
];

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    /// Relative URL to access media via backend proxy: /api/media/{id}
    /// Note: The {id} will be assigned by the backend after creating the media record
    pub url: String,
    /// Storage key in Object Storage (internal use only, never expose to frontend)
    pub storage_key: String,
    /// File size in bytes
    pub size: u64,
    /// MIME type
    pub mime_type: String,
    /// Original filename
    pub filename: String,
    /// Media ID (UUID) - to be used after backend creates the media record
    /// For now, this is a placeholder that the backend will replace
    pub media_id: Option<String>,
}

pub async fn handle_upload(
    State(state): State<AppState>,
    auth_user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    let filename = field
        .file_name()
        .unwrap_or("file")
        .to_string();

    let data = field
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let is_image = IMAGE_TYPES.contains(&content_type.as_str());
    let is_video = VIDEO_TYPES.contains(&content_type.as_str());

    if !is_image && !is_video {
        tracing::error!("Unsupported content type: {}", content_type);
        return Err(StatusCode::BAD_REQUEST);
    }

    let max_size = if is_video { MAX_VIDEO_BYTES } else { MAX_IMAGE_BYTES };
    if data.len() as u64 > max_size {
        tracing::error!("File too large: {} bytes", data.len());
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let key = state.s3.generate_key(&filename);
    state
        .s3
        .upload(&key, data.clone(), &content_type)
        .await
        .map_err(|e| {
            tracing::error!("Failed to upload: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(
        user_id = %auth_user.id,
        key = %key,
        size = data.len(),
        mime = %content_type,
        "File uploaded"
    );

    // Return storage_key for backend to create media record
    // The backend will assign a media_id and return /api/media/{id} to the frontend
    Ok(Json(UploadResponse {
        url: String::new(), // Will be set by backend after creating media record
        storage_key: key,
        size: data.len() as u64,
        mime_type: content_type,
        filename,
        media_id: None, // Will be set by backend
    }))
}
