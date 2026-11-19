//! The authenticated caller: a session cookie that still points at a live session. The
//! rejection is the standard 401 `unauthenticated` body, with `WWW-Authenticate` (added by
//! `api_error`).

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;
use uuid::Uuid;

use crate::RequestId;
use crate::api_error::ApiError;
use crate::cookies;
use crate::error_codes::ErrorCode;
use crate::state::AppState;
use crate::store_errors;
use crate::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentStudent {
    pub student_id: Uuid,
    pub session_id: Uuid,
}

impl FromRequestParts<AppState> for CurrentStudent {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let id = parts
            .extensions
            .get::<RequestId>()
            .map_or_else(|| RequestId(Uuid::now_v7().to_string()), Clone::clone);
        let unauthorized = || ApiError::new(ErrorCode::Unauthenticated).into_response(&id.0);
        let Some(token) = cookies::token_from_headers(&parts.headers) else {
            return Err(unauthorized());
        };
        let now = state.clock.now();
        match state
            .accounts
            .find_session(&tokens::hash_token(&token), now)
            .await
        {
            Ok(Some(record)) => Ok(Self {
                student_id: record.student_id,
                session_id: record.session_id,
            }),
            Ok(None) => Err(unauthorized()),
            Err(err) => Err(store_errors::into_response(err, &id.0)),
        }
    }
}
