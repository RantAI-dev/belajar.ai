//! The `belajar-fake-upstream` binary: the in-repo fake of the upstream learning API, for local
//! development and the demo. Listens on `FAKE_UPSTREAM_ADDR` (default `127.0.0.1:8090`), delays
//! each streamed step by `FAKE_UPSTREAM_STEP_DELAY_MS` (default 700, so the demo shows live
//! progress), and logs one JSON line per request.

use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::Request;
use axum::middleware::{self, Next};
use axum::response::Response;
use belajar_fake_upstream::{FakeConfig, router};
use serde_json::json;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = match std::env::var("FAKE_UPSTREAM_ADDR") {
        Ok(raw) => match raw.parse() {
            Ok(addr) => addr,
            Err(_) => fail("FAKE_UPSTREAM_ADDR", &raw),
        },
        Err(_) => SocketAddr::from(([127, 0, 0, 1], 8090)),
    };
    let step_delay_ms: u64 = match std::env::var("FAKE_UPSTREAM_STEP_DELAY_MS") {
        Ok(raw) => match raw.parse() {
            Ok(value) => value,
            Err(_) => fail("FAKE_UPSTREAM_STEP_DELAY_MS", &raw),
        },
        Err(_) => 700,
    };
    let config = FakeConfig {
        step_delay: Duration::from_millis(step_delay_ms),
        ..FakeConfig::default()
    };
    let base_url = format!("http://{addr}");
    let app = router(config, base_url.clone()).layer(middleware::from_fn(log_requests));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("fake-upstream: cannot listen on {addr}: {error}");
            std::process::exit(1);
        }
    };
    println!(
        "{}",
        json!({
            "ts": timestamp(),
            "level": "info",
            "msg": "listening",
            "addr": addr.to_string(),
            "base_url": base_url,
        })
    );
    if let Err(error) = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
    {
        eprintln!("fake-upstream: {error}");
        std::process::exit(1);
    }
}

/// One JSON log line per request: method, route, status, duration.
async fn log_requests(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let start = std::time::Instant::now();
    let response = next.run(request).await;
    println!(
        "{}",
        json!({
            "ts": timestamp(),
            "level": "info",
            "msg": "request",
            "method": method.as_str().to_owned(),
            "path": path,
            "status": response.status().as_u16(),
            "duration_ms": start.elapsed().as_millis() as u64,
        })
    );
    response
}

fn timestamp() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

/// A bad environment value stops the process, naming the variable, as the conventions require.
fn fail(name: &str, value: &str) -> ! {
    eprintln!("fake-upstream: {name} is not a valid value: {value}");
    std::process::exit(1);
}
