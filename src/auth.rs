use axum::{
    extract::{FromRequestParts, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: Uuid,
    pub jti: Uuid,
    pub token_version: i32,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
}

impl AuthUser {
    fn verify_token(token: &str, secret: &str) -> Result<UserClaims, AuthError> {
        let data = decode::<UserClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| {
            tracing::warn!("JWT validation failed: {}", e);
            AuthError::InvalidToken
        })?;
        Ok(data.claims)
    }
}

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    TokenExpired,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::MissingToken => {
                (StatusCode::UNAUTHORIZED, "missing authentication token").into_response()
            }
            AuthError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "invalid authentication token").into_response()
            }
            AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "token expired").into_response()
            }
        }
    }
}

pub async fn auth_required(
    State(state): State<AppState>,
    mut req: axum::extract::Request,
    next: Next,
) -> Result<Response, AuthError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::MissingToken)?;

    let claims = AuthUser::verify_token(token, &state.config.jwt_secret)?;

    req.extensions_mut().insert(AuthUser { id: claims.sub });
    Ok(next.run(req).await)
}

pub async fn optional_auth(
    State(state): State<AppState>,
    mut req: axum::extract::Request,
    next: Next,
) -> Response {
    if let Some(auth_header) = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if let Ok(claims) = AuthUser::verify_token(token, &state.config.jwt_secret) {
                req.extensions_mut().insert(AuthUser { id: claims.sub });
                return next.run(req).await;
            }
        }
    }

    next.run(req).await
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut http::request::Parts,
        _state: &'life1 S,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            parts
                .extensions
                .get::<AuthUser>()
                .cloned()
                .ok_or(AuthError::MissingToken)
        })
    }
}

pub struct MaybeAuthUser(pub Option<AuthUser>);

impl<S> FromRequestParts<S> for MaybeAuthUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut http::request::Parts,
        _state: &'life1 S,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            Ok(MaybeAuthUser(parts.extensions.get::<AuthUser>().cloned()))
        })
    }
}
