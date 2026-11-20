//! The browser's side of a Google sign-in or link, shared by the Google test files: start,
//! let the fake issue a code, come back to the callback.

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;

use super::{FakeGoogle, Signer};
use crate::support::{TestApp, TestResponse};

pub const FLOW: &str = "belajar_google_flow";
pub const SIGNUP: &str = "belajar_google_signup";

/// What the start step handed the browser.
pub struct Started {
    pub state: String,
    pub nonce: String,
    pub challenge: String,
    /// "belajar_google_flow=<value>", for the Cookie header.
    pub flow_cookie: String,
}

pub fn query_param(url: &reqwest::Url, name: &str) -> String {
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
        .unwrap_or_else(|| panic!("{name} in {url}"))
}

pub async fn start(app: &TestApp) -> Started {
    start_at(app, "/v1/auth/google/start", None).await
}

/// "Hubungkan dengan Google" from Akun, in the session `session` ("belajar_session=<token>").
pub async fn start_link(app: &TestApp, session: &str) -> Started {
    start_at(app, "/v1/auth/google/link/start", Some(session)).await
}

async fn start_at(app: &TestApp, path: &str, cookie: Option<&str>) -> Started {
    let res = app.get(path, cookie).await;
    assert_eq!(res.status, StatusCode::SEE_OTHER, "{}", res.text);
    let url = reqwest::Url::parse(&res.header("location").unwrap()).unwrap();
    Started {
        state: query_param(&url, "state"),
        nonce: query_param(&url, "nonce"),
        challenge: query_param(&url, "code_challenge"),
        flow_cookie: res.cookie_named(FLOW).expect("a flow cookie"),
    }
}

pub async fn callback(app: &TestApp, query: &str, cookie: &str) -> TestResponse {
    app.get(&format!("/v1/auth/google/callback?{query}"), Some(cookie))
        .await
}

/// Start, let the fake Google issue a code for `claims`, and come back with it.
pub async fn sign_in_with(
    app: &TestApp,
    fake: &FakeGoogle,
    claims: impl FnOnce(&Started) -> Value,
    signer: Signer,
) -> TestResponse {
    let started = start(app).await;
    let code = fake.issue_code(&started.challenge, &claims(&started), signer);
    callback(
        app,
        &format!("code={code}&state={}", started.state),
        &started.flow_cookie,
    )
    .await
}

pub async fn sign_in(app: &TestApp, fake: &FakeGoogle, sub: &str, name: &str) -> TestResponse {
    sign_in_with(
        app,
        fake,
        |s| fake.claims(sub, name, &s.nonce),
        Signer::Published,
    )
    .await
}

/// A new Google account all the way to a signed-in student. Returns the session cookie.
pub async fn sign_up(app: &TestApp, fake: &FakeGoogle, sub: &str, nickname: &str) -> String {
    let res = sign_in(app, fake, sub, nickname).await;
    assert_eq!(res.header("location").as_deref(), Some("/daftar/google"));
    let signup = res.cookie_named(SIGNUP).unwrap();
    let done = app
        .post(
            "/v1/auth/google/signup",
            json!({ "nickname": nickname, "grade": 5 }),
            Some(&signup),
        )
        .await;
    assert_eq!(done.status, StatusCode::CREATED, "{}", done.text);
    done.cookie().expect("a session cookie")
}

pub async fn count(pool: &PgPool, table: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(&format!("SELECT count(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap()
}

pub async fn app_with_google(pool: PgPool) -> (TestApp, FakeGoogle) {
    let fake = FakeGoogle::start().await;
    (TestApp::with_google(pool, fake.config()).await, fake)
}
