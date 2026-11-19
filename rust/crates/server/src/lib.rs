//! The HTTP server: routes, middleware and the static web app. Handlers stay thin: decode, call
//! one service, map the error through `api_error`.

pub mod api_error;
pub mod error_codes;
mod request_id;
mod routes;

use std::path::PathBuf;

use axum::Router;
use axum::middleware;
use tower_http::services::{ServeDir, ServeFile};

pub use request_id::RequestId;

#[derive(Debug, Clone)]
pub struct Config {
    /// The built web app (`web/dist`). Unknown paths outside `/v1` get its `index.html`, so the
    /// client-side router owns them.
    pub web_dir: PathBuf,
}

pub fn app(config: &Config) -> Router {
    let index = config.web_dir.join("index.html");
    let web = ServeDir::new(&config.web_dir).not_found_service(ServeFile::new(index));
    Router::new()
        .nest("/v1", routes::v1())
        .fallback_service(web)
        .layer(middleware::from_fn(request_id::stamp))
}
