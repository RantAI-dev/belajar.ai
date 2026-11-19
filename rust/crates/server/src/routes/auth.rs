//! The auth routes: register, login, logout, password change. Each handler only decodes,
//! validates, calls the store and maps the error.

use axum::Extension;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use belajar_core::grade::Grade;
use belajar_ports::accounts::StudentAccount;
use belajar_ports::store::StoreError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody, FieldError};
use crate::cookies;
use crate::current_student::CurrentStudent;
use crate::error_codes::ErrorCode;
use crate::passwords;
use crate::state::AppState;
use crate::store_errors;
use crate::tokens;
use crate::wire::extract::ApiJson;
use crate::wire::instant;

/// A whole student, as every auth and `/v1/me` response shows them. No password material, ever.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StudentDto {
    pub id: Uuid,
    pub username: String,
    pub nickname: String,
    pub grade: u8,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl From<StudentAccount> for StudentDto {
    fn from(account: StudentAccount) -> Self {
        Self {
            id: account.id,
            username: account.username,
            nickname: account.nickname,
            grade: account.grade.get(),
            created_at: account.created_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub nickname: String,
    pub grade: u8,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// `POST /v1/auth/logout` takes an empty object, so the extractor can tell `{}` from nothing.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LogoutRequest {}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
}

// The field-error vocabulary: `required`, `too_short`, `too_long`, `invalid_format`, `out_of_range`.
fn required(field: &str) -> FieldError {
    FieldError {
        field: field.into(),
        code: "required",
        message: "This field is required.".into(),
    }
}

fn too_short(field: &str, min: usize) -> FieldError {
    FieldError {
        field: field.into(),
        code: "too_short",
        message: format!("Must be at least {min} characters."),
    }
}

fn too_long(field: &str, max: usize) -> FieldError {
    FieldError {
        field: field.into(),
        code: "too_long",
        message: format!("Must be at most {max} characters."),
    }
}

fn invalid_format(field: &str, what: &str) -> FieldError {
    FieldError {
        field: field.into(),
        code: "invalid_format",
        message: format!("Must be {what}."),
    }
}

fn out_of_range(field: &str, min: u8, max: u8) -> FieldError {
    FieldError {
        field: field.into(),
        code: "out_of_range",
        message: format!("Must be between {min} and {max}."),
    }
}

/// Every bad field, not only the first.
fn check_length(errors: &mut Vec<FieldError>, field: &str, len: usize, min: usize, max: usize) {
    if len < min {
        errors.push(too_short(field, min));
    } else if len > max {
        errors.push(too_long(field, max));
    }
}

/// 3–32 characters of letters, digits, `_` and `.`.
fn validate_username(errors: &mut Vec<FieldError>, username: &str) {
    check_length(errors, "/username", username.chars().count(), 3, 32);
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        errors.push(invalid_format(
            "/username",
            "3–32 letters, digits, underscores or dots",
        ));
    }
}

fn validate_password(errors: &mut Vec<FieldError>, password: &str) {
    check_length(errors, "/password", password.chars().count(), 8, 128);
}

fn validate_nickname(errors: &mut Vec<FieldError>, nickname: &str) {
    check_length(errors, "/nickname", nickname.trim().chars().count(), 1, 40);
}

fn validate_grade(errors: &mut Vec<FieldError>, grade: u8) {
    if Grade::new(grade).is_err() {
        errors.push(out_of_range("/grade", Grade::MIN, Grade::MAX));
    }
}

fn validate_register(req: &RegisterRequest) -> Vec<FieldError> {
    let mut errors = Vec::new();
    validate_username(&mut errors, &req.username);
    validate_password(&mut errors, &req.password);
    validate_nickname(&mut errors, &req.nickname);
    validate_grade(&mut errors, req.grade);
    errors
}

fn internal(id: &RequestId, cause: &str) -> Response {
    tracing::error!(request_id = %id.0, code = "internal", error = %cause, "auth failed");
    ApiError::new(ErrorCode::Internal).into_response(&id.0)
}

fn unauthenticated(id: &RequestId) -> Response {
    ApiError::new(ErrorCode::Unauthenticated).into_response(&id.0)
}

fn password_cause(err: &passwords::PasswordError) -> String {
    match err {
        passwords::PasswordError::Internal(text) => text.clone(),
    }
}

/// A fresh session for `student_id`, as a `Set-Cookie` value. The caller maps the store error,
/// so this stays one small `Result`.
async fn start_session(state: &AppState, student_id: Uuid) -> Result<String, StoreError> {
    let token = tokens::new_token();
    let expires_at =
        state.clock.now() + chrono::Duration::seconds(cookies::SESSION_MAX_AGE_SECONDS as i64);
    state
        .accounts
        .create_session(student_id, &tokens::hash_token(&token), expires_at)
        .await?;
    Ok(cookies::session_cookie(&token, state.config.cookie_secure))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/password", post(change_password))
}

/// Creates a student and signs them in.
#[utoipa::path(
    post,
    path = "/v1/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "The student, signed in with a session cookie", body = StudentDto),
        (status = 400, body = ErrorBody),
        (status = 409, description = "The username is already taken", body = ErrorBody),
    )
)]
pub(crate) async fn register(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    ApiJson(req): ApiJson<RegisterRequest>,
) -> Response {
    let errors = validate_register(&req);
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    // Validation has already bounded the grade; this arm only keeps the type honest.
    let Ok(grade) = Grade::new(req.grade) else {
        return internal(&id, "grade out of range after validation");
    };
    let password_hash = match passwords::hash(req.password.clone()).await {
        Ok(hash) => hash,
        Err(err) => return internal(&id, &password_cause(&err)),
    };
    // Usernames are stored lowercased: "Budi" and "budi" are one account.
    let username = req.username.to_lowercase();
    let account = match state
        .accounts
        .create_student(&username, &password_hash, req.nickname.trim(), grade)
        .await
    {
        Ok(account) => account,
        Err(StoreError::Conflict) => {
            return ApiError::new(ErrorCode::Conflict)
                .message("That username is already taken.")
                .into_response(&id.0);
        }
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let cookie = match start_session(&state, account.id).await {
        Ok(cookie) => cookie,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    (
        StatusCode::CREATED,
        [(header::SET_COOKIE, cookie)],
        Json(StudentDto::from(account)),
    )
        .into_response()
}

/// Signs a student in. Unknown username and wrong password answer the same body.
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "The student, with a fresh session cookie", body = StudentDto),
        (status = 400, body = ErrorBody),
        (status = 401, description = "Unknown username or wrong password", body = ErrorBody),
        (status = 429, description = "Too many failed logins; see Retry-After", body = ErrorBody),
    )
)]
pub(crate) async fn login(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    ApiJson(req): ApiJson<LoginRequest>,
) -> Response {
    let mut errors = Vec::new();
    if req.username.is_empty() {
        errors.push(required("/username"));
    }
    if req.password.is_empty() {
        errors.push(required("/password"));
    }
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    let username = req.username.to_lowercase();
    // The lock is checked before any password work, so a locked username stays locked even with
    // the right password.
    if let Some(seconds) = state.limiter.locked_remaining(&username) {
        return ApiError::new(ErrorCode::RateLimited)
            .retry_after(seconds)
            .into_response(&id.0);
    }
    let record = match state.accounts.find_login_by_username(&username).await {
        Ok(Some(record)) => record,
        Ok(None) => {
            state.limiter.record_failure(&username);
            return unauthenticated(&id);
        }
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let ok = match passwords::verify(record.password_hash.clone(), req.password.clone()).await {
        Ok(ok) => ok,
        Err(err) => return internal(&id, &password_cause(&err)),
    };
    if !ok {
        state.limiter.record_failure(&username);
        return unauthenticated(&id);
    }
    // The right password clears the username's failures, whatever happened before.
    state.limiter.clear(&username);
    let account = match state.accounts.get_student(record.user_id).await {
        Ok(account) => account,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let cookie = match start_session(&state, account.id).await {
        Ok(cookie) => cookie,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie)],
        Json(StudentDto::from(account)),
    )
        .into_response()
}

