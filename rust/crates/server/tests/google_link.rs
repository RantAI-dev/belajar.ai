//! "Hubungkan dengan Google" from Akun, end to end against the fake Google: a signed-in student
//! with a password links a Google account through the same callback as a sign-in. The link is
//! for the student and session recorded at start, never for whoever holds the callback.

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]
#![allow(clippy::panic, reason = "tests")]

mod google_fake;
mod support;

use axum::http::StatusCode;
use google_fake::flow::{Started, app_with_google, callback, count, sign_in, sign_up, start_link};
use google_fake::{FakeGoogle, Signer};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{TestApp, TestResponse};
use uuid::Uuid;

/// The flow cookie and a session together, as the browser sends them to the callback.
fn both(started: &Started, session: &str) -> String {
    format!("{}; {session}", started.flow_cookie)
}

/// Let the fake Google answer `started` for `sub` and come back with `cookie`.
async fn finish(
    app: &TestApp,
    fake: &FakeGoogle,
    started: &Started,
    sub: &str,
    cookie: &str,
) -> TestResponse {
    let code = fake.issue_code(
        &started.challenge,
        &fake.claims(sub, "Nama Google", &started.nonce),
        Signer::Published,
    );
    callback(app, &format!("code={code}&state={}", started.state), cookie).await
}

/// The whole link, in the session that started it.
async fn link(app: &TestApp, fake: &FakeGoogle, session: &str, sub: &str) -> TestResponse {
    let started = start_link(app, session).await;
    finish(app, fake, &started, sub, &both(&started, session)).await
}

async fn me(app: &TestApp, session: &str) -> Value {
    let res = app.get("/v1/me", Some(session)).await;
    assert_eq!(res.status, StatusCode::OK);
    res.body
}

