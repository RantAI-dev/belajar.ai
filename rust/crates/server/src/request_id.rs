use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

use crate::api_error::sent_request_id;

/// The request's id: the caller's `X-Request-Id` when it is safe to echo, otherwise a new UUIDv7.
/// Every response carries it back, and every log line for the request names it.
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

pub async fn stamp(mut request: Request, next: Next) -> Response {
    let id = sent_request_id(request.headers())
        .map_or_else(|| uuid::Uuid::now_v7().to_string(), str::to_owned);
    request.extensions_mut().insert(RequestId(id.clone()));
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}
