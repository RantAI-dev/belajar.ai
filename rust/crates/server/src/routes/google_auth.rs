//! "Masuk dengan Google", the redirect half: whether it is on, the start redirect to Google, and
//! the callback. The callback always answers with a redirect into the web app; a failure says
//! only which of a few outcomes it was (`/masuk?google=failed`), never Google's text or a token.
//! A link attempt started from Akun (`google_link`) comes back through the same callback and
//! lands on `/akun?google=<outcome>` instead.

use axum::Extension;
use axum::Json;
use axum::Router;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{AppendHeaders, IntoResponse, Response};
use axum::routing::get;
use belajar_ports::google::{GoogleAttempt, LinkTarget, PendingGoogleSignup};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody};
use crate::cookies;
use crate::error_codes::ErrorCode;
use crate::google::cookies::{self as google_cookies, FLOW_COOKIE, MAX_AGE_SECONDS, SIGNUP_COOKIE};
use crate::google::flow::{constant_time_eq, pkce_challenge, random_value};
use crate::google::id_token::IdClaims;
use crate::google::{GoogleClient, GoogleError};
use crate::routes::auth::start_session;
use crate::routes::{google_link, google_signup};
use crate::state::AppState;
use crate::store_errors;
use crate::tokens;

/// Whether the web app shows the "Masuk dengan Google" button.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSignInDto {
    pub is_enabled: bool,
}

/// What Google appends to the callback. Unknown parameters (`scope`, `authuser`, ...) are ignored.
#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

/// Where a failed callback sends the browser: `/masuk?google=<value>`, or `/akun?google=<value>`
/// for a link attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// The student pressed "Cancel" on Google's page.
    Cancelled,
    /// Google could not be reached or answered 5xx.
    Unavailable,
    /// Anything else: a stale or forged callback, a refused code, a token that failed a check.
    Failed,
    /// A link attempt whose Google account belongs to another student.
    Conflict,
}

impl Outcome {
    pub(crate) const fn query_value(self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::Unavailable => "unavailable",
            Self::Failed => "failed",
            Self::Conflict => "conflict",
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/google", get(google_status))
        .route("/auth/google/start", get(google_start))
        .route("/auth/google/callback", get(google_callback))
        .route("/auth/google/link/start", get(google_link::link_start))
        .route(
            "/auth/google/signup",
            get(google_signup::get_pending).post(google_signup::complete),
        )
}

pub(crate) fn disabled(id: &RequestId) -> Response {
    ApiError::new(ErrorCode::GoogleDisabled).into_response(&id.0)
}

/// Whether "Masuk dengan Google" is configured on this server.
#[utoipa::path(
    get,
    path = "/v1/auth/google",
    responses((status = 200, body = GoogleSignInDto))
)]
pub(crate) async fn google_status(state: State<AppState>) -> Response {
    Json(GoogleSignInDto {
        is_enabled: state.google.is_some(),
    })
    .into_response()
}

/// Starts a Google sign-in: a browser navigation, answered with a redirect to Google.
#[utoipa::path(
    get,
    path = "/v1/auth/google/start",
    responses(
        (status = 303, description = "To Google's consent page, with a short-lived httpOnly cookie that binds the attempt to this browser"),
        (status = 404, description = "`google_disabled`: Google sign-in is not configured", body = ErrorBody),
    )
)]
pub(crate) async fn google_start(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
) -> Response {
    let Some(google) = state.google.clone() else {
        return disabled(&id);
    };
    begin(&state, &id, &google, None).await
}

/// Saves a new attempt (a sign-in, or a link for `link`) and sends the browser to Google.
pub(crate) async fn begin(
    state: &AppState,
    id: &RequestId,
    google: &GoogleClient,
    link: Option<LinkTarget>,
) -> Response {
    let browser_token = tokens::new_token();
    let attempt = GoogleAttempt {
        state: random_value(),
        nonce: random_value(),
        code_verifier: random_value(),
        link,
    };
    let url = match google.authorization_url(
        &attempt.state,
        &attempt.nonce,
        &pkce_challenge(&attempt.code_verifier),
    ) {
        Ok(url) => url,
        Err(err) => {
            tracing::error!(request_id = %id.0, code = "internal", error = %err, "google authorization url");
            return ApiError::new(ErrorCode::Internal).into_response(&id.0);
        }
    };
    let expires_at = state.clock.now() + chrono::Duration::seconds(MAX_AGE_SECONDS);
    if let Err(err) = state
        .google_sign_in
        .save_attempt(&tokens::hash_token(&browser_token), &attempt, expires_at)
        .await
    {
        return store_errors::into_response(err, &id.0);
    }
    redirect(
        &url,
        &[google_cookies::set(
            FLOW_COOKIE,
            &browser_token,
            state.config.cookie_secure,
        )],
    )
}

