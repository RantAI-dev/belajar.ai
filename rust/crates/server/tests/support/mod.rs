//! Test harness for the server's HTTP tests. The foundation slice replaces this stub with a real
//! implementation of exactly this API (the acceptance tests depend on it):
//!
//! ```text
//! pub struct TestApp { .. }
//! impl TestApp {
//!     /// The full app on a fresh database from #[sqlx::test], with a stub upstream and the
//!     /// system clock. Cookie: Secure off.
//!     pub async fn new(pool: sqlx::PgPool) -> TestApp;
//!     /// The same with a chosen upstream and clock.
//!     pub async fn with_parts(pool: sqlx::PgPool, upstream: Arc<dyn Upstream>, clock: Arc<dyn Clock>) -> TestApp;
//!     pub async fn get(&self, path: &str, cookie: Option<&str>) -> TestResponse;
//!     pub async fn delete(&self, path: &str, cookie: Option<&str>) -> TestResponse;
//!     pub async fn post(&self, path: &str, body: serde_json::Value, cookie: Option<&str>) -> TestResponse;
//!     pub async fn patch(&self, path: &str, body: serde_json::Value, cookie: Option<&str>) -> TestResponse;
//!     /// A POST with a raw body and content type.
//!     pub async fn post_raw(&self, path: &str, content_type: &str, body: &str, cookie: Option<&str>) -> TestResponse;
//!     /// Registers `username` (password "rahasia-123", nickname = username, grade 7) and returns
//!     /// the value for a Cookie header: "belajar_session=<token>".
//!     pub async fn student(&self, username: &str) -> String;
//! }
//! pub struct TestResponse {
//!     pub status: axum::http::StatusCode,
//!     pub headers: axum::http::HeaderMap,
//!     /// Parsed JSON, or Value::Null when the body is not JSON.
//!     pub body: serde_json::Value,
//!     /// The raw body (an SSE stream is read to its end).
//!     pub text: String,
//! }
//! impl TestResponse {
//!     /// The whole first Set-Cookie header, if any.
//!     pub fn set_cookie_header(&self) -> Option<String>;
//!     /// "belajar_session=<token>" from Set-Cookie, if it sets a non-empty session.
//!     pub fn cookie(&self) -> Option<String>;
//! }
//! ```
