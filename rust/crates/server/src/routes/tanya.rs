//! `tanya`: the student's conversations — create, read, rename and list. The message routes
//! live in `tanya_messages`, the page preview in `tanya_pages`; the wire DTOs they share live
//! here, because every response about a conversation or a message uses them.

use axum::Extension;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use belajar_core::subject;
use belajar_ports::conversations::{Conversation, ConversationMode, Message, MessageRole};
use belajar_ports::upstream::Source;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody, FieldError};
use crate::current_student::CurrentStudent;
use crate::state::AppState;
use crate::store_errors;
use crate::wire::cursor;
use crate::wire::extract::{ApiJson, ApiQuery};
use crate::wire::instant;

/// The title a conversation starts with, before its first question names it.
pub(crate) const NEW_CONVERSATION_TITLE: &str = "Percakapan baru";

/// How much of a conversation's history one page shows.
pub(crate) const CONVERSATION_PAGE_MAX: u32 = 50;
/// How much of a conversation's messages one page shows.
pub(crate) const MESSAGE_PAGE_MAX: u32 = 100;
/// How many earlier messages the memory window re-sends to the upstream.
pub(crate) const MEMORY_WINDOW: u32 = 10;

// ===== wire types ==================================================================================================

/// `ask` is a Tanya conversation, `learn` a Belajar chapter session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConversationModeDto {
    Ask,
    Learn,
}

impl From<ConversationMode> for ConversationModeDto {
    fn from(mode: ConversationMode) -> Self {
        match mode {
            ConversationMode::Ask => Self::Ask,
            ConversationMode::Learn => Self::Learn,
        }
    }
}

/// Who wrote a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MessageRoleDto {
    User,
    Assistant,
}

impl From<MessageRole> for MessageRoleDto {
    fn from(role: MessageRole) -> Self {
        match role {
            MessageRole::User => Self::User,
            MessageRole::Assistant => Self::Assistant,
        }
    }
}

/// One conversation, as every conversation response shows it.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConversationDto {
    pub id: Uuid,
    pub mode: ConversationModeDto,
    /// Our subject id ("ipa", "mtk", ...) or `null` for "all subjects".
    pub subject_id: Option<String>,
    pub chapter_id: Option<String>,
    pub title: String,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl From<Conversation> for ConversationDto {
    fn from(conversation: Conversation) -> Self {
        Self {
            id: conversation.id,
            mode: conversation.mode.into(),
            subject_id: conversation.subject_id,
            chapter_id: conversation.chapter_id,
            title: conversation.title,
            created_at: conversation.created_at,
            updated_at: conversation.updated_at,
        }
    }
}

/// One book passage an answer used. The upstream's `page_url` stays on our server; the browser
/// fetches the page through `pagePreviewUrl` instead.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceDto {
    /// The citation number used in the answer text, e.g. `[1]`.
    pub number: u32,
    pub title: String,
    pub section: Option<String>,
    pub page: Option<u32>,
    pub is_cited: bool,
    /// `/v1/messages/{messageId}/sources/{number}/page` when the page image exists, else `null`.
    pub page_preview_url: Option<String>,
    /// When `pagePreviewUrl` serves the whole book (a PDF) rather than one page: the 1-based page
    /// of that file the source cites. `null` when the preview is the page itself.
    pub preview_page: Option<u32>,
}

/// The message's own id, for the sources' preview URLs.
impl SourceDto {
    fn from_source(message_id: Uuid, source: &Source) -> Self {
        Self {
            number: source.number,
            title: source.title.clone(),
            section: source.section.clone(),
            page: source.page,
            is_cited: source.cited,
            page_preview_url: source
                .page_url
                .as_ref()
                .map(|_| format!("/v1/messages/{message_id}/sources/{}/page", source.number)),
            preview_page: source.page_url.as_deref().and_then(pdf_page_fragment),
        }
    }
}

/// The upstream links a whole PDF with `#page=<n>` (1-based, as PDF viewers read it); a page
/// image has no fragment. The fragment is the only place that page number is given.
pub(crate) fn pdf_page_fragment(page_url: &str) -> Option<u32> {
    let (_, fragment) = page_url.split_once('#')?;
    fragment
        .split('&')
        .find_map(|part| part.strip_prefix("page="))
        .and_then(|n| n.parse::<u32>().ok())
        .filter(|n| *n > 0)
}