async fn id_of(app: &TestApp, session: &str) -> Uuid {
    me(app, session).await["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

/// Every `(subject, student_id)` row, sorted.
async fn identities(pool: &PgPool) -> Vec<(String, Uuid)> {
    sqlx::query_as::<_, (String, Uuid)>(
        "SELECT subject, student_id FROM google_identities ORDER BY subject",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

fn assert_lands(res: &TestResponse, outcome: &str) {
    assert_eq!(res.status, StatusCode::SEE_OTHER, "{}", res.text);
    assert_eq!(
        res.header("location").as_deref(),
        Some(format!("/akun?google={outcome}").as_str())
    );
    assert_eq!(
        res.cookie(),
        None,
        "a link never starts or changes a session"
    );
}

// ---- the positive control ---------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn linking_creates_exactly_one_identity_for_the_signed_in_student(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let budi = app.student("budi").await;
    let siti = app.student("siti").await;
    let budi_id = id_of(&app, &budi).await;
    assert_eq!(me(&app, &budi).await["isGoogleLinked"], false);

    let res = link(&app, &fake, &budi, "sub-budi").await;
    assert_lands(&res, "linked");
    assert_eq!(
        identities(&pool).await,
        vec![("sub-budi".to_owned(), budi_id)]
    );
    let after = me(&app, &budi).await;
    assert_eq!(after["isGoogleLinked"], true);
    assert_eq!(after["hasPassword"], true, "the password stays");
    assert_eq!(me(&app, &siti).await["isGoogleLinked"], false);

    // Now Google signs Budi in, to Budi; the password works too.
    let res = sign_in(&app, &fake, "sub-budi", "x").await;
    assert_eq!(res.header("location").as_deref(), Some("/"));
    assert_eq!(id_of(&app, &res.cookie().unwrap()).await, budi_id);
    let res = app
        .post(
            "/v1/auth/login",
            json!({ "username": "budi", "password": "rahasia-123" }),
            None,
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(count(&pool, "users").await, 2, "no student created");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_same_google_account_finishing_twice_for_the_same_student_is_still_linked(
    pool: PgPool,
) {
    // Two tabs: both start before either finishes, both come back with the same account.
    let (app, fake) = app_with_google(pool.clone()).await;
    let budi = app.student("budi").await;
    let first = start_link(&app, &budi).await;
    let second = start_link(&app, &budi).await;
    assert_lands(
        &finish(&app, &fake, &first, "sub-budi", &both(&first, &budi)).await,
        "linked",
    );
    assert_lands(
        &finish(&app, &fake, &second, "sub-budi", &both(&second, &budi)).await,
        "linked",
    );
    assert_eq!(count(&pool, "google_identities").await, 1);
}

// ---- conflict ----------------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn a_google_account_of_another_student_is_not_moved(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let ani = sign_up(&app, &fake, "sub-ani", "Ani").await;
    let ani_id = id_of(&app, &ani).await;
    let budi = app.student("budi").await;
    let before_ani = me(&app, &ani).await;
    let before_budi = me(&app, &budi).await;

    let res = link(&app, &fake, &budi, "sub-ani").await;
    assert_lands(&res, "conflict");
    assert_eq!(
        identities(&pool).await,
        vec![("sub-ani".to_owned(), ani_id)]
    );
    assert_eq!(me(&app, &ani).await, before_ani, "Ani unchanged");
    assert_eq!(me(&app, &budi).await, before_budi, "Budi unchanged");

    // Google still signs Ani in, not Budi.
    let res = sign_in(&app, &fake, "sub-ani", "Ani").await;
    assert_eq!(id_of(&app, &res.cookie().unwrap()).await, ani_id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_linked_student_cannot_start_a_second_link(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let budi = app.student("budi").await;
    assert_lands(&link(&app, &fake, &budi, "sub-budi").await, "linked");
    let res = app.get("/v1/auth/google/link/start", Some(&budi)).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.body["error"]["code"], "google_already_linked");

    // A student who signed up with Google is linked from the start.
    let ani = sign_up(&app, &fake, "sub-ani", "Ani").await;
    let res = app.get("/v1/auth/google/link/start", Some(&ani)).await;
    assert_eq!(res.body["error"]["code"], "google_already_linked");
    assert_eq!(
        count(&pool, "google_sign_in_attempts").await,
        0,
        "no attempt saved"
    );
}

// ---- the session check -------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn a_link_started_by_one_student_cannot_finish_in_another_students_session(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let budi = app.student("budi").await;
    let siti = app.student("siti").await;
    let started = start_link(&app, &budi).await;

    let res = finish(&app, &fake, &started, "sub-x", &both(&started, &siti)).await;
    assert_lands(&res, "failed");
    assert_eq!(fake.token_hits(), 0, "the code is never exchanged");
    assert_eq!(count(&pool, "google_identities").await, 0);
    assert_eq!(me(&app, &siti).await["isGoogleLinked"], false);
    assert_eq!(me(&app, &budi).await["isGoogleLinked"], false);

    // The attempt is used up: Budi cannot finish it afterwards either (an unknown attempt says
    // nothing about being a link, so it lands on the login page like any stale callback).
    let res = finish(&app, &fake, &started, "sub-x", &both(&started, &budi)).await;
    assert_eq!(
        res.header("location").as_deref(),
        Some("/masuk?google=failed")
    );
    assert_eq!(count(&pool, "google_identities").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_link_whose_session_ended_or_changed_is_refused(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let budi = app.student("budi").await;

    // No session cookie at the callback.
    let started = start_link(&app, &budi).await;
    let res = finish(&app, &fake, &started, "sub-budi", &started.flow_cookie).await;
    assert_lands(&res, "failed");

    // Another session of the same student (signed in again elsewhere): not the one that started.
    let started = start_link(&app, &budi).await;
    let other = app
        .post(
            "/v1/auth/login",
            json!({ "username": "budi", "password": "rahasia-123" }),
            None,
        )
        .await
        .cookie()
        .unwrap();
    let res = finish(&app, &fake, &started, "sub-budi", &both(&started, &other)).await;
    assert_lands(&res, "failed");

    // Logged out between start and callback: the session's attempts end with it, so the
    // callback is as unknown as a forged one.
    let started = start_link(&app, &budi).await;
    let out = app.post("/v1/auth/logout", json!({}), Some(&budi)).await;
    assert_eq!(out.status, StatusCode::NO_CONTENT);
    let res = finish(&app, &fake, &started, "sub-budi", &both(&started, &budi)).await;
    assert_eq!(
        res.header("location").as_deref(),
        Some("/masuk?google=failed")
    );
    assert_eq!(fake.token_hits(), 0, "no code is ever exchanged");
    assert_eq!(count(&pool, "google_identities").await, 0);
}

// ---- the same checks as a sign-in --------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn a_state_mismatch_on_the_link_path_is_rejected(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let budi = app.student("budi").await;
    let started = start_link(&app, &budi).await;
    let code = fake.issue_code(
        &started.challenge,
        &fake.claims("sub-budi", "Budi", &started.nonce),
        Signer::Published,
    );
    let forged = format!("x{}", &started.state[1..]);
    let res = callback(
        &app,
        &format!("code={code}&state={forged}"),
        &both(&started, &budi),
    )
    .await;
    assert_lands(&res, "failed");
    assert_eq!(fake.token_hits(), 0);
    assert_eq!(count(&pool, "google_identities").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_bad_id_token_on_the_link_path_is_rejected(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let budi = app.student("budi").await;

    let started = start_link(&app, &budi).await;
    let code = fake.issue_code(
        &started.challenge,
        &fake.claims("sub-budi", "Budi", &started.nonce),
        Signer::Rogue,
    );
    let query = format!("code={code}&state={}", started.state);
    assert_lands(
        &callback(&app, &query, &both(&started, &budi)).await,
        "failed",
    );

    let started = start_link(&app, &budi).await;
    let code = fake.issue_code(
        &started.challenge,
        &fake.claims("sub-budi", "Budi", "replayed-nonce"),
        Signer::Published,
    );
    let query = format!("code={code}&state={}", started.state);
    assert_lands(
        &callback(&app, &query, &both(&started, &budi)).await,
        "failed",
    );
    assert_eq!(count(&pool, "google_identities").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn cancelling_or_google_being_down_comes_back_to_akun(pool: PgPool) {
    let (app, fake) = app_with_google(pool.clone()).await;
    let budi = app.student("budi").await;

    let started = start_link(&app, &budi).await;
    let query = format!("error=access_denied&state={}", started.state);
    assert_lands(
        &callback(&app, &query, &both(&started, &budi)).await,
        "cancelled",
    );

    fake.fail_token_requests_with(503);
    let res = link(&app, &fake, &budi, "sub-budi").await;
    assert_lands(&res, "unavailable");
    assert_eq!(count(&pool, "google_identities").await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn starting_a_link_needs_a_session_and_google(pool: PgPool) {
    let (app, _fake) = app_with_google(pool.clone()).await;
    let res = app.get("/v1/auth/google/link/start", None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"]["code"], "unauthenticated");

    let off = TestApp::new(pool).await;
    let budi = off.student("budi").await;
    let res = off.get("/v1/auth/google/link/start", Some(&budi)).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body["error"]["code"], "google_disabled");
}
