//! `tanya`: the page preview behind a source. The upstream's signed `page_url` is stored with the
//! message and fetched by our server on demand, so the browser only ever talks to `/v1`.

use axum::Extension;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use belajar_ports::upstream::Source;
use uuid::Uuid;

use super::tanya::not_found;
use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody};
use crate::current_student::CurrentStudent;
use crate::error_codes::ErrorCode;
use crate::state::AppState;
use crate::store_errors;
use crate::upstream_errors;

/// How long a browser may keep a page image. The pages are book pages, but the URL is only
/// reachable with the student's session, so the cache stays private.
const PAGE_CACHE_CONTROL: &str = "private, max-age=3600";

/// The source with this citation number, and its signed page URL, which this endpoint fetches
/// and never forwards.
fn cited_page(sources: &[Source], number: u32) -> Option<&str> {
    sources
        .iter()
        .find(|source| source.number == number)
        .and_then(|source| source.page_url.as_deref())
}

/// The book page one source cites, with the upstream's own content type.
#[utoipa::path(
    get,
    path = "/v1/messages/{messageId}/sources/{number}/page",
    params(
        ("messageId" = Uuid, Path, description = "The message the source belongs to"),
        ("number" = u32, Path, description = "The source's citation number"),
    ),
    responses(
        (status = 200, description = "The page image bytes", content_type = "image/*"),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
        (status = 502, body = ErrorBody),
        (status = 503, body = ErrorBody),
        (status = 504, body = ErrorBody),
    )
)]
pub(crate) async fn message_source_page(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path((raw_message_id, raw_number)): Path<(String, String)>,
) -> Response {
    // A path that is not even the right shape answers the same as a missing row.
    let Some(message_id) = Uuid::parse_str(&raw_message_id).ok() else {
        return not_found(&id);
    };
    let Ok(number) = raw_number.parse::<u32>() else {
        return not_found(&id);
    };
    let message = match state
        .conversations
        .get_message(student.student_id, message_id)
        .await
    {
        Ok(message) => message,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    // An unknown number, or a source with no page image: the same 404 as a missing message.
    let Some(page_url) = cited_page(&message.sources, number) else {
        return not_found(&id);
    };
    let page = match state.upstream.page(page_url).await {
        Ok(page) => page,
        Err(err) => return upstream_errors::into_response(&err, &id.0),
    };
    let content_type = match HeaderValue::from_str(&page.content_type) {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(
                request_id = %id.0,
                code = "internal",
                error = %err,
                "the page image content type is not a valid header"
            );
            return ApiError::new(ErrorCode::Internal).into_response(&id.0);
        }
    };
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static(PAGE_CACHE_CONTROL),
            ),
        ],
        page.bytes,
    )
        .into_response()
}
