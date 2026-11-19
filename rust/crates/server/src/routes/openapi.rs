//! Serves `ApiDoc` as JSON. The same document is printed by `--bin openapi` and committed at
//! `web/src/contracts/openapi.json`; the acceptance test compares all three.

use axum::extract::Extension;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::RequestId;
use crate::api_error::ApiError;
use crate::error_codes::ErrorCode;
use crate::openapi::ApiDoc;
use utoipa::OpenApi;

/// The OpenAPI document for this API.
#[utoipa::path(
    get,
    path = "/v1/openapi.json",
    responses((status = 200, description = "The OpenAPI document", content_type = "application/json"))
)]
pub(crate) async fn openapi_json(Extension(id): Extension<RequestId>) -> Response {
    match ApiDoc::openapi().to_json() {
        // `to_json` already produces the document as JSON text; serving it raw avoids parsing it
        // just to re-serialize.
        Ok(doc) => ([(header::CONTENT_TYPE, "application/json")], doc).into_response(),
        // Serializing our own types cannot fail; if it ever does, this is a bug, so it is one
        // logged 500, not a plain-text panic.
        Err(err) => {
            tracing::error!(
                request_id = %id.0,
                code = "internal",
                error = %err,
                "serializing the OpenAPI document failed"
            );
            ApiError::new(ErrorCode::Internal).into_response(&id.0)
        }
    }
}
