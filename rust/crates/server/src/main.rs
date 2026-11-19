use std::process::ExitCode;

use belajar_ports::clock::SystemClock;
use belajar_server::{Config, Deps, app, upstream_stub::UpstreamStub};
use belajar_store_pg::Db;
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

    // The upstream slice replaces this stub with the real client; until then every upstream
    // call answers 503 `unavailable`.
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
    let addr = config.addr;
    let router = app(
        config,
        Deps {
            db,
            upstream: Arc::new(UpstreamStub),
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
