//! One place that turns a `StoreError` into a response. The driver's text stays in the log; the
//! body says only what a client can act on.

use axum::response::Response;

use crate::api_error::ApiError;
use crate::error_codes::ErrorCode;
use belajar_ports::store::StoreError;

pub fn into_response(err: StoreError, request_id: &str) -> Response {
    match err {
        StoreError::NotFound => ApiError::new(ErrorCode::NotFound).into_response(request_id),
        StoreError::Conflict => ApiError::new(ErrorCode::Conflict).into_response(request_id),
        StoreError::Internal(cause) => {
            tracing::error!(request_id, code = "internal", error = %cause, "store failed");
            ApiError::new(ErrorCode::Internal).into_response(request_id)
        }
    }
}
