//! `GET/PATCH /v1/me`: the signed-in student's own profile.

use axum::Extension;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use belajar_core::grade::Grade;
use belajar_core::theme::ThemePreference;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody, FieldError};
use crate::current_student::CurrentStudent;
use crate::routes::auth::StudentDto;
use crate::state::AppState;
use crate::store_errors;
use crate::wire::extract::ApiJson;
use crate::wire::fields::invalid_format;

/// The colour theme the student chose: `system` follows the device's light/dark setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreferenceDto {
    System,
    Light,
    Dark,
}

impl From<ThemePreference> for ThemePreferenceDto {
    fn from(theme: ThemePreference) -> Self {
        match theme {
            ThemePreference::System => Self::System,
            ThemePreference::Light => Self::Light,
            ThemePreference::Dark => Self::Dark,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatchMeRequest {
    /// A `nickname` that is only whitespace is invalid, so `null` is the only "no change" form.
    pub nickname: Option<String>,
    pub grade: Option<u8>,
    /// Read as text so a value outside the three gets a field error, not a bare 400.
    #[schema(value_type = Option<ThemePreferenceDto>)]
    pub theme: Option<String>,
}

/// The signed-in student.
#[utoipa::path(
    get,
    path = "/v1/me",
    responses(
        (status = 200, body = StudentDto),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn get_me(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
) -> Response {
    match state.accounts.get_student(student.student_id).await {
        Ok(account) => (StatusCode::OK, Json(StudentDto::from(account))).into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// Updates the nickname, the grade and/or the theme. `username` and friends are not editable here.
#[utoipa::path(
    patch,
    path = "/v1/me",
    request_body = PatchMeRequest,
    responses(
        (status = 200, description = "The updated student", body = StudentDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn patch_me(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    ApiJson(req): ApiJson<PatchMeRequest>,
) -> Response {
    let mut errors = Vec::new();
    if let Some(nickname) = &req.nickname {
        let len = nickname.trim().chars().count();
        if !(1..=40).contains(&len) {
            errors.push(FieldError {
                field: "/nickname".into(),
                code: if len < 1 { "too_short" } else { "too_long" },
                message: if len < 1 {
                    "Must be at least 1 characters.".into()
                } else {
                    "Must be at most 40 characters.".into()
                },
            });
        }
    }
    if let Some(grade) = req.grade
        && Grade::new(grade).is_err()
    {
        errors.push(FieldError {
            field: "/grade".into(),
            code: "out_of_range",
            message: format!("Must be between {} and {}.", Grade::MIN, Grade::MAX),
        });
    }
    let theme = match req.theme.as_deref().map(ThemePreference::parse) {
        Some(None) => {
            errors.push(invalid_format("/theme", "one of system, light, dark"));
            None
        }
        Some(Some(theme)) => Some(theme),
        None => None,
    };
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    match state
        .accounts
        .update_profile(
            student.student_id,
            req.nickname.as_deref().map(str::trim),
            req.grade.and_then(|g| Grade::new(g).ok()),
            theme,
        )
        .await
    {
        Ok(account) => (StatusCode::OK, Json(StudentDto::from(account))).into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::expect_used, reason = "tests")]
#[allow(clippy::panic, reason = "tests")]
mod tests {
    use serde_json::json;

    use super::PatchMeRequest;

    #[test]
    fn the_patch_shape_is_camel_case_and_rejects_unknown_fields() {
        // A `username` key fails deserialization (deny_unknown_fields), which the extractor
        // answers with 400 invalid_request; the acceptance test covers it end to end. Here we
        // pin the shape itself: the two editable fields deserialize, `null` means "no change".
        let parsed: PatchMeRequest = serde_json::from_value(json!({
            "nickname": "Budi Kecil", "grade": 12
        }))
        .expect("nickname and grade are the only editable fields");
        assert_eq!(parsed.nickname.as_deref(), Some("Budi Kecil"));
        assert_eq!(parsed.grade, Some(12));
        assert_eq!(parsed.theme, None, "an absent theme is no change");

        let parsed: PatchMeRequest =
            serde_json::from_value(json!({ "theme": "dark" })).expect("the theme is editable");
        assert_eq!(parsed.theme.as_deref(), Some("dark"));

        let parsed: PatchMeRequest = serde_json::from_value(json!({
            "nickname": null, "grade": null
        }))
        .expect("null means no change");
        assert_eq!(parsed.nickname, None);
        assert_eq!(parsed.grade, None);

        let body = json!({ "username": "lain" });
        assert!(
            serde_json::from_value::<PatchMeRequest>(body).is_err(),
            "username is not editable through PATCH /v1/me"
        );
    }
}
