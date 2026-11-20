//! "Masuk dengan Google" end to end, against an in-process fake Google: the start redirect, the
//! callback's state and ID-token checks, the grade step for a new account, and isolation between
//! browsers and between Google accounts.

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]
#![allow(clippy::panic, reason = "tests")]

mod google_fake;
mod support;

use axum::http::StatusCode;
use google_fake::flow::{
    FLOW, SIGNUP, app_with_google, callback, count, query_param, sign_in, sign_in_with, sign_up,
    start,
};
use google_fake::{CLIENT_ID, REDIRECT_URL, Signer};
use serde_json::json;
use sqlx::PgPool;
use support::{TestApp, TestResponse};

/// A refused callback: back to the login page with `google=<outcome>`, no session, no account,
/// nothing parked.
async fn assert_refused(res: &TestResponse, outcome: &str, pool: &PgPool) {
    assert_eq!(res.status, StatusCode::SEE_OTHER);
    assert_eq!(
        res.header("location").as_deref(),
        Some(format!("/masuk?google={outcome}").as_str())
    );
    assert_eq!(res.cookie(), None, "no session");
    assert_eq!(res.cookie_named(SIGNUP), None, "nothing parked");
    assert_eq!(count(pool, "google_identities").await, 0);
    assert_eq!(count(pool, "google_pending_signups").await, 0);
    assert_eq!(count(pool, "users").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn unconfigured_google_is_reported_off_and_its_routes_answer_google_disabled(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let res = app.get("/v1/auth/google", None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body, json!({ "isEnabled": false }));
    for path in [
        "/v1/auth/google/start",
        "/v1/auth/google/callback?code=x&state=y",
        "/v1/auth/google/signup",
    ] {
        let res = app.get(path, None).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{path}");
        assert_eq!(res.body["error"]["code"], "google_disabled", "{path}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn configured_google_is_reported_on(pool: PgPool) {
    let (app, _fake) = app_with_google(pool).await;
    let res = app.get("/v1/auth/google", None).await;
    assert_eq!(res.body, json!({ "isEnabled": true }));
}

#[sqlx::test(migrations = "../../migrations")]
async fn start_sends_the_browser_to_google_with_pkce_state_and_nonce(pool: PgPool) {
    let (app, _fake) = app_with_google(pool).await;
    let res = app.get("/v1/auth/google/start", None).await;
    assert_eq!(res.status, StatusCode::SEE_OTHER);
    let location = res.header("location").unwrap();
    assert!(location.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
    let url = reqwest::Url::parse(&location).unwrap();
    assert_eq!(query_param(&url, "response_type"), "code");
    assert_eq!(query_param(&url, "client_id"), CLIENT_ID);
    assert_eq!(query_param(&url, "redirect_uri"), REDIRECT_URL);
    assert_eq!(
        query_param(&url, "scope"),
        "openid profile",
        "no email asked for"
    );
    assert_eq!(query_param(&url, "code_challenge_method"), "S256");
    for name in ["state", "nonce", "code_challenge"] {
        assert_eq!(query_param(&url, name).len(), 43, "{name}");
    }
    assert!(
        !location.contains("secret"),
        "the client secret never goes to the browser"
    );
    let cookie = res
        .set_cookies()
        .into_iter()
        .find(|c| c.starts_with(FLOW))
        .unwrap();
    assert!(
        cookie.contains("HttpOnly") && cookie.contains("Max-Age=600"),
        "{cookie}"
    );
}

// ---- the positive control ---------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn a_first_sign_in_asks_for_a_grade_then_creates_and_links_exactly_one_student(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;

    let res = sign_in(&app, &fake, "google-sub-1", "Budi Santoso").await;
    assert_eq!(res.status, StatusCode::SEE_OTHER);
    assert_eq!(res.header("location").as_deref(), Some("/daftar/google"));
    assert_eq!(res.cookie(), None, "no session before the grade is picked");
    assert_eq!(count(&pool, "users").await, 0, "no student yet");
    let signup = res.cookie_named(SIGNUP).unwrap();

    let pending = app.get("/v1/auth/google/signup", Some(&signup)).await;
    assert_eq!(pending.status, StatusCode::OK);
    assert_eq!(pending.body, json!({ "suggestedNickname": "Budi Santoso" }));

    let bad = app
        .post(
            "/v1/auth/google/signup",
            json!({ "nickname": " ", "grade": 13 }),
            Some(&signup),
        )
        .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(bad.body["error"]["errors"].as_array().unwrap().len(), 2);

    let done = app
        .post(
            "/v1/auth/google/signup",
            json!({ "nickname": "Budi", "grade": 5 }),
            Some(&signup),
        )
        .await;
    assert_eq!(done.status, StatusCode::CREATED);
    let session = done.cookie().unwrap();
    let me = app.get("/v1/me", Some(&session)).await;
    assert_eq!(me.body["nickname"], "Budi");
    assert_eq!(me.body["grade"], 5);
    let student_id = me.body["id"].clone();
    assert_eq!(count(&pool, "users").await, 1);
    assert_eq!(count(&pool, "google_identities").await, 1);
    assert_eq!(count(&pool, "google_pending_signups").await, 0, "used up");

    // The pending signup is single use.
    let again = app
        .post(
            "/v1/auth/google/signup",
            json!({ "nickname": "Budi", "grade": 5 }),
            Some(&signup),
        )
        .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);

    // The next sign-in with the same Google account goes straight in, to the same student.
    let res = sign_in(&app, &fake, "google-sub-1", "Budi Santoso").await;
    assert_eq!(res.header("location").as_deref(), Some("/"));
    let me = app.get("/v1/me", Some(&res.cookie().unwrap())).await;
    assert_eq!(me.body["id"], student_id);
    assert_eq!(count(&pool, "users").await, 1, "linked, not created again");
    assert_eq!(count(&pool, "google_identities").await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_bare_issuer_form_is_accepted_too(pool: PgPool) {
    let (app, fake) = app_with_google(pool).await;
    let res = sign_in_with(
        &app,
        &fake,
        |s| {
            let mut claims = fake.claims("sub-bare", "Sari", &s.nonce);
            claims["iss"] = json!("accounts.google.com");
            claims
        },
        Signer::Published,
    )
    .await;
    assert_eq!(res.header("location").as_deref(), Some("/daftar/google"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_signing_keys_are_fetched_once_and_cached(pool: PgPool) {
    let (app, fake) = app_with_google(pool).await;
    sign_in(&app, &fake, "sub-1", "A").await;
    sign_in(&app, &fake, "sub-2", "B").await;
    assert_eq!(fake.jwks_hits(), 1);
}

// ---- the state check ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn a_state_mismatch_is_rejected_before_the_code_is_used(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let started = start(&app).await;
    let code = fake.issue_code(
        &started.challenge,
        &fake.claims("sub-1", "A", &started.nonce),
        Signer::Published,
    );
    let forged = format!("x{}", &started.state[1..]);
    let res = callback(
        &app,
        &format!("code={code}&state={forged}"),
        &started.flow_cookie,
    )
    .await;
    assert_refused(&res, "failed", &pool).await;
    assert_eq!(fake.token_hits(), 0, "the code is never exchanged");

    // The attempt is used up: the right state afterwards is refused too.
    let res = callback(
        &app,
        &format!("code={code}&state={}", started.state),
        &started.flow_cookie,
    )
    .await;
    assert_refused(&res, "failed", &pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_missing_state_is_rejected(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let started = start(&app).await;
    let code = fake.issue_code(
        &started.challenge,
        &fake.claims("sub-1", "A", &started.nonce),
        Signer::Published,
    );
    let res = callback(&app, &format!("code={code}"), &started.flow_cookie).await;
    assert_refused(&res, "failed", &pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_callback_in_a_browser_that_did_not_start_it_is_rejected(pool: PgPool) {
    // Login CSRF: an attacker's own code and state, replayed into the victim's browser.
    let (app, fake) = app_with_google(pool.clone()).await;
    let started = start(&app).await;
    let code = fake.issue_code(
        &started.challenge,
        &fake.claims("attacker", "A", &started.nonce),
        Signer::Published,
    );
    let query = format!("code={code}&state={}", started.state);
    let res = app
        .get(&format!("/v1/auth/google/callback?{query}"), None)
        .await;
    assert_refused(&res, "failed", &pool).await;
    let res = callback(&app, &query, &format!("{FLOW}=someone-elses")).await;
    assert_refused(&res, "failed", &pool).await;
}

// ---- the ID-token checks -----------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn an_id_token_signed_by_another_key_is_rejected(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let res = sign_in_with(
        &app,
        &fake,
        |s| fake.claims("sub-1", "A", &s.nonce),
        Signer::Rogue,
    )
    .await;
    assert_refused(&res, "failed", &pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_id_token_for_another_client_is_rejected(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let res = sign_in_with(
        &app,
        &fake,
        |s| {
            let mut claims = fake.claims("sub-1", "A", &s.nonce);
            claims["aud"] = json!("another-app.apps.example");
            claims
        },
        Signer::Published,
    )
    .await;
    assert_refused(&res, "failed", &pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_id_token_from_another_issuer_is_rejected(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let res = sign_in_with(
        &app,
        &fake,
        |s| {
            let mut claims = fake.claims("sub-1", "A", &s.nonce);
            claims["iss"] = json!("https://accounts.evil.example");
            claims
        },
        Signer::Published,
    )
    .await;
    assert_refused(&res, "failed", &pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_expired_id_token_is_rejected(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let res = sign_in_with(
        &app,
        &fake,
        |s| {
            let mut claims = fake.claims("sub-1", "A", &s.nonce);
            let past = chrono::Utc::now().timestamp() - 7_200;
            claims["iat"] = json!(past - 3_600);
            claims["exp"] = json!(past);
            claims
        },
        Signer::Published,
    )
    .await;
    assert_refused(&res, "failed", &pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_id_token_with_another_nonce_is_rejected(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let res = sign_in_with(
        &app,
        &fake,
        |_| fake.claims("sub-1", "A", "replayed-nonce"),
        Signer::Published,
    )
    .await;
    assert_refused(&res, "failed", &pool).await;
}

// ---- the other outcomes ------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn cancelling_on_googles_page_says_cancelled(pool: PgPool) {
    let (app, _fake) = app_with_google(pool.clone()).await;
    let started = start(&app).await;
    let res = callback(
        &app,
        &format!("error=access_denied&state={}", started.state),
        &started.flow_cookie,
    )
    .await;
    assert_refused(&res, "cancelled", &pool).await;
    assert!(
        res.set_cookies()
            .iter()
            .any(|c| c.starts_with(&format!("{FLOW}=;"))),
        "the flow cookie is cleared"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn google_being_down_says_unavailable(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    fake.fail_token_requests_with(503);
    let res = sign_in(&app, &fake, "sub-1", "A").await;
    assert_refused(&res, "unavailable", &pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_code_google_refuses_says_failed(pool: PgPool) {
    let (app, _fake) = app_with_google(pool.clone()).await;
    let started = start(&app).await;
    let res = callback(
        &app,
        &format!("code=never-issued&state={}", started.state),
        &started.flow_cookie,
    )
    .await;
    assert_refused(&res, "failed", &pool).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_tabs_finishing_the_same_google_signup_conflict(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let first = sign_in(&app, &fake, "sub-1", "A")
        .await
        .cookie_named(SIGNUP)
        .unwrap();
    let second = sign_in(&app, &fake, "sub-1", "A")
        .await
        .cookie_named(SIGNUP)
        .unwrap();
    let body = json!({ "nickname": "A", "grade": 3 });
    let res = app
        .post("/v1/auth/google/signup", body.clone(), Some(&first))
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    let res = app
        .post("/v1/auth/google/signup", body, Some(&second))
        .await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.body["error"]["code"], "conflict");
    assert_eq!(count(&pool, "users").await, 1);
    assert_eq!(count(&pool, "google_identities").await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_google_student_has_no_password_to_log_in_with_or_change(pool: PgPool) {
    let (app, fake) = app_with_google(pool).await;
    let session = sign_up(&app, &fake, "sub-1", "Ani").await;
    let username = app.get("/v1/me", Some(&session)).await.body["username"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(username.starts_with("google."));
    let res = app
        .post(
            "/v1/auth/login",
            json!({ "username": username, "password": "anything-at-all" }),
            None,
        )
        .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    let res = app
        .post(
            "/v1/auth/password",
            json!({ "currentPassword": "anything", "newPassword": "rahasia-baru-123" }),
            Some(&session),
        )
        .await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.body["error"]["code"], "password_not_set");
    // Whatever the body holds: an empty form gets the same answer, not a field error.
    let res = app
        .post(
            "/v1/auth/password",
            json!({ "currentPassword": "", "newPassword": "" }),
            Some(&session),
        )
        .await;
    assert_eq!(res.body["error"]["code"], "password_not_set");
    let me = app.get("/v1/me", Some(&session)).await;
    assert_eq!(me.body["hasPassword"], false);
    assert_eq!(me.body["isGoogleLinked"], true);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_password_student_still_changes_their_password_and_is_told_a_wrong_one(pool: PgPool) {
    let (app, _fake) = app_with_google(pool).await;
    let session = app.student("budi").await;
    let me = app.get("/v1/me", Some(&session)).await;
    assert_eq!(me.body["hasPassword"], true);
    assert_eq!(me.body["isGoogleLinked"], false);
    let wrong = app
        .post(
            "/v1/auth/password",
            json!({ "currentPassword": "salah-sekali", "newPassword": "rahasia-baru-123" }),
            Some(&session),
        )
        .await;
    assert_eq!(wrong.status, StatusCode::FORBIDDEN);
    assert_eq!(wrong.body["error"]["code"], "permission_denied");
    let right = app
        .post(
            "/v1/auth/password",
            json!({ "currentPassword": "rahasia-123", "newPassword": "rahasia-baru-123" }),
            Some(&session),
        )
        .await;
    assert_eq!(right.status, StatusCode::NO_CONTENT);
}

// ---- isolation ---------------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn a_pending_signup_answers_only_the_browser_that_started_it(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let ani = sign_in(&app, &fake, "sub-ani", "Ani")
        .await
        .cookie_named(SIGNUP)
        .unwrap();
    let budi = sign_in(&app, &fake, "sub-budi", "Budi")
        .await
        .cookie_named(SIGNUP)
        .unwrap();
    let stranger = format!("{SIGNUP}=not-a-real-token");
    let body = json!({ "nickname": "Siapa", "grade": 4 });

    for cookie in [None, Some(stranger.as_str())] {
        let res = app.get("/v1/auth/google/signup", cookie).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND);
        assert_eq!(res.body["error"]["code"], "not_found");
        let res = app
            .post("/v1/auth/google/signup", body.clone(), cookie)
            .await;
        assert_eq!(res.status, StatusCode::NOT_FOUND);
    }
    assert_eq!(count(&pool, "users").await, 0);

    // Positive control: each browser sees and completes its own, not the other's.
    let res = app.get("/v1/auth/google/signup", Some(&budi)).await;
    assert_eq!(res.body["suggestedNickname"], "Budi");
    let res = app.get("/v1/auth/google/signup", Some(&ani)).await;
    assert_eq!(res.body["suggestedNickname"], "Ani");
    let res = app
        .post(
            "/v1/auth/google/signup",
            json!({ "nickname": "Ani", "grade": 4 }),
            Some(&ani),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    let res = app.get("/v1/auth/google/signup", Some(&budi)).await;
    assert_eq!(res.status, StatusCode::OK, "Budi's is untouched");
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_google_account_signs_in_to_its_own_student_only(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let ani = sign_up(&app, &fake, "sub-ani", "Ani").await;
    let budi = sign_up(&app, &fake, "sub-budi", "Budi").await;
    let ani_id = app.get("/v1/me", Some(&ani)).await.body["id"].clone();
    let budi_id = app.get("/v1/me", Some(&budi)).await.body["id"].clone();
    assert_ne!(ani_id, budi_id);

    for (sub, id) in [("sub-ani", &ani_id), ("sub-budi", &budi_id)] {
        let res = sign_in(&app, &fake, sub, "x").await;
        assert_eq!(res.header("location").as_deref(), Some("/"));
        let me = app.get("/v1/me", Some(&res.cookie().unwrap())).await;
        assert_eq!(&me.body["id"], id, "{sub} lands on its own student");
    }
    assert_eq!(count(&pool, "google_identities").await, 2);
}
