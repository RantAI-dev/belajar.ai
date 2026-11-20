//! "Hubungkan dengan Google" from Akun: a signed-in student with a username and password links a
//! Google account, so they can sign in either way. The same flow as a sign-in (PKCE, state, nonce,
//! a single-use attempt bound to this browser); the attempt also records, at start, the student
//! and the session it is for. The shared callback in `google_auth` finishes it here.

use axum::Extension;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use belajar_ports::google::{GoogleLinkResult, LinkTarget};
use belajar_ports::store::StoreError;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody};
use crate::cookies;
use crate::current_student::CurrentStudent;
use crate::error_codes::ErrorCode;
use crate::google::id_token::IdClaims;
use crate::routes::google_auth::{Outcome, begin, disabled};
use crate::routes::google_signup::nickname_suggestion;
use crate::state::AppState;
use crate::store_errors;
use crate::tokens;

/// Where a link attempt's callback sends the browser, with `?google=<outcome>`.
pub(crate) const AKUN: &str = "/akun";

/// Starts linking Google to the signed-in student: a browser navigation from Akun.
#[utoipa::path(
    get,
    path = "/v1/auth/google/link/start",
    responses(
        (status = 303, description = "To Google's consent page, with a short-lived httpOnly cookie that binds the attempt to this browser and this session"),
        (status = 401, body = ErrorBody),
        (status = 404, description = "`google_disabled`: Google sign-in is not configured", body = ErrorBody),
        (status = 409, description = "`google_already_linked`: this student already has a Google account", body = ErrorBody),
    )
)]
pub(crate) async fn link_start(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
) -> Response {
    let Some(google) = state.google.clone() else {
        return disabled(&id);
    };
    match state.accounts.get_student(student.student_id).await {
        Ok(account) if account.is_google_linked => {
            return ApiError::new(ErrorCode::GoogleAlreadyLinked).into_response(&id.0);
        }
        Ok(_) => {}
        Err(err) => return store_errors::into_response(err, &id.0),
    }
    let target = LinkTarget {
        student_id: student.student_id,
        session_id: student.session_id,
    };
    begin(&state, &id, &google, Some(target)).await
}

/// Whether the callback arrives in the session that started the link. A logout, another login or
/// a session cookie from someone else refuses it, so a link never lands on another student.
pub(crate) async fn same_session(
    state: &AppState,
    headers: &HeaderMap,
    target: LinkTarget,
) -> Result<bool, StoreError> {
    let Some(token) = cookies::token_from_headers(headers) else {
        return Ok(false);
    };
    let session = state
        .accounts
        .find_session(&tokens::hash_token(&token), state.clock.now())
        .await?;
    Ok(session
        .is_some_and(|s| s.session_id == target.session_id && s.student_id == target.student_id))
}

/// Links the verified Google account to the attempt's student. `Ok(location)` on success, or the
/// outcome and log reason of a refusal.
pub(crate) async fn finish(
    state: &AppState,
    target: LinkTarget,
    claims: &IdClaims,
) -> Result<Result<&'static str, (Outcome, &'static str)>, StoreError> {
    let display_name = nickname_suggestion(claims.name.as_deref());
    let result = state
        .google_sign_in
        .link_to_student(target.student_id, &claims.sub, &display_name)
        .await?;
    Ok(match result {
        GoogleLinkResult::Linked | GoogleLinkResult::AlreadyLinked => Ok("/akun?google=linked"),
        GoogleLinkResult::LinkedToAnother => Err((Outcome::Conflict, "linked_to_another_student")),
        GoogleLinkResult::StudentHasAnother => Err((Outcome::Failed, "student_linked_meanwhile")),
    })
}
