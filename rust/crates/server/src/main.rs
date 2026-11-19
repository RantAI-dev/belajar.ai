use std::process::ExitCode;

use belajar_ports::clock::SystemClock;
use belajar_ports::upstream::Upstream;
use belajar_server::{Config, Deps, app, upstream_stub::UpstreamStub};
use belajar_store_pg::Db;
use belajar_upstream::{HttpUpstream, UpstreamConfig, UpstreamKeys};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("belajar-server: {message}");
            return ExitCode::FAILURE;
        }
    };
    let db = match Db::connect(&config.database_url).await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!(code = "startup", error = %err, "cannot connect to the database");
            return ExitCode::FAILURE;
        }
    };
    if let Err(err) = db.migrate().await {
        tracing::error!(code = "startup", error = %err, "migrations failed");
        return ExitCode::FAILURE;
    }
    let upstream = match upstream_from(&config) {
        Ok(upstream) => upstream,
        Err(message) => {
            eprintln!("belajar-server: {message}");
            return ExitCode::FAILURE;
        }
    };
    let addr = config.addr;
    let router = app(
        config,
        Deps {
            db,
            upstream,
            clock: Arc::new(SystemClock),
        },
    );

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!(code = "startup", error = %err, %addr, "cannot listen");
            return ExitCode::FAILURE;
        }
    };
    tracing::info!(%addr, "listening");
    let serve = axum::serve(listener, router).with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    });
    if let Err(err) = serve.await {
        tracing::error!(code = "startup", error = %err, "server stopped");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// The real client when `BELAJAR_UPSTREAM_URL` is set; otherwise a stub whose every call answers
/// 503, so the rest of the app still runs. `Err` names the variable at fault.
fn upstream_from(config: &Config) -> Result<Arc<dyn Upstream>, String> {
    let Some(base_url) = config.upstream_url.clone() else {
        tracing::warn!("BELAJAR_UPSTREAM_URL is not set: every upstream call answers 503");
        return Ok(Arc::new(UpstreamStub));
    };
    let keys = UpstreamKeys::parse(config.upstream_keys.as_deref().unwrap_or(""))
        .map_err(|err| format!("BELAJAR_UPSTREAM_KEYS is not valid: {err}"))?;
    let client = HttpUpstream::new(UpstreamConfig {
        base_url,
        keys,
        timeout: std::time::Duration::from_millis(config.upstream_timeout_ms),
    })
    .map_err(|_| "BELAJAR_UPSTREAM_URL is not a valid URL.".to_owned())?;
    Ok(Arc::new(client))
}
