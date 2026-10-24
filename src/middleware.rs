use axum::{
    extract::{ConnectInfo, State, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;

use crate::AppState;

pub async fn rate_limit(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(&req);

    if let Some(ip) = &client_ip {
        let allowed = state
            .redis
            .check_ip_rate_limit(ip, state.config.traffic.max_requests_per_hour)
            .await
            .unwrap_or(true);

        if !allowed {
            tracing::warn!("Rate limit exceeded for IP: {}", ip);
            return (
                StatusCode::TOO_MANY_REQUESTS,
                "rate limit exceeded, try again later",
            )
                .into_response();
        }
    }

    if let Some(ip) = client_ip {
        req.extensions_mut().insert(ClientIp(ip));
    }
    next.run(req).await
}

#[derive(Debug, Clone)]
pub struct ClientIp(pub String);

fn extract_client_ip(req: &Request) -> Option<String> {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            req.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
        })
}
