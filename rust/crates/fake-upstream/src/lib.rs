//! An in-repo fake of the upstream learning API ("Elise"), for development, tests and the demo —
//! the real one is on a private network nobody here can reach.
//!
//! The library exposes [`router`] (any axum host can serve it) and [`spawn`] (a disposable server
//! on a free port). The `belajar-fake-upstream` binary serves the same router with JSON logging.

mod bank;
mod data;
mod grade;
mod handlers;
mod recorded;

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use tokio::sync::mpsc;

/// How the fake paces its streams.
#[derive(Debug, Clone, Copy)]
pub struct FakeConfig {
    /// Pause before each streamed event: `Duration::ZERO` for tests, ~700 ms so the demo shows
    /// live progress.
    pub step_delay: Duration,
    /// A `: ping` comment at least this often while a stream is open. Kept by the client's
    /// timeout, never shown to a student.
    pub ping_every: Duration,
}

impl Default for FakeConfig {
    fn default() -> Self {
        Self {
            step_delay: Duration::ZERO,
            ping_every: Duration::from_secs(10),
        }
    }
}

/// The whole fake: one router, the page images unauthenticated, the API behind a fake key.
pub fn router(config: FakeConfig, base_url: String) -> Router {
    let state = AppState {
        config,
        base_url: Arc::from(base_url),
    };
    let api = Router::new()
        .route("/api/v1/mapel", get(handlers::mapel))
        .route("/api/v1/chat/completions", post(handlers::chat))
        .route(
            "/api/v1/practice/generate",
            post(handlers::generate_practice),
        )
        .route("/api/v1/practice/grade", post(grade::grade_practice))
        .layer(axum::middleware::from_fn(require_fake_key))
        .with_state(state.clone());
    // Page URLs are pre-signed by the upstream; the fake serves them without a key.
    let pages = Router::new()
        .route("/pages/{document_id}/{page}", get(handlers::page_image))
        .with_state(state);
    api.merge(pages)
}

#[derive(Clone)]
struct AppState {
    config: FakeConfig,
    base_url: Arc<str>,
}

/// Starts the fake on `127.0.0.1:<a free port>`; it stops when the returned server is dropped.
pub async fn spawn(config: FakeConfig) -> std::io::Result<FakeServer> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let base_url = format!("http://{addr}");
    let app = router(config, base_url.clone());
    let (shutdown, mut shutdown_rx) = tokio::sync::watch::channel(());
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                // Resolves when the sender fires, or when it is dropped at process end.
                let _ = shutdown_rx.changed().await;
            })
            .await;
    });
    Ok(FakeServer { base_url, shutdown })
}

/// A running fake. Use `base_url` as the upstream's `base_url`.
pub struct FakeServer {
    pub base_url: String,
    shutdown: tokio::sync::watch::Sender<()>,
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        // `drop` is all a test has, so the signal is best-effort; every server binds its own
        // port, so a slow task cannot get in another test's way.
        let _ = self.shutdown.send(());
    }
}

/// Any bearer key starting with `fake-` opens the API; anything else is a 401 like the real one.
async fn require_fake_key(request: Request, next: Next) -> Response {
    let authorized = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.strip_prefix("Bearer fake-").is_some());
    if !authorized {
        return handlers::error_json(
            StatusCode::UNAUTHORIZED,
            "the fake only accepts keys that start with 'fake-'",
            "invalid_api_key",
        );
    }
    next.run(request).await
}

/// Streams prepared SSE frames with a pause before each and a `: ping` while it waits.
pub(crate) fn sse_body(
    frames: Vec<Bytes>,
    step_delay: Duration,
    ping_every: Duration,
) -> axum::body::Body {
    let (tx, rx) = mpsc::channel::<Result<Bytes, Infallible>>(16);
    tokio::spawn(async move {
        let mut last_ping = tokio::time::Instant::now();
        for frame in frames {
            if delay_with_pings(step_delay, ping_every, &mut last_ping, &tx)
                .await
                .is_err()
            {
                return;
            }
            if tx.send(Ok(frame)).await.is_err() {
                return; // the consumer hung up
            }
        }
    });
    axum::body::Body::from_stream(tokio_stream::wrappers::ReceiverStream::new(rx))
}

async fn delay_with_pings(
    delay: Duration,
    ping_every: Duration,
    last_ping: &mut tokio::time::Instant,
    tx: &mpsc::Sender<Result<Bytes, Infallible>>,
) -> Result<(), ()> {
    const PING: &[u8] = b": ping\n\n";
    if ping_every.is_zero() {
        tokio::time::sleep(delay).await;
        return Ok(());
    }
    let start = tokio::time::Instant::now();
    loop {
        let now = tokio::time::Instant::now();
        let next_ping = *last_ping + ping_every;
        if next_ping <= now {
            if tx.send(Ok(Bytes::from_static(PING))).await.is_err() {
                return Err(());
            }
            *last_ping = now;
            continue;
        }
        if now >= start + delay {
            return Ok(());
        }
        tokio::time::sleep_until(next_ping.min(start + delay)).await;
    }
}

/// An SSE response: the frames as the body, with the SSE headers.
pub(crate) fn sse_response(body: axum::body::Body) -> Response {
    let mut response = Response::new(body);
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
}

/// A bare `{"error": ...}` JSON with a status, like the upstream's own error responses.
pub(crate) fn error_response(status: StatusCode, message: &str, code: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "error": { "message": message, "type": "fake_upstream", "code": code }
        })),
    )
        .into_response()
}

/// One `event: <name>` frame. A data-only frame is an event named "message" on the wire.
pub(crate) fn event_frame(name: &str, payload: &serde_json::Value) -> Bytes {
    Bytes::from(format!("event: {name}\ndata: {payload}\n\n"))
}

/// One `data: <payload>` frame.
pub(crate) fn data_frame(payload: &serde_json::Value) -> Bytes {
    Bytes::from(format!("data: {payload}\n\n"))
}
