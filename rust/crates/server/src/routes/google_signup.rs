//! "Masuk dengan Google", the onboarding half: a first Google sign-in has no student yet, so the
//! web app asks for a nickname and a grade (the same picker as the register form) and then
//! creates the student here. The pending Google account is found only by this browser's cookie.

use axum::Extension;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{AppendHeaders, IntoResponse, Response};
use belajar_core::grade::Grade;
use belajar_ports::google::NewGoogleStudent;
use belajar_ports::store::StoreError;
use rand::Rng;
use rand::distr::Alphanumeric;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody};
use crate::cookies;
use crate::error_codes::ErrorCode;
use crate::google::cookies::{self as google_cookies, SIGNUP_COOKIE};
use crate::routes::auth::{StudentDto, start_session, validate_grade, validate_nickname};
use crate::routes::google_auth::disabled;
use crate::state::AppState;
use crate::store_errors;
use crate::tokens;

/// The longest nickname the profile accepts (see `validate_nickname`).
const NICKNAME_MAX: usize = 40;

/// A verified Google account waiting for its grade.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PendingGoogleSignupDto {
    /// The Google profile name, cut to the nickname limit; empty when Google sent none.
    pub suggested_nickname: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteGoogleSignupRequest {
    pub nickname: String,
    pub grade: u8,
}

/// The profile name, trimmed and cut to what a nickname may hold.
#[must_use]
pub fn nickname_suggestion(name: Option<&str>) -> String {
    name.unwrap_or_default()
        .trim()
        .chars()
        .take(NICKNAME_MAX)
        .collect()
}

/// `google.` and 12 random letters or digits: a Google student never types it, and it fits the
/// username rules (3–32 of `a-z 0-9 . _`).
fn generated_username() -> String {
    let suffix: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(12)
        .map(|c| char::from(c).to_ascii_lowercase())
        .collect();
    format!("google.{suffix}")
}

fn signup_token(headers: &HeaderMap) -> Option<Vec<u8>> {
    cookies::value_from_headers(headers, SIGNUP_COOKIE).map(|token| tokens::hash_token(&token))
}

fn not_found(id: &RequestId) -> Response {
    ApiError::new(ErrorCode::NotFound).into_response(&id.0)
}

/// The Google account this browser is signing up with.
#[utoipa::path(
    get,
    path = "/v1/auth/google/signup",
    responses(
        (status = 200, body = PendingGoogleSignupDto),
        (status = 404, description = "No pending Google sign-up for this browser, or it expired (10 minutes); `google_disabled` when Google sign-in is off", body = ErrorBody),
    )
)]
pub(crate) async fn get_pending(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    headers: HeaderMap,
) -> Response {
    if state.google.is_none() {
        return disabled(&id);
    }
    let Some(token_hash) = signup_token(&headers) else {
        return not_found(&id);
    };
    match state
        .google_sign_in
        .find_pending_signup(&token_hash, state.clock.now())
        .await
    {
        Ok(Some(pending)) => Json(PendingGoogleSignupDto {
            suggested_nickname: pending.display_name,
        })
        .into_response(),
        Ok(None) => not_found(&id),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// Creates the student for this browser's pending Google account and signs them in.
#[utoipa::path(
    post,
    path = "/v1/auth/google/signup",
    request_body = CompleteGoogleSignupRequest,
    responses(
        (status = 201, description = "The new student, signed in with a session cookie", body = StudentDto),
        (status = 400, body = ErrorBody),
        (status = 404, description = "No pending Google sign-up for this browser, or it expired; `google_disabled` when Google sign-in is off", body = ErrorBody),
        (status = 409, description = "This Google account was linked to a student meanwhile (another tab finished first)", body = ErrorBody),
    )
)]
pub(crate) async fn complete(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    headers: HeaderMap,
    crate::wire::extract::ApiJson(req): crate::wire::extract::ApiJson<CompleteGoogleSignupRequest>,
) -> Response {
    if state.google.is_none() {
        return disabled(&id);
    }
    let mut errors = Vec::new();
    validate_nickname(&mut errors, &req.nickname);
    validate_grade(&mut errors, req.grade);
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    let (Ok(grade), Some(token_hash)) = (Grade::new(req.grade), signup_token(&headers)) else {
        return not_found(&id);
    };
    let username = generated_username();
    let student = NewGoogleStudent {
        username: &username,
        nickname: req.nickname.trim(),
        grade,
    };
    let account = match state
        .google_sign_in
        .complete_signup(&token_hash, student, state.clock.now())
        .await
    {
        Ok(account) => account,
        Err(StoreError::Conflict) => {
            return ApiError::new(ErrorCode::Conflict)
                .message("This Google account is already linked to a student. Sign in with Google again.")
                .into_response(&id.0);
        }
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let session = match start_session(&state, account.id).await {
        Ok(cookie) => cookie,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let secure = state.config.cookie_secure;
    (
        StatusCode::CREATED,
        AppendHeaders([
            (header::SET_COOKIE, session),
            (
                header::SET_COOKIE,
                google_cookies::cleared(SIGNUP_COOKIE, secure),
            ),
        ]),
        Json(StudentDto::from(account)),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_suggestion_is_trimmed_and_cut_to_the_nickname_limit() {
        assert_eq!(nickname_suggestion(Some("  Budi Santoso ")), "Budi Santoso");
        assert_eq!(nickname_suggestion(None), "");
        assert_eq!(
            nickname_suggestion(Some(&"é".repeat(50))).chars().count(),
            40
        );
    }

    #[test]
    fn a_generated_username_passes_the_username_rules() {
        let name = generated_username();
        assert_eq!(name.len(), 19);
        assert!(name.starts_with("google."));
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.')
        );
        assert_ne!(name, generated_username());
    }
}
