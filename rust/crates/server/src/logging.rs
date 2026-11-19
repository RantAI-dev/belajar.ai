//! One JSON line per request at `info`: method, route template, status, duration_ms and the
//! request id. Errors that become a 5xx are logged once, where they become the response (in the
//! store or password mappers), not here.

use axum::extract::{MatchedPath, Request};
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;

use crate::RequestId;

pub async fn trace(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    // Unknown paths (the static web app's fallback) have no template; the path itself is not
    // logged, so equal requests still group.
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "-".to_owned(), |m| m.as_str().to_owned());
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map_or_else(|| "-".to_owned(), |id| id.0.clone());
    let start = Instant::now();
    let response = next.run(request).await;
    let duration_ms = start.elapsed().as_millis() as u64;
    tracing::info!(
        method = %method,
        route = %route,
        status = response.status().as_u16(),
        duration_ms,
        request_id = %request_id,
        "request"
    );
    response
}
