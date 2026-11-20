//! Test harness for the server's HTTP tests: exactly the API documented here, driven in-process
//! through the router (tower `oneshot`), with the cookie `Secure` flag off.
//!
//! ```text
//! pub struct TestApp { .. }
//! impl TestApp {
//!     pub async fn new(pool: sqlx::PgPool) -> TestApp;
//!     pub async fn with_parts(pool: sqlx::PgPool, upstream: Arc<dyn Upstream>, clock: Arc<dyn Clock>) -> TestApp;
//!     pub async fn with_google(pool: sqlx::PgPool, google: GoogleConfig) -> TestApp;
//!     pub async fn get(&self, path: &str, cookie: Option<&str>) -> TestResponse;
//!     pub async fn delete(&self, path: &str, cookie: Option<&str>) -> TestResponse;
//!     pub async fn post(&self, path: &str, body: serde_json::Value, cookie: Option<&str>) -> TestResponse;
//!     pub async fn patch(&self, path: &str, body: serde_json::Value, cookie: Option<&str>) -> TestResponse;
//!     pub async fn post_raw(&self, path: &str, content_type: &str, body: &str, cookie: Option<&str>) -> TestResponse;
//!     pub async fn student(&self, username: &str) -> String;
//! }
//! pub struct TestResponse {
//!     pub status: axum::http::StatusCode,
//!     pub headers: axum::http::HeaderMap,
//!     pub body: serde_json::Value,
//!     pub text: String,
//! }
//! impl TestResponse {
//!     pub fn set_cookie_header(&self) -> Option<String>;
//!     pub fn cookie(&self) -> Option<String>;
//!     pub fn set_cookies(&self) -> Vec<String>;
//!     pub fn cookie_named(&self, name: &str) -> Option<String>;
//! }
//! ```

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]
// One shared harness: each test binary uses only the part of this API it needs.
#![allow(
    dead_code,
    reason = "one shared harness, every test binary uses part of it"
)]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use belajar_ports::clock::{Clock, SystemClock};
use belajar_ports::upstream::Upstream;
use belajar_server::config::GoogleConfig;
use belajar_server::{Config, Deps, app, upstream_stub::UpstreamStub};
use belajar_store_pg::Db;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

fn test_config() -> Config {
    Config {
        database_url: String::new(),
        addr: SocketAddr::from(([127, 0, 0, 1], 0)),
        web_dir: std::env::temp_dir().join("belajar-test-no-web"),
        cookie_secure: false,
        upstream_url: None,
        upstream_keys: None,
        upstream_timeout_ms: 60_000,
        google: None,
    }
}

pub struct TestApp {
    router: Router,
}

impl TestApp {
    /// The full app on a fresh database from `#[sqlx::test]`, with a stub upstream and the
    /// system clock. Cookie: Secure off.
    pub async fn new(pool: sqlx::PgPool) -> TestApp {
        Self::with_parts(pool, Arc::new(UpstreamStub), Arc::new(SystemClock)).await
    }

    /// The same with Google sign-in configured (endpoints usually on an in-process fake).
    pub async fn with_google(pool: sqlx::PgPool, google: GoogleConfig) -> TestApp {
        let config = Config {
            google: Some(google),
            ..test_config()
        };
        Self::build(pool, config, Arc::new(UpstreamStub), Arc::new(SystemClock))
    }

    /// The same with a chosen upstream and clock.
    pub async fn with_parts(
        pool: sqlx::PgPool,
        upstream: Arc<dyn Upstream>,
        clock: Arc<dyn Clock>,
    ) -> TestApp {
        Self::build(pool, test_config(), upstream, clock)
    }

    fn build(
        pool: sqlx::PgPool,
        config: Config,
        upstream: Arc<dyn Upstream>,
        clock: Arc<dyn Clock>,
    ) -> TestApp {
        TestApp {
            router: app(
                config,
                Deps {
                    db: Db::from_pool(pool),
                    upstream,
                    clock,
                },
            ),
        }
    }

