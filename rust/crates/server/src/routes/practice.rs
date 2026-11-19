//! The Latihan routes: practice generation (SSE), the stored sets, submission with grading, the
//! book page behind a submitted answer, and topic mastery. This file is the hub: the routes
//! themselves live in `practice_generate`, `practice_submit`, `practice_views`, and the wire
//! types they share in `practice_dto`.

pub(crate) mod practice_dto;
pub(crate) mod practice_generate;
pub(crate) mod practice_submit;
pub(crate) mod practice_views;

use axum::Router;
use axum::response::Response;
use axum::routing::{get, post};

use crate::RequestId;
use crate::api_error::ApiError;
use crate::error_codes::ErrorCode;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/practice-sets",
            post(practice_generate::create_practice_set).get(practice_views::list_practice_sets),
        )
        .route("/practice-sets/{id}", get(practice_views::get_practice_set))
        .route(
            "/practice-sets/{id}/submit",
            post(practice_submit::submit_practice_set),
        )
        .route(
            "/practice-sets/{id}/questions/{questionId}/page",
            get(practice_views::question_page),
        )
        .route("/topic-mastery", get(practice_views::topic_mastery))
}

/// A bug on our side: one logged 500, its cause in the log, the registry's message in the body.
pub(crate) fn internal(id: &RequestId, cause: &str) -> Response {
    tracing::error!(request_id = %id.0, code = "internal", error = %cause, "latihan failed");
    ApiError::new(ErrorCode::Internal).into_response(&id.0)
}
