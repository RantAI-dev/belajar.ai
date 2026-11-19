pub(crate) mod auth;
pub(crate) mod health;
pub(crate) mod me;
pub(crate) mod openapi;
pub(crate) mod root;

use axum::Router;
use axum::extract::Extension;
use axum::response::Response;
use axum::routing::get;

use crate::RequestId;
use crate::api_error::ApiError;
use crate::error_codes::ErrorCode;
use crate::state::AppState;

pub fn v1() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::v1_health))
        .route("/openapi.json", get(openapi::openapi_json))
        .route("/me", get(me::get_me).patch(me::patch_me))
        .merge(auth::routes())
        .fallback(not_found)
}

/// Unknown `/v1` paths answer with the standard error body, not the router's plain-text 404.
pub async fn not_found(Extension(id): Extension<RequestId>) -> Response {
    ApiError::new(ErrorCode::NotFound).into_response(&id.0)
}
