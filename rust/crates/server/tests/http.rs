//! The HTTP contract around the whole app: request ids and the JSON 404 fallback.

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]

mod support;

use axum::http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;
use support::TestApp;

#[sqlx::test(migrations = "../../migrations")]
async fn health_answers_ok(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let res = app.get("/v1/health", None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["status"], "ok");
    assert!(
        res.header("x-request-id").is_some(),
        "every response carries x-request-id"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_api_path_returns_the_standard_error_body(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let res = app
        .get_with_headers("/v1/does-not-exist", None, &[("x-request-id", "req-123")])
        .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body["error"]["code"], "not_found");
    assert_eq!(res.body["error"]["requestId"], "req-123");
    assert_eq!(res.header("x-request-id").as_deref(), Some("req-123"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unsafe_request_id_is_replaced_not_echoed(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let res = app
        .get_with_headers("/v1/nope", None, &[("x-request-id", "bad id with spaces")])
        .await;
    let id = res.header("x-request-id").unwrap();
    assert_ne!(id, "bad id with spaces");
    assert_eq!(res.body["error"]["requestId"], Value::String(id));
}
