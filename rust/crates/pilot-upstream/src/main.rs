//! Runs the pilot upstream. Configuration: see `config.rs` (all `PILOT_*` variables).

use std::sync::Arc;

use belajar_pilot_upstream::config::Config;
use belajar_pilot_upstream::{App, router};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("pilot-upstream: {error}");
            std::process::exit(2);
        }
    };
    let addr = config.addr.clone();
    let app = match App::new(config) {
        Ok(app) => Arc::new(app),
        Err(error) => {
            eprintln!("pilot-upstream: {error}");
            std::process::exit(2);
        }
    };
    // Warm the list of bases; a failure here is retried on the first request.
    if let Err(error) = app.catalog().await {
        tracing::warn!(%error, "the knowledge bases could not be listed yet");
    }
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("pilot-upstream: cannot listen on {addr}: {error}");
            std::process::exit(1);
        }
    };
    tracing::info!(%addr, "listening");
    let shutdown = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    if let Err(error) = axum::serve(listener, router(app))
        .with_graceful_shutdown(shutdown)
        .await
    {
        eprintln!("pilot-upstream: {error}");
        std::process::exit(1);
    }
}
