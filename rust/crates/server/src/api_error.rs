//! The one place that turns an error into a response. Copy it beside the generated
//! `error_codes.rs` (see `.agents/error-codes.tsv`) and have every handler return through it.
//! Written for axum; it needs `axum`, `serde` and `serde_json`.

use axum::Json;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

use crate::error_codes::ErrorCode;

/// One bad field, named by a JSON Pointer. The `code` vocabulary is the one the plan fixes:
/// `required`, `too_short`, `too_long`, `invalid_format`, `out_of_range`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FieldError {
    /// A JSON Pointer into the body (`/items/0/quantity`), or the name of a query or path parameter.
    pub field: String,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: ErrorCode,
    message: Option<String>,
    errors: Vec<FieldError>,
    retry_after_seconds: Option<u64>,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "<[FieldError]>::is_empty")]
    pub errors: Vec<FieldError>,
}

impl ApiError {
    #[must_use]
    pub const fn new(code: ErrorCode) -> Self {
        Self {
            code,
            message: None,
            errors: Vec::new(),
            retry_after_seconds: None,
        }
    }

    /// A more specific sentence than the registry's. Never pass `err.to_string()` here: log it.
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Every bad field, not only the first.
    #[must_use]
    pub fn invalid(errors: Vec<FieldError>) -> Self {
        Self {
            errors,
            ..Self::new(ErrorCode::InvalidRequest)
        }
    }

    #[must_use]
    pub const fn retry_after(mut self, seconds: u64) -> Self {
        self.retry_after_seconds = Some(seconds);
        self
    }

    fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.code.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// The standard body. Also the `data` of an SSE `error` event, so a failure after a stream
    /// started carries the same `code` and `requestId` as a JSON error.
    #[must_use]
    pub fn body(&self, request_id: &str) -> ErrorBody {
        // A 5xx never carries a caller-supplied sentence: the cause belongs in the log.
        let message = match &self.message {
            Some(text) if !self.status().is_server_error() => text.as_str(),
            _ => self.code.message(),
        };
        ErrorBody {
            error: ErrorDetail {
                code: self.code.as_str(),
                message: message.to_owned(),
                request_id: request_id.to_owned(),
                errors: self.errors.clone(),
            },
        }
    }

    #[must_use]
    pub fn into_response(self, request_id: &str) -> Response {
        let status = self.status();
        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(request_id) {
            headers.insert("x-request-id", value);
        }
        if status == StatusCode::UNAUTHORIZED {
            headers.insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        if let Some(seconds) = self.retry_after_seconds {
            headers.insert(header::RETRY_AFTER, HeaderValue::from(seconds));
        }
        (status, headers, Json(self.body(request_id))).into_response()
    }
}

/// The caller's `X-Request-Id` when it is safe to echo, otherwise `None`: make a UUID then.
#[must_use]
pub fn sent_request_id(headers: &HeaderMap) -> Option<&str> {
    let sent = headers.get("x-request-id")?.to_str().ok()?;
    let safe = (1..=128).contains(&sent.len())
        && sent
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
    safe.then_some(sent)
}
