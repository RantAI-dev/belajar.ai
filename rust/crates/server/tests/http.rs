#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use belajar_server::{Config, app};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

fn test_app() -> axum::Router {
    app(&Config {
        web_dir: std::env::temp_dir().join("belajar-no-web"),
    })
}

async fn call(path: &str, request_id: Option<&str>) -> (StatusCode, Option<String>, Value) {
    let mut req = Request::get(path);
    if let Some(id) = request_id {
        req = req.header("x-request-id", id);
    }
    let res = test_app()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let header = res
        .headers()
        .get("x-request-id")
        .map(|v| v.to_str().unwrap().to_owned());
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, header, body)
}

#[tokio::test]
async fn health_answers_ok() {
    let (status, header, body) = call("/v1/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert!(header.is_some(), "every response carries x-request-id");
}

#[tokio::test]
async fn unknown_api_path_returns_the_standard_error_body() {
    let (status, header, body) = call("/v1/does-not-exist", Some("req-123")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    assert_eq!(body["error"]["requestId"], "req-123");
    assert_eq!(header.as_deref(), Some("req-123"));
}

#[tokio::test]
async fn an_unsafe_request_id_is_replaced_not_echoed() {
    let (_, header, body) = call("/v1/nope", Some("bad id with spaces")).await;
    let id = header.unwrap();
    assert_ne!(id, "bad id with spaces");
    assert_eq!(body["error"]["requestId"], id.as_str());
}
