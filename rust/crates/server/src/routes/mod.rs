mod health;

use axum::Router;
use axum::extract::Extension;
use axum::response::Response;
use axum::routing::get;

use crate::RequestId;
use crate::api_error::ApiError;
use crate::error_codes::ErrorCode;

pub fn v1() -> Router {
    Router::new()
        .route("/health", get(health::get))
        .fallback(not_found)
}

/// Unknown `/v1` paths answer with the standard error body, not the router's plain-text 404.
async fn not_found(Extension(id): Extension<RequestId>) -> Response {
    ApiError::new(ErrorCode::NotFound).into_response(&id.0)
}
