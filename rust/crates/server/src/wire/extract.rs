//! The one JSON extractor. Every axum rejection becomes the standard error body: a wrong or
//! missing content type is 415 `unsupported_media_type`, anything else (unparseable JSON, a wrong
//! type, a missing or unknown field) is 400 `invalid_request`.

use axum::Json;
use axum::extract::FromRequest;
use axum::http::header;
use axum::response::Response;
use serde::de::DeserializeOwned;

use crate::RequestId;
use crate::api_error::ApiError;
use crate::error_codes::ErrorCode;

pub struct ApiJson<T>(pub T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Response;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Response> {
        let request_id = req
            .extensions()
            .get::<RequestId>()
            .map_or_else(|| RequestId(uuid::Uuid::now_v7().to_string()), Clone::clone);
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| {
                v.split(';')
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase()
            });
        if content_type.as_deref() != Some("application/json") {
            return Err(ApiError::new(ErrorCode::UnsupportedMediaType).into_response(&request_id.0));
        }
        // From here every rejection is about the body's shape or content, not its transport.
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(ApiJson(value)),
            Err(_rejection) => {
                Err(ApiError::new(ErrorCode::InvalidRequest).into_response(&request_id.0))
            }
        }
    }
}
