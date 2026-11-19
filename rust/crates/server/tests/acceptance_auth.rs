//! Acceptance tests for the foundation + auth/profile slice. Written by the lead before the slice
//! started; the worker must make them pass without editing this file. They use the harness in
//! `tests/support/mod.rs` (the worker writes it, with exactly the API used here).
#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]

mod support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use support::TestApp;

const PASSWORD: &str = "rahasia-123";

fn register_body(username: &str) -> Value {
    json!({ "username": username, "password": PASSWORD, "nickname": "Budi", "grade": 7 })
}

fn error_fields(body: &Value) -> Vec<String> {
    let mut fields: Vec<String> = body["error"]["errors"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|e| e["field"].as_str().unwrap().to_owned())
                .collect()
        })
        .unwrap_or_default();
    fields.sort();
    fields
}

fn assert_no_snake_case_keys(value: &Value) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                assert!(!key.contains('_'), "response key {key} is not camelCase");
                assert_no_snake_case_keys(inner);
            }
        }
        Value::Array(items) => items.iter().for_each(assert_no_snake_case_keys),
        _ => {}
    }
}

fn is_instant(text: &str) -> bool {
    // 2026-11-11T08:30:00.000Z: RFC 3339, UTC, exactly three decimals.
    let b = text.as_bytes();
    b.len() == 24
        && b[4] == b'-'
        && b[10] == b'T'
        && b[19] == b'.'
        && b[23] == b'Z'
        && b.iter()
            .enumerate()
            .all(|(i, c)| matches!(i, 4 | 7 | 10 | 13 | 16 | 19 | 23) || c.is_ascii_digit())
}