/// Ends the caller's session.
#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 204, description = "The session is gone; the cookie is cleared"),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn logout(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    ApiJson(_req): ApiJson<LogoutRequest>,
) -> Response {
    match state
        .accounts
        .delete_session(student.student_id, student.session_id)
        .await
    {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            [(
                header::SET_COOKIE,
                cookies::cleared_cookie(state.config.cookie_secure),
            )],
        )
            .into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// Changes the password: the current session survives, every other one ends.
#[utoipa::path(
    post,
    path = "/v1/auth/password",
    request_body = PasswordChangeRequest,
    responses(
        (status = 204, description = "The password is changed and other sessions are ended"),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
        (status = 403, description = "The current password is wrong", body = ErrorBody),
    )
)]
pub(crate) async fn change_password(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    ApiJson(req): ApiJson<PasswordChangeRequest>,
) -> Response {
    let mut errors = Vec::new();
    if req.current_password.is_empty() {
        errors.push(required("/currentPassword"));
    }
    if req.new_password.chars().count() < 8 || req.new_password.chars().count() > 128 {
        errors.push(too_short_or_long_password(&req.new_password));
    }
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    let stored_hash = match state.accounts.password_hash(student.student_id).await {
        Ok(hash) => hash,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let ok = match passwords::verify(stored_hash, req.current_password).await {
        Ok(ok) => ok,
        Err(err) => return internal(&id, &password_cause(&err)),
    };
    if !ok {
        return ApiError::new(ErrorCode::PermissionDenied).into_response(&id.0);
    }
    let new_hash = match passwords::hash(req.new_password).await {
        Ok(hash) => hash,
        Err(err) => return internal(&id, &password_cause(&err)),
    };
    if let Err(err) = state
        .accounts
        .set_password_hash(student.student_id, &new_hash)
        .await
    {
        return store_errors::into_response(err, &id.0);
    }
    if let Err(err) = state
        .accounts
        .delete_other_sessions(student.student_id, student.session_id)
        .await
    {
        return store_errors::into_response(err, &id.0);
    }
    StatusCode::NO_CONTENT.into_response()
}

fn too_short_or_long_password(password: &str) -> FieldError {
    let len = password.chars().count();
    if len < 8 {
        too_short("/newPassword", 8)
    } else {
        too_long("/newPassword", 128)
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use super::*;

    #[test]
    fn register_collects_every_bad_field() {
        let req = RegisterRequest {
            username: "a".into(),
            password: "short".into(),
            nickname: "   ".into(),
            grade: 13,
        };
        let errors = validate_register(&req);
        let mut fields: Vec<&str> = errors.iter().map(|e| e.field.as_str()).collect();
        fields.sort_unstable();
        assert_eq!(
            fields,
            vec!["/grade", "/nickname", "/password", "/username"]
        );
    }

    #[test]
    fn a_valid_body_has_no_errors() {
        let req = RegisterRequest {
            username: "Budi.S_7".into(),
            password: "rahasia-123".into(),
            nickname: " Budi ".into(),
            grade: 7,
        };
        assert!(validate_register(&req).is_empty());
    }

    #[test]
    fn username_length_counts_characters_and_charset_is_limited() {
        let mut errors = Vec::new();
        validate_username(&mut errors, "ab"); // too short
        assert_eq!(errors.len(), 1);
        let mut errors = Vec::new();
        validate_username(&mut errors, &"a".repeat(33));
        assert_eq!(errors.len(), 1, "too long only; the charset is fine");
        let mut errors = Vec::new();
        validate_username(&mut errors, "budi and co");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "invalid_format");
    }

    #[test]
    fn nickname_length_is_measured_after_trimming() {
        let mut errors = Vec::new();
        validate_nickname(&mut errors, "   ");
        assert_eq!(errors[0].code, "too_short");
        let mut errors = Vec::new();
        validate_nickname(&mut errors, &"x".repeat(41));
        assert_eq!(errors[0].code, "too_long");
        let mut errors = Vec::new();
        validate_nickname(&mut errors, "  Budi  ");
        assert!(errors.is_empty());
    }
}