/// One stored message: the student's question or the assistant's answer.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageDto {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: MessageRoleDto,
    pub content: String,
    pub sources: Vec<SourceDto>,
    /// `null` on user messages.
    pub is_grounded: Option<bool>,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl From<Message> for MessageDto {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            conversation_id: message.conversation_id,
            role: message.role.into(),
            content: message.content,
            sources: message
                .sources
                .iter()
                .map(|source| SourceDto::from_source(message.id, source))
                .collect(),
            is_grounded: message.is_grounded,
            created_at: message.created_at,
        }
    }
}

/// Every list response: rows plus the cursor the next page needs, `null` on the last page.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConversationListDto {
    pub data: Vec<ConversationDto>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageListDto {
    pub data: Vec<MessageDto>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateConversationRequest {
    /// `null` for "all subjects".
    pub subject_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenameConversationRequest {
    pub title: String,
}

// ===== shared validation ============================================================================================

/// One bad field. The `code` vocabulary is fixed: `required`, `too_short`, `too_long`,
/// `invalid_format`, `out_of_range`.
fn bad_field(field: &str, code: &'static str, message: String) -> FieldError {
    FieldError {
        field: field.into(),
        code,
        message,
    }
}

/// The length rule the slice's text fields share, measured in characters.
pub(crate) fn bad_length(
    errors: &mut Vec<FieldError>,
    field: &str,
    len: usize,
    min: usize,
    max: usize,
) {
    if len < min {
        errors.push(bad_field(
            field,
            "too_short",
            format!("Must be at least {min} characters."),
        ));
    } else if len > max {
        errors.push(bad_field(
            field,
            "too_long",
            format!("Must be at most {max} characters."),
        ));
    }
}

/// The wire rule every list shares: a default and a maximum, a larger value is lowered to the
/// maximum, anything below one is refused. Field names are bare query parameter names.
pub(crate) fn query_limit(
    raw: Option<&str>,
    default: u32,
    max: u32,
    errors: &mut Vec<FieldError>,
) -> u32 {
    let Some(raw) = raw else {
        return default;
    };
    match raw.parse::<i64>() {
        Ok(value) if value < 1 => {
            errors.push(bad_field(
                "limit",
                "out_of_range",
                format!("Must be between 1 and {max}."),
            ));
            default
        }
        Ok(value) => u32::try_from(value).unwrap_or(max).min(max),
        Err(_) => {
            errors.push(bad_field(
                "limit",
                "invalid_format",
                format!("Must be a whole number between 1 and {max}."),
            ));
            default
        }
    }
}

/// A cursor we issued, or a 400 error entry when the text is not one.
pub(crate) fn query_cursor(
    raw: Option<&str>,
    errors: &mut Vec<FieldError>,
) -> Option<belajar_ports::conversations::PageCursor> {
    let raw = raw?;
    match cursor::decode(raw) {
        Some(decoded) => Some(decoded),
        None => {
            errors.push(bad_field(
                "cursor",
                "invalid_format",
                "Must be a cursor from a previous page.".into(),
            ));
            None
        }
    }
}

/// The conversation a path names, or the standard 404 the same missing id gets.
pub(crate) fn conversation_id(raw: &str) -> Option<Uuid> {
    Uuid::parse_str(raw).ok()
}

/// The standard 404: the same body for a missing row and another student's row.
pub(crate) fn not_found(id: &RequestId) -> Response {
    ApiError::new(crate::error_codes::ErrorCode::NotFound).into_response(&id.0)
}

// ===== handlers ====================================================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationsQuery {
    mode: Option<String>,
    limit: Option<String>,
    cursor: Option<String>,
}

/// Starts a conversation, optionally about one subject.
#[utoipa::path(
    post,
    path = "/v1/conversations",
    request_body = CreateConversationRequest,
    responses(
        (status = 201, description = "The new conversation", body = ConversationDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn create_conversation(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    ApiJson(req): ApiJson<CreateConversationRequest>,
) -> Response {
    let mut errors = Vec::new();
    if let Some(subject_id) = &req.subject_id
        && subject::by_id(subject_id).is_none()
    {
        errors.push(bad_field(
            "/subjectId",
            "invalid_format",
            "Must be one of the known subject ids.".into(),
        ));
    }
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    let now = state.clock.now();
    match state
        .conversations
        .create_conversation(
            student.student_id,
            ConversationMode::Ask,
            req.subject_id.as_deref(),
            None,
            NEW_CONVERSATION_TITLE,
            now,
        )
        .await
    {
        Ok(conversation) => (
            StatusCode::CREATED,
            Json(ConversationDto::from(conversation)),
        )
            .into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// One conversation.
#[utoipa::path(
    get,
    path = "/v1/conversations/{id}",
    params(("id" = Uuid, Path, description = "The conversation")),
    responses(
        (status = 200, body = ConversationDto),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
    )
)]
pub(crate) async fn get_conversation(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    axum::extract::Path(raw): axum::extract::Path<String>,
) -> Response {
    let Some(conversation_id) = conversation_id(&raw) else {
        return not_found(&id);
    };
    match state
        .conversations
        .get_conversation(student.student_id, conversation_id)
        .await
    {
        Ok(conversation) => {
            (StatusCode::OK, Json(ConversationDto::from(conversation))).into_response()
        }
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// Renames a conversation. A later message never overwrites the new title.
#[utoipa::path(
    patch,
    path = "/v1/conversations/{id}",
    request_body = RenameConversationRequest,
    params(("id" = Uuid, Path, description = "The conversation")),
    responses(
        (status = 200, description = "The renamed conversation", body = ConversationDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
    )
)]
pub(crate) async fn patch_conversation(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    axum::extract::Path(raw): axum::extract::Path<String>,
    ApiJson(req): ApiJson<RenameConversationRequest>,
) -> Response {
    let title = req.title.trim();
    let mut errors = Vec::new();
    bad_length(&mut errors, "/title", title.chars().count(), 1, 80);
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    let Some(conversation_id) = conversation_id(&raw) else {
        return not_found(&id);
    };
    match state
        .conversations
        .rename_conversation(student.student_id, conversation_id, title)
        .await
    {
        Ok(conversation) => {
            (StatusCode::OK, Json(ConversationDto::from(conversation))).into_response()
        }
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// The student's conversations, newest activity first.
#[utoipa::path(
    get,
    path = "/v1/conversations",
    params(
        ("mode" = Option<String>, Query, description = "`ask` or `learn`; both when absent"),
        ("limit" = Option<u32>, Query, description = "Default 20, at most 50"),
        ("cursor" = Option<String>, Query, description = "The nextCursor of a previous page"),
    ),
    responses(
        (status = 200, body = ConversationListDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn list_conversations(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    ApiQuery(query): ApiQuery<ConversationsQuery>,
) -> Response {
    let mut errors = Vec::new();
    let mode = match query.mode.as_deref() {
        None => None,
        Some("ask") => Some(ConversationMode::Ask),
        Some("learn") => Some(ConversationMode::Learn),
        Some(_) => {
            errors.push(bad_field(
                "mode",
                "invalid_format",
                "Must be `ask` or `learn`.".into(),
            ));
            None
        }
    };
    let limit = query_limit(
        query.limit.as_deref(),
        20,
        CONVERSATION_PAGE_MAX,
        &mut errors,
    );
    let after = query_cursor(query.cursor.as_deref(), &mut errors);
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    match state
        .conversations
        .list_conversations(student.student_id, mode, after, limit)
        .await
    {
        Ok(page) => (
            StatusCode::OK,
            Json(ConversationListDto {
                data: page.data.into_iter().map(ConversationDto::from).collect(),
                next_cursor: page.next_cursor.map(cursor::encode),
            }),
        )
            .into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_conversation_dto_is_camel_case_and_holds_no_upstream_url() {
        let conversation = Conversation {
            id: Uuid::now_v7(),
            student_id: Uuid::now_v7(),
            mode: ConversationMode::Ask,
            subject_id: Some("ipa".into()),
            chapter_id: None,
            title: "Kenapa rel kereta api diberi celah?".into(),
            created_at: DateTime::parse_from_rfc3339("2026-11-19T08:30:00.000Z")
                .unwrap()
                .into(),
            updated_at: DateTime::parse_from_rfc3339("2026-11-19T08:30:00.000Z")
                .unwrap()
                .into(),
        };
        let body = json!(ConversationDto::from(conversation));
        assert!(body.get("subjectId").is_some() && body.get("subject_id").is_none());
        assert_eq!(body["mode"], "ask");
        assert_eq!(body["createdAt"], "2026-11-19T08:30:00.000Z");
    }

    #[test]
    fn the_message_dto_points_previews_at_our_server_only() {
        let message = Message {
            id: Uuid::now_v7(),
            conversation_id: Uuid::now_v7(),
            student_id: Uuid::now_v7(),
            role: MessageRole::Assistant,
            content: "Rel memuai [1].".into(),
            sources: vec![
                Source {
                    number: 1,
                    title: "IPA Kelas VII".into(),
                    section: Some("Bab 5".into()),
                    document_id: "kb-ipa-7".into(),
                    page: Some(78),
                    cited: true,
                    page_url: Some("http://upstream/pages/kb-ipa-7/78.svg".into()),
                },
                Source {
                    number: 2,
                    title: "IPA Kelas VII".into(),
                    section: None,
                    document_id: "kb-ipa-7".into(),
                    page: None,
                    cited: false,
                    page_url: None,
                },
            ],
            is_grounded: Some(true),
            created_at: DateTime::parse_from_rfc3339("2026-11-19T08:30:00.000Z")
                .unwrap()
                .into(),
        };
        let id = message.id;
        let body = json!(MessageDto::from(message));
        let text = body.to_string();
        assert!(text.contains("pagePreviewUrl"));
        assert!(!text.contains("page_url") && !text.contains("documentId"));
        assert!(
            !text.contains("upstream"),
            "the upstream host never crosses the wire"
        );
        assert_eq!(
            body["sources"][0]["pagePreviewUrl"],
            format!("/v1/messages/{id}/sources/1/page")
        );
        assert!(body["sources"][1]["pagePreviewUrl"].is_null());
        assert!(
            body["sources"][0]["previewPage"].is_null(),
            "a page image needs no page"
        );
        assert_eq!(body["isGrounded"], true);
    }

    #[test]
    fn a_whole_pdf_preview_names_the_cited_page_from_the_upstream_fragment() {
        assert_eq!(
            pdf_page_fragment("http://u/api/v1/sources/page?t=abc.def#page=79"),
            Some(79)
        );
        assert_eq!(pdf_page_fragment("http://u/p?t=x#zoom=2&page=3"), Some(3));
        for url in [
            "http://u/pages/kb-ipa-7/78.svg?sig=t",
            "http://u/p#page=0",
            "http://u/p#page=abc",
            "http://u/p#section-2",
        ] {
            assert_eq!(pdf_page_fragment(url), None, "{url}");
        }
    }

    #[test]
    fn the_request_shapes_reject_unknown_fields() {
        let ok: CreateConversationRequest =
            serde_json::from_value(json!({ "subjectId": null })).unwrap();
        assert_eq!(ok.subject_id, None);
        assert!(
            serde_json::from_value::<CreateConversationRequest>(json!({ "chapter": 1 })).is_err()
        );
        let rename: RenameConversationRequest =
            serde_json::from_value(json!({ "title": "Bab baru" })).unwrap();
        assert_eq!(rename.title, "Bab baru");
    }

    #[test]
    fn limit_follows_the_list_rules() {
        let mut errors = Vec::new();
        assert_eq!(query_limit(None, 20, 50, &mut errors), 20);
        assert_eq!(query_limit(Some("2"), 20, 50, &mut errors), 2);
        assert_eq!(query_limit(Some("999"), 20, 50, &mut errors), 50, "lowered");
        for raw in ["0", "-1", "abc", "1.5"] {
            let mut errors = Vec::new();
            assert_eq!(query_limit(Some(raw), 20, 50, &mut errors), 20);
            assert_eq!(errors.len(), 1, "{raw} is refused");
            assert_eq!(errors[0].field, "limit");
        }
    }

    #[test]
    fn a_title_is_measured_after_trimming() {
        let mut errors = Vec::new();
        bad_length(&mut errors, "/title", "  ".trim().chars().count(), 1, 80);
        assert_eq!(errors[0].code, "too_short");
        let mut errors = Vec::new();
        bad_length(&mut errors, "/title", "x".repeat(81).chars().count(), 1, 80);
        assert_eq!(errors[0].code, "too_long");
        let mut errors = Vec::new();
        bad_length(&mut errors, "/title", " x ".trim().chars().count(), 1, 80);
        assert!(errors.is_empty(), "trimmed length is what counts");
    }
}