    async fn call(
        &self,
        method: Method,
        path: &str,
        content_type: Option<&str>,
        body: Option<String>,
        cookie: Option<&str>,
        extra_headers: &[(&str, &str)],
    ) -> TestResponse {
        let mut builder = axum::http::Request::builder().method(method).uri(path);
        if let Some(kind) = content_type {
            builder = builder.header(header::CONTENT_TYPE, kind);
        }
        if let Some(value) = cookie {
            builder = builder.header(header::COOKIE, value);
        }
        for (name, value) in extra_headers {
            builder = builder.header(*name, *value);
        }
        let body = Body::from(body.unwrap_or_default());
        let response = self
            .router
            .clone()
            .oneshot(builder.body(body).expect("the request builds"))
            .await
            .expect("the router answers");

        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body reads")
            .to_bytes();
        // An SSE stream is read to its end by `collect` above.
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let parsed = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        TestResponse {
            status,
            headers,
            body: parsed,
            text,
        }
    }

    pub async fn get(&self, path: &str, cookie: Option<&str>) -> TestResponse {
        self.call(Method::GET, path, None, None, cookie, &[]).await
    }

    pub async fn delete(&self, path: &str, cookie: Option<&str>) -> TestResponse {
        self.call(Method::DELETE, path, None, None, cookie, &[])
            .await
    }

    pub async fn post(&self, path: &str, body: Value, cookie: Option<&str>) -> TestResponse {
        self.call(
            Method::POST,
            path,
            Some("application/json"),
            Some(body.to_string()),
            cookie,
            &[],
        )
        .await
    }

    pub async fn patch(&self, path: &str, body: Value, cookie: Option<&str>) -> TestResponse {
        self.call(
            Method::PATCH,
            path,
            Some("application/json"),
            Some(body.to_string()),
            cookie,
            &[],
        )
        .await
    }

    /// A POST with a raw body and content type.
    pub async fn post_raw(
        &self,
        path: &str,
        content_type: &str,
        body: &str,
        cookie: Option<&str>,
    ) -> TestResponse {
        self.call(
            Method::POST,
            path,
            Some(content_type),
            Some(body.to_owned()),
            cookie,
            &[],
        )
        .await
    }

    /// Registers `username` (password "rahasia-123", nickname = username, grade 7) and returns
    /// the value for a Cookie header: "belajar_session=<token>".
    pub async fn student(&self, username: &str) -> String {
        let res = self
            .post(
                "/v1/auth/register",
                json!({
                    "username": username,
                    "password": "rahasia-123",
                    "nickname": username,
                    "grade": 7,
                }),
                None,
            )
            .await;
        assert_eq!(
            res.status,
            StatusCode::CREATED,
            "the harness student exists"
        );
        res.cookie().expect("registering sets a session cookie")
    }

    /// A GET with extra request headers (used by `tests/http.rs` for `X-Request-Id`).
    pub async fn get_with_headers(
        &self,
        path: &str,
        cookie: Option<&str>,
        headers: &[(&str, &str)],
    ) -> TestResponse {
        self.call(Method::GET, path, None, None, cookie, headers)
            .await
    }
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    /// Parsed JSON, or `Value::Null` when the body is not JSON.
    pub body: Value,
    /// The raw body (an SSE stream is read to its end).
    pub text: String,
}

impl TestResponse {
    /// The whole first Set-Cookie header, if any.
    pub fn set_cookie_header(&self) -> Option<String> {
        self.headers
            .get(header::SET_COOKIE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    }

    /// "belajar_session=<token>" from Set-Cookie, if it sets a non-empty session.
    pub fn cookie(&self) -> Option<String> {
        let header = self.set_cookie_header()?;
        let rest = header.strip_prefix("belajar_session=")?;
        let token = rest.split(';').next()?.trim();
        if token.is_empty() {
            return None;
        }
        Some(format!("belajar_session={token}"))
    }

    /// Every Set-Cookie header, in order.
    pub fn set_cookies(&self) -> Vec<String> {
        self.headers
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(str::to_owned)
            .collect()
    }

    /// "<name>=<value>" from the Set-Cookie headers, if one sets a non-empty `name`.
    pub fn cookie_named(&self, name: &str) -> Option<String> {
        self.set_cookies().iter().find_map(|header| {
            let value = header
                .strip_prefix(&format!("{name}="))?
                .split(';')
                .next()?
                .trim();
            (!value.is_empty()).then(|| format!("{name}={value}"))
        })
    }

    /// A header value, for assertions.
    pub fn header(&self, name: &str) -> Option<String> {
        self.headers
            .get(name)
            .and_then(|v: &HeaderValue| v.to_str().ok())
            .map(str::to_owned)
    }
}
