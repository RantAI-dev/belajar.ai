//! Unversioned process routes: `/health` (the process is up) and `/ready` (Postgres answers).
//! Neither needs credentials. See `docs/conventions.md` on why they live outside `/v1`.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct Health {
    status: &'static str,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    checks: Option<Checks>,
}

#[derive(Serialize, ToSchema)]
struct Checks {
    postgres: &'static str,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
}

#[utoipa::path(get, path = "/health", responses((status = 200, body = Health)))]
pub(crate) async fn health() -> Response {
    (StatusCode::OK, Json(Health { status: "ok" })).into_response()
}

#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, description = "Postgres answers"),
        (status = 503, description = "Postgres is down", body = Ready),
    )
)]
pub(crate) async fn ready(state: State<AppState>) -> Response {
    if state.db.ping().await.is_ok() {
        (
            StatusCode::OK,
            Json(Ready {
                status: "ok",
                checks: None,
            }),
        )
            .into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(Ready {
                status: "degraded",
                checks: Some(Checks { postgres: "failed" }),
            }),
        )
            .into_response()
    }
}