#[sqlx::test(migrations = "../../migrations")]
async fn register_sets_a_session_cookie_and_returns_the_student(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let res = app
        .post("/v1/auth/register", register_body("budi"), None)
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_eq!(res.body["username"], "budi");
    assert_eq!(res.body["nickname"], "Budi");
    assert_eq!(res.body["grade"], 7);
    assert!(res.body["id"].is_string());
    assert!(is_instant(res.body["createdAt"].as_str().unwrap()));
    assert!(res.body.get("password").is_none() && res.body.get("passwordHash").is_none());
    assert_no_snake_case_keys(&res.body);

    let set_cookie = res.set_cookie_header().expect("register sets a cookie");
    assert!(set_cookie.starts_with("belajar_session="));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Lax"));
    assert!(set_cookie.contains("Path=/"));

    let cookie = res.cookie().unwrap();
    let me = app.get("/v1/me", Some(&cookie)).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.body["id"], res.body["id"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn usernames_are_unique_regardless_of_case(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.student("budi").await;
    let res = app
        .post("/v1/auth/register", register_body("BUDI"), None)
        .await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.body["error"]["code"], "conflict");
}

#[sqlx::test(migrations = "../../migrations")]
async fn register_reports_every_bad_field(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let body = json!({ "username": "a", "password": "short", "nickname": "Budi", "grade": 13 });
    let res = app.post("/v1/auth/register", body, None).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"]["code"], "invalid_request");
    assert_eq!(
        error_fields(&res.body),
        vec!["/grade", "/password", "/username"]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_fields_and_wrong_content_types_are_rejected(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let mut body = register_body("budi");
    body["isAdmin"] = json!(true);
    let res = app.post("/v1/auth/register", body, None).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"]["code"], "invalid_request");

    let res = app
        .post_raw("/v1/auth/register", "application/json", "{not json", None)
        .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"]["code"], "invalid_request");

    let res = app
        .post_raw("/v1/auth/register", "text/plain", "username=budi", None)
        .await;
    assert_eq!(res.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(res.body["error"]["code"], "unsupported_media_type");
}

#[sqlx::test(migrations = "../../migrations")]
async fn passwords_are_stored_as_argon2id(pool: PgPool) {
    let app = TestApp::new(pool.clone()).await;
    app.student("budi").await;
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(
        hash.starts_with("$argon2id$"),
        "stored hash is not argon2id"
    );
    assert!(!hash.contains(PASSWORD));
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_with_a_wrong_password_or_unknown_user_is_unauthenticated(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.student("budi").await;
    for body in [
        json!({ "username": "budi", "password": "salah-salah" }),
        json!({ "username": "tidak-ada", "password": PASSWORD }),
    ] {
        let res = app.post("/v1/auth/login", body, None).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert_eq!(res.body["error"]["code"], "unauthenticated");
        assert!(res.headers.contains_key("www-authenticate"));
        assert!(res.cookie().is_none());
    }
    let ok = app
        .post(
            "/v1/auth/login",
            json!({ "username": "Budi", "password": PASSWORD }),
            None,
        )
        .await;
    assert_eq!(ok.status, StatusCode::OK, "username match ignores case");
    assert_eq!(ok.body["username"], "budi");
    assert!(ok.cookie().is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_is_rate_limited_after_five_failures(pool: PgPool) {
    let app = TestApp::new(pool).await;
    app.student("budi").await;
    for _ in 0..5 {
        let res = app
            .post(
                "/v1/auth/login",
                json!({ "username": "budi", "password": "salah-salah" }),
                None,
            )
            .await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    }
    // Even the right password is refused until the window passes.
    let res = app
        .post(
            "/v1/auth/login",
            json!({ "username": "budi", "password": PASSWORD }),
            None,
        )
        .await;
    assert_eq!(res.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(res.body["error"]["code"], "rate_limited");
    let retry: u64 = res.headers["retry-after"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(retry > 0);

    // Another account is not affected.
    app.student("siti").await;
    let other = app
        .post(
            "/v1/auth/login",
            json!({ "username": "siti", "password": PASSWORD }),
            None,
        )
        .await;
    assert_eq!(other.status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn me_needs_a_valid_session(pool: PgPool) {
    let app = TestApp::new(pool).await;
    for cookie in [None, Some("belajar_session=not-a-real-token")] {
        let res = app.get("/v1/me", cookie).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert_eq!(res.body["error"]["code"], "unauthenticated");
        assert!(res.headers.contains_key("www-authenticate"));
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_session_sees_only_its_own_student(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let budi = app.student("budi").await;
    let siti = app.student("siti").await;
    assert_eq!(
        app.get("/v1/me", Some(&budi)).await.body["username"],
        "budi"
    );
    assert_eq!(
        app.get("/v1/me", Some(&siti)).await.body["username"],
        "siti"
    );
    let res = app
        .patch("/v1/me", json!({ "nickname": "Sitii" }), Some(&siti))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(
        app.get("/v1/me", Some(&budi)).await.body["nickname"],
        "budi"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn logout_ends_the_session(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let cookie = app.student("budi").await;
    let res = app.post("/v1/auth/logout", json!({}), Some(&cookie)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let cleared = res.set_cookie_header().expect("logout clears the cookie");
    assert!(cleared.starts_with("belajar_session="));
    assert!(cleared.contains("Max-Age=0"));
    assert_eq!(
        app.get("/v1/me", Some(&cookie)).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn patch_me_changes_grade_and_nickname_within_limits(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let cookie = app.student("budi").await;
    let res = app
        .patch(
            "/v1/me",
            json!({ "grade": 12, "nickname": "Budi Kecil" }),
            Some(&cookie),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["grade"], 12);
    assert_eq!(res.body["nickname"], "Budi Kecil");
    assert_eq!(app.get("/v1/me", Some(&cookie)).await.body["grade"], 12);

    for (body, field) in [
        (json!({ "grade": 0 }), "/grade"),
        (json!({ "grade": 13 }), "/grade"),
        (json!({ "nickname": "" }), "/nickname"),
    ] {
        let res = app.patch("/v1/me", body, Some(&cookie)).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST);
        assert_eq!(res.body["error"]["code"], "invalid_request");
        assert_eq!(error_fields(&res.body), vec![field]);
    }
    let res = app
        .patch("/v1/me", json!({ "username": "lain" }), Some(&cookie))
        .await;
    assert_eq!(
        res.status,
        StatusCode::BAD_REQUEST,
        "username is not editable"
    );
    assert_eq!(app.get("/v1/me", Some(&cookie)).await.body["grade"], 12);
}

#[sqlx::test(migrations = "../../migrations")]
async fn changing_the_password_keeps_this_session_and_ends_the_others(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let here = app.student("budi").await;
    let elsewhere = app
        .post(
            "/v1/auth/login",
            json!({ "username": "budi", "password": PASSWORD }),
            None,
        )
        .await
        .cookie()
        .unwrap();

    let wrong = app
        .post(
            "/v1/auth/password",
            json!({ "currentPassword": "salah-salah", "newPassword": "baru-sekali-1" }),
            Some(&here),
        )
        .await;
    assert_eq!(wrong.status, StatusCode::FORBIDDEN);
    assert_eq!(wrong.body["error"]["code"], "permission_denied");

    let short = app
        .post(
            "/v1/auth/password",
            json!({ "currentPassword": PASSWORD, "newPassword": "pendek" }),
            Some(&here),
        )
        .await;
    assert_eq!(short.status, StatusCode::BAD_REQUEST);
    assert_eq!(error_fields(&short.body), vec!["/newPassword"]);

    let ok = app
        .post(
            "/v1/auth/password",
            json!({ "currentPassword": PASSWORD, "newPassword": "baru-sekali-1" }),
            Some(&here),
        )
        .await;
    assert_eq!(ok.status, StatusCode::NO_CONTENT);
    assert_eq!(app.get("/v1/me", Some(&here)).await.status, StatusCode::OK);
    assert_eq!(
        app.get("/v1/me", Some(&elsewhere)).await.status,
        StatusCode::UNAUTHORIZED
    );

    let old = app
        .post(
            "/v1/auth/login",
            json!({ "username": "budi", "password": PASSWORD }),
            None,
        )
        .await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    let new = app
        .post(
            "/v1/auth/login",
            json!({ "username": "budi", "password": "baru-sekali-1" }),
            None,
        )
        .await;
    assert_eq!(new.status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn health_and_ready_are_unversioned(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let health = app.get("/health", None).await;
    assert_eq!(health.status, StatusCode::OK);
    assert_eq!(health.body["status"], "ok");
    let ready = app.get("/ready", None).await;
    assert_eq!(ready.status, StatusCode::OK);
    assert_eq!(ready.body["status"], "ok");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_committed_web_contract_matches_the_served_openapi(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let served = app.get("/v1/openapi.json", None).await;
    assert_eq!(served.status, StatusCode::OK);
    for path in [
        "/v1/auth/register",
        "/v1/auth/login",
        "/v1/auth/logout",
        "/v1/auth/password",
        "/v1/me",
    ] {
        assert!(
            served.body["paths"].get(path).is_some(),
            "openapi lacks {path}"
        );
    }
    let committed = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../web/src/contracts/openapi.json");
    let text = std::fs::read_to_string(&committed).expect("web/src/contracts/openapi.json exists");
    let committed: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        committed, served.body,
        "web/src/contracts/openapi.json is stale: run scripts/gen-contracts.sh"
    );
}