/// Google sends the browser back here. Signs an existing student in, or parks a new Google
/// account until the student picks a grade.
#[utoipa::path(
    get,
    path = "/v1/auth/google/callback",
    params(
        ("code" = Option<String>, Query, description = "The authorization code"),
        ("state" = Option<String>, Query, description = "Must equal the state this browser's attempt sent"),
        ("error" = Option<String>, Query, description = "Set by Google when the student cancels"),
    ),
    responses(
        (status = 303, description = "To `/` (signed in, with the session cookie), `/daftar/google` (new account: pick a grade), or `/masuk?google=cancelled|unavailable|failed`. A link attempt goes to `/akun?google=linked|conflict|cancelled|unavailable|failed`"),
        (status = 404, description = "`google_disabled`: Google sign-in is not configured", body = ErrorBody),
    )
)]
pub(crate) async fn google_callback(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    headers: HeaderMap,
    query: Result<Query<CallbackQuery>, QueryRejection>,
) -> Response {
    let Some(google) = state.google.clone() else {
        return disabled(&id);
    };
    let secure = state.config.cookie_secure;
    let clear_flow = google_cookies::cleared(FLOW_COOKIE, secure);
    // Until the attempt is read, nothing says it was a link: a failure then goes to the login page.
    let fail_to = |page: &str, outcome: Outcome, reason: &str| {
        // A 4xx-like outcome: info, with a fixed reason and nothing from Google or the browser.
        tracing::info!(request_id = %id.0, reason, outcome = outcome.query_value(), "google sign-in refused");
        redirect(
            &format!("{page}?google={}", outcome.query_value()),
            std::slice::from_ref(&clear_flow),
        )
    };
    let Ok(Query(query)) = query else {
        return fail_to("/masuk", Outcome::Failed, "callback_query_malformed");
    };
    let Some(browser_token) = cookies::value_from_headers(&headers, FLOW_COOKIE) else {
        return fail_to("/masuk", Outcome::Failed, "flow_cookie_missing");
    };
    let attempt = match state
        .google_sign_in
        .take_attempt(&tokens::hash_token(&browser_token), state.clock.now())
        .await
    {
        Ok(Some(attempt)) => attempt,
        Ok(None) => return fail_to("/masuk", Outcome::Failed, "attempt_unknown_or_expired"),
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let page = if attempt.link.is_some() {
        google_link::AKUN
    } else {
        "/masuk"
    };
    let fail = |outcome: Outcome, reason: &str| fail_to(page, outcome, reason);
    let state_ok = query
        .state
        .as_deref()
        .is_some_and(|given| constant_time_eq(given, &attempt.state));
    if !state_ok {
        return fail(Outcome::Failed, "state_mismatch");
    }
    if let Some(target) = attempt.link {
        match google_link::same_session(&state, &headers, target).await {
            Ok(true) => {}
            Ok(false) => return fail(Outcome::Failed, "link_session_changed"),
            Err(err) => return store_errors::into_response(err, &id.0),
        }
    }
    if let Some(error) = query.error.as_deref() {
        return if error == "access_denied" {
            fail(Outcome::Cancelled, "cancelled_by_student")
        } else {
            fail(Outcome::Failed, "authorization_error")
        };
    }
    let Some(code) = query.code.as_deref() else {
        return fail(Outcome::Failed, "code_missing");
    };
    let claims = match verified_claims(&google, &state, code, &attempt).await {
        Ok(claims) => claims,
        Err(GoogleError::Unavailable(cause)) => {
            tracing::warn!(request_id = %id.0, error = %cause, "google unavailable");
            return fail(Outcome::Unavailable, "google_unavailable");
        }
        Err(GoogleError::Rejected(cause)) => {
            tracing::info!(request_id = %id.0, error = %cause, "google refused the code");
            return fail(Outcome::Failed, "code_rejected");
        }
        Err(GoogleError::BadIdToken(err)) => return fail(Outcome::Failed, err.reason()),
    };
    if let Some(target) = attempt.link {
        return match google_link::finish(&state, target, &claims).await {
            Ok(Ok(location)) => redirect(location, &[clear_flow]),
            Ok(Err((outcome, reason))) => fail(outcome, reason),
            Err(err) => store_errors::into_response(err, &id.0),
        };
    }
    match state
        .google_sign_in
        .find_student_by_subject(&claims.sub)
        .await
    {
        Ok(Some(student_id)) => match start_session(&state, student_id).await {
            Ok(session) => redirect("/", &[session, clear_flow]),
            Err(err) => store_errors::into_response(err, &id.0),
        },
        Ok(None) => park_new_account(&state, &id, &claims, clear_flow).await,
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

async fn verified_claims(
    google: &GoogleClient,
    state: &AppState,
    code: &str,
    attempt: &GoogleAttempt,
) -> Result<IdClaims, GoogleError> {
    let id_token = google.exchange_code(code, &attempt.code_verifier).await?;
    google
        .verify_id_token(&id_token, &attempt.nonce, state.clock.now())
        .await
}

/// A Google account with no student: keep it server-side and send the browser to pick a grade.
async fn park_new_account(
    state: &AppState,
    id: &RequestId,
    claims: &IdClaims,
    clear_flow: String,
) -> Response {
    let signup_token = tokens::new_token();
    let signup = PendingGoogleSignup {
        subject: claims.sub.clone(),
        display_name: google_signup::nickname_suggestion(claims.name.as_deref()),
    };
    let expires_at = state.clock.now() + chrono::Duration::seconds(MAX_AGE_SECONDS);
    if let Err(err) = state
        .google_sign_in
        .save_pending_signup(&tokens::hash_token(&signup_token), &signup, expires_at)
        .await
    {
        return store_errors::into_response(err, &id.0);
    }
    let signup_cookie =
        google_cookies::set(SIGNUP_COOKIE, &signup_token, state.config.cookie_secure);
    redirect("/daftar/google", &[signup_cookie, clear_flow])
}

/// `303 See Other` to `location`, with every cookie as its own `Set-Cookie` header.
pub(crate) fn redirect(location: &str, set_cookies: &[String]) -> Response {
    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, location.to_owned())],
        AppendHeaders(
            set_cookies
                .iter()
                .map(|cookie| (header::SET_COOKIE, cookie.clone())),
        ),
    )
        .into_response()
}
