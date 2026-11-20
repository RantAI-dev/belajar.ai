pub(crate) mod auth;
pub(crate) mod google_auth;
pub(crate) mod google_link;
pub(crate) mod google_signup;
pub(crate) mod health;
pub(crate) mod home;
pub(crate) mod lessons;
pub(crate) mod me;
pub(crate) mod openapi;
pub(crate) mod practice;
pub(crate) mod progress;
pub(crate) mod progress_dto;
pub(crate) mod root;
pub(crate) mod subjects;
pub(crate) mod tanya;
pub(crate) mod tanya_messages;
pub(crate) mod tanya_pages;

use axum::Router;
use axum::extract::Extension;
use axum::response::Response;
use axum::routing::{get, post};

use crate::RequestId;
use crate::api_error::ApiError;
use crate::error_codes::ErrorCode;
use crate::state::AppState;

pub fn v1() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::v1_health))
        .route("/openapi.json", get(openapi::openapi_json))
        .route("/me", get(me::get_me).patch(me::patch_me))
        .route("/subjects", get(subjects::list_subjects))
        .route(
            "/subjects/{subjectId}/chapters",
            get(subjects::list_chapters),
        )
        .route("/home", get(home::get_home))
        .route("/progress", get(progress::get_progress))
        .route("/lessons", post(lessons::create_lesson))
        .route(
            "/lessons/{id}",
            get(lessons::get_lesson).patch(lessons::patch_lesson),
        )
        .route("/lessons/{id}/complete", post(lessons::complete_lesson))
        .route(
            "/conversations",
            get(tanya::list_conversations).post(tanya::create_conversation),
        )
        .route(
            "/conversations/{id}",
            get(tanya::get_conversation).patch(tanya::patch_conversation),
        )
        .route(
            "/conversations/{id}/messages",
            get(tanya_messages::list_messages).post(tanya_messages::post_message),
        )
        .route(
            "/messages/{message_id}/sources/{number}/page",
            get(tanya_pages::message_source_page),
        )
        .merge(auth::routes())
        .merge(google_auth::routes())
        .merge(practice::routes())
        .fallback(not_found)
}

/// Unknown `/v1` paths answer with the standard error body, not the router's plain-text 404.
pub async fn not_found(Extension(id): Extension<RequestId>) -> Response {
    ApiError::new(ErrorCode::NotFound).into_response(&id.0)
}
