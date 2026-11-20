//! The HTTP server: routes, middleware and the static web app. Handlers stay thin: decode, call
//! one service, map the error through `api_error`.

pub mod api_error;
pub mod config;
pub mod cookies;
pub mod current_student;
pub mod error_codes;
pub mod google;
pub mod logging;
pub mod login_limiter;
pub mod openapi;
pub mod passwords;
mod request_id;
mod routes;
pub mod state;
pub mod store_errors;
pub mod tokens;
pub mod upstream_errors;
pub mod upstream_stub;
pub mod wire;

use axum::Router;
use axum::middleware;
use tower_http::services::{ServeDir, ServeFile};

pub use config::Config;
pub use request_id::RequestId;
pub use state::{AppState, Deps};

/// The whole app, ready to serve. `Deps` is everything outside the process; later slices add one
/// field to it and one store to `AppState`, here and nowhere else.
pub fn app(config: Config, deps: Deps) -> Router {
    let state = AppState::new(config.clone(), deps);
    let index = state.config.web_dir.join("index.html");
    let web = ServeDir::new(&state.config.web_dir).not_found_service(ServeFile::new(index));
    Router::new()
        .merge(routes::root::routes())
        .nest("/v1", routes::v1())
        .fallback_service(web)
        .layer(middleware::from_fn(logging::trace))
        .layer(middleware::from_fn(request_id::stamp))
        .with_state(state)
}
