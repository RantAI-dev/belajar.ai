//! `tanya`: asking a question and watching the answer stream in, and browsing a conversation's
//! stored messages. One SSE stream per question: the stored question as a `message` event, one
//! `delta` per answer piece, then the stored answer as `done` — or one last `error` event, the
//! same body a JSON error response carries.

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{
    IntoResponse, Response,
    sse::{Event, KeepAlive, Sse},
};
use belajar_core::subject;
use belajar_ports::accounts::StudentAccount;
use belajar_ports::clock::Clock;
use belajar_ports::conversations::{
    Conversation, ConversationMode, ConversationStore, Message, MessageRole, NewMessage,
};
use belajar_ports::store::StoreError;
use belajar_ports::upstream::{
    Assistant, ChatEvent, ChatMessage, ChatRequest, ChatRole, ChatStream,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::tanya::{
    MEMORY_WINDOW, MESSAGE_PAGE_MAX, MessageDto, MessageListDto, bad_length, conversation_id,
    not_found, query_cursor, query_limit,
};
use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody};
use crate::current_student::CurrentStudent;
use crate::error_codes::ErrorCode;
use crate::state::AppState;
use crate::store_errors;
use crate::upstream_errors;
use crate::wire::cursor;
use crate::wire::extract::{ApiJson, ApiQuery};

// ===== wire types ==================================================================================================

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SendMessageRequest {
    /// 1–2000 characters, after trimming.
    pub content: String,
}

/// One piece of the answer as it streams in.
#[derive(Debug, Serialize, ToSchema)]
pub struct DeltaData {
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MessagesQuery {
    limit: Option<String>,
    cursor: Option<String>,
}

// ===== the memory the upstream sees =================================================================================

/// The assistant this conversation's mode answers with.
fn assistant_for(mode: ConversationMode) -> Assistant {
    match mode {
        ConversationMode::Ask => Assistant::Ask,
        ConversationMode::Learn => Assistant::Learn,
    }
}

fn chat_role(role: MessageRole) -> ChatRole {
    match role {
        MessageRole::User => ChatRole::User,
        MessageRole::Assistant => ChatRole::Assistant,
    }
}

/// A short Indonesian learning profile: who is asking, at which grade, about what, and to
/// answer only from the textbook, simply, for that grade.
fn profile_system_message(profile: &StudentAccount, conversation: &Conversation) -> String {
    let grade = profile.grade.get();
    let mut text = format!(
        "Kamu adalah guru pendamping untuk {}, siswa kelas {grade}.",
        profile.nickname
    );
    match conversation.subject_id.as_deref().and_then(subject::by_id) {
        Some(subject) => text.push_str(&format!(" Mapel: {}.", subject.name)),
        None => text.push_str(" Mapel: semua mapel."),
    }
    if conversation.mode == ConversationMode::Learn {
        text.push_str(&format!(" Bab: {}.", conversation.title));
    }
    text.push_str(&format!(
        " Jawab hanya berdasarkan buku pelajaran, dengan bahasa sederhana yang sesuai \
         untuk siswa kelas {grade}."
    ));
    text
}

/// The conversation's title from its first question: trimmed, at most 60 characters, cut at a
/// character boundary.
fn title_from_question(question: &str) -> String {
    question.trim().chars().take(60).collect()
}

/// The memory window: the profile, then the last stored messages (oldest first, without the new
/// question), then the new question as the final user message.
fn chat_messages(
    profile: &StudentAccount,
    conversation: &Conversation,
    history: &[Message],
    question: &str,
) -> Vec<ChatMessage> {
    let mut messages = Vec::with_capacity(history.len() + 2);
    messages.push(ChatMessage {
        role: ChatRole::System,
        content: profile_system_message(profile, conversation),
    });
    for message in history {
        messages.push(ChatMessage {
            role: chat_role(message.role),
            content: message.content.clone(),
        });
    }
    messages.push(ChatMessage {
        role: ChatRole::User,
        content: question.to_owned(),
    });
    messages
}

/// The `mapel` the upstream retrieves from: the subject's own upstream name, or all of them.
fn mapel_of(conversation: &Conversation) -> Option<String> {
    conversation
        .subject_id
        .as_deref()
        .and_then(subject::by_id)
        .map(|subject| subject.mapel.to_owned())
}

// ===== the answer stream ============================================================================================

/// One SSE event whose data is a single line of JSON. `None` only if serializing our own wire
/// type failed, which would be a bug on our side; the caller logs it and ends the stream.
fn json_event(name: &'static str, body: &impl Serialize) -> Option<Event> {
    let data = serde_json::to_string(body).ok()?;
    Some(Event::default().event(name).data(data))
}

type Storing = Pin<Box<dyn Future<Output = Result<Message, StoreError>> + Send>>;

/// Drives one answer: the stored question first, the upstream's deltas as they arrive, and the
/// stored answer once the upstream finishes. The assistant message is stored only then, so an
/// answer cut short by an error leaves nothing partial behind.
struct AnswerStream {
    first: Option<Event>,
    inner: ChatStream,
    storing: Option<Storing>,
    store: Arc<dyn ConversationStore>,
    clock: Arc<dyn Clock>,
    student_id: Uuid,
    conversation_id: Uuid,
    /// The answer so far; the stored message when the upstream finishes.
    buffer: String,
    request_id: String,
    ended: bool,
}

impl AnswerStream {
    /// One `delta` event per answer piece, in order.
    fn delta(text: &str) -> Poll<Option<Result<Event, Infallible>>> {
        match json_event(
            "delta",
            &DeltaData {
                text: text.to_owned(),
            },
        ) {
            Some(event) => Poll::Ready(Some(Ok(event))),
            None => Poll::Ready(None),
        }
    }

    /// The one last `error` event, the same body a JSON error response carries. `None` if even
    /// that could not be built, which is a bug the caller has logged.
    fn error(&self, api_error: &ApiError) -> Poll<Option<Result<Event, Infallible>>> {
        match json_event("error", &api_error.body(&self.request_id)) {
            Some(event) => Poll::Ready(Some(Ok(event))),
            None => Poll::Ready(None),
        }
    }

    /// The stored answer, or one last `error` if the store refused it.
    fn stored(
        &mut self,
        result: Result<Message, StoreError>,
    ) -> Poll<Option<Result<Event, Infallible>>> {
        match result {
            Ok(message) => match json_event("done", &MessageDto::from(message)) {
                Some(event) => Poll::Ready(Some(Ok(event))),
                None => Poll::Ready(None),
            },
            Err(err) => {
                // The answer is lost, so the question keeps no half-answer beside it.
                tracing::error!(
                    request_id = %self.request_id,
                    code = "internal",
                    error = %err,
                    "storing the answer failed"
                );
                self.error(&ApiError::new(ErrorCode::Internal))
            }
        }
    }
}

impl Stream for AnswerStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        // The stored question always arrives before any upstream byte.
        if let Some(event) = this.first.take() {
            return Poll::Ready(Some(Ok(event)));
        }
        if this.ended {
            return Poll::Ready(None);
        }
        if let Some(future) = this.storing.as_mut() {
            return match future.as_mut().poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(result) => {
                    this.ended = true;
                    this.stored(result)
                }
            };
        }
        match this.inner.as_mut().poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok(ChatEvent::Delta(text)))) => {
                this.buffer.push_str(&text);
                Self::delta(&text)
            }
            Poll::Ready(Some(Ok(ChatEvent::Done(outcome)))) => {
                let now = this.clock.now();
                let store = Arc::clone(&this.store);
                let student_id = this.student_id;
                let conversation_id = this.conversation_id;
                let content = std::mem::take(&mut this.buffer);
                this.storing = Some(Box::pin(async move {
                    // Stored only now, when the upstream finished: the content is exactly the
                    // concatenated deltas.
                    let message = NewMessage {
                        conversation_id,
                        role: MessageRole::Assistant,
                        content: content.as_str(),
                        sources: outcome.sources.as_slice(),
                        is_grounded: Some(outcome.grounded),
                        // Only a conversation's first user message names it.
                        title_if_first: None,
                    };
                    store.add_message(student_id, message, now).await
                }));
                // The store call runs on the next poll of this same stream.
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Some(Err(err))) => {
                // Logs once, where the failure becomes the response.
                let api_error = upstream_errors::api_error(&err, &this.request_id);
                this.ended = true;
                this.error(&api_error)
            }
            // The upstream's contract is a Done or an error first; anything else is its bug.
            Poll::Ready(None) => {
                let api_error = upstream_errors::api_error(
                    &belajar_ports::upstream::UpstreamError::Protocol(
                        "the chat stream ended without a final event".to_owned(),
                    ),
                    &this.request_id,
                );
                this.ended = true;
                this.error(&api_error)
            }
        }
    }
}

// ===== handlers ====================================================================================================

/// Asks a question. The answer streams in as `text/event-stream`.
#[utoipa::path(
    post,
    path = "/v1/conversations/{id}/messages",
    request_body = SendMessageRequest,
    params(("id" = Uuid, Path, description = "The conversation")),
    responses(
        (status = 200, description = "SSE events: `message` (the stored question), `delta` per answer piece, then `done` (the stored answer) — or one last `error` with the standard error body", content_type = "text/event-stream"),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
        (status = 502, body = ErrorBody),
        (status = 503, body = ErrorBody),
        (status = 504, body = ErrorBody),
    )
)]
pub(crate) async fn post_message(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path(raw): Path<String>,
    ApiJson(req): ApiJson<SendMessageRequest>,
) -> Response {
    // Validation runs before anything is stored or any upstream call is made.
    let content = req.content.trim();
    let mut errors = Vec::new();
    bad_length(&mut errors, "/content", content.chars().count(), 1, 2000);
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    let Some(conversation_id) = conversation_id(&raw) else {
        return not_found(&id);
    };
    let conversation = match state
        .conversations
        .get_conversation(student.student_id, conversation_id)
        .await
    {
        Ok(conversation) => conversation,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let profile = match state.accounts.get_student(student.student_id).await {
        Ok(profile) => profile,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let history = match state
        .conversations
        .last_messages(student.student_id, conversation_id, MEMORY_WINDOW)
        .await
    {
        Ok(history) => history,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let title = title_if_first(&conversation, &history, content);
    let now = state.clock.now();
    let user_message = match state
        .conversations
        .add_message(
            student.student_id,
            NewMessage {
                conversation_id,
                role: MessageRole::User,
                content,
                sources: &[],
                is_grounded: None,
                title_if_first: title.as_deref(),
            },
            now,
        )
        .await
    {
        Ok(message) => message,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let request = ChatRequest {
        grade: profile.grade,
        assistant: assistant_for(conversation.mode),
        messages: chat_messages(&profile, &conversation, &history, content),
        mapel: mapel_of(&conversation),
    };
    let inner = match state.upstream.chat(request).await {
        Ok(inner) => inner,
        // The question is stored; the answer could not even start.
        Err(err) => return upstream_errors::into_response(&err, &id.0),
    };
    let Some(first) = json_event("message", &MessageDto::from(user_message)) else {
        tracing::error!(
            request_id = %id.0,
            code = "internal",
            "serializing the stored question failed"
        );
        return ApiError::new(ErrorCode::Internal).into_response(&id.0);
    };
    let answer = AnswerStream {
        first: Some(first),
        inner,
        storing: None,
        store: Arc::clone(&state.conversations),
        clock: Arc::clone(&state.clock),
        student_id: student.student_id,
        conversation_id,
        buffer: String::new(),
        request_id: id.0.clone(),
        ended: false,
    };
    Sse::new(answer)
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// The conversation's stored messages, oldest first.
#[utoipa::path(
    get,
    path = "/v1/conversations/{id}/messages",
    params(
        ("id" = Uuid, Path, description = "The conversation"),
        ("limit" = Option<u32>, Query, description = "Default 50, at most 100"),
        ("cursor" = Option<String>, Query, description = "The nextCursor of a previous page"),
    ),
    responses(
        (status = 200, body = MessageListDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
    )
)]
pub(crate) async fn list_messages(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path(raw): Path<String>,
    ApiQuery(query): ApiQuery<MessagesQuery>,
) -> Response {
    let mut errors = Vec::new();
    let limit = query_limit(query.limit.as_deref(), 50, MESSAGE_PAGE_MAX, &mut errors);
    let after = query_cursor(query.cursor.as_deref(), &mut errors);
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    let Some(conversation_id) = conversation_id(&raw) else {
        return not_found(&id);
    };
    match state
        .conversations
        .list_messages(student.student_id, conversation_id, after, limit)
        .await
    {
        Ok(page) => (
            StatusCode::OK,
            Json(MessageListDto {
                data: page.data.into_iter().map(MessageDto::from).collect(),
                next_cursor: page.next_cursor.map(cursor::encode),
            }),
        )
            .into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// The first question of an ask conversation names it; a renamed one already has messages.
/// A learn conversation keeps its chapter title: the lesson's `chapterTitle` is read from it.
fn title_if_first(
    conversation: &Conversation,
    history: &[Message],
    content: &str,
) -> Option<String> {
    (conversation.mode == ConversationMode::Ask && history.is_empty())
        .then(|| title_from_question(content))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use belajar_core::grade::Grade;
    use chrono::{TimeZone, Utc};

    use super::*;

    fn profile(grade: u8) -> StudentAccount {
        StudentAccount {
            id: Uuid::now_v7(),
            username: "budi".into(),
            nickname: "budi".into(),
            grade: Grade::new(grade).unwrap(),
            theme: belajar_core::theme::ThemePreference::System,
            created_at: Utc.with_ymd_and_hms(2026, 11, 19, 8, 30, 0).unwrap(),
            has_password: true,
            is_google_linked: false,
        }
    }

    fn ask_conversation() -> Conversation {
        Conversation {
            id: Uuid::now_v7(),
            student_id: Uuid::now_v7(),
            mode: ConversationMode::Ask,
            subject_id: Some("ipa".into()),
            chapter_id: None,
            title: "Percakapan baru".into(),
            created_at: Utc.with_ymd_and_hms(2026, 11, 19, 8, 30, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 11, 19, 8, 30, 0).unwrap(),
        }
    }

    #[test]
    fn the_system_message_profiles_the_student_and_the_subject() {
        let text = profile_system_message(&profile(7), &ask_conversation());
        assert!(text.contains("budi"));
        assert!(text.contains("7"));
        assert!(text.contains("IPA"));
        assert!(text.contains("buku pelajaran"));

        let mut all = ask_conversation();
        all.subject_id = None;
        assert!(profile_system_message(&profile(7), &all).contains("semua mapel"));

        let mut learn = ask_conversation();
        learn.mode = ConversationMode::Learn;
        learn.title = "Bab 5: Suhu, Kalor, dan Pemuaian".into();
        assert!(profile_system_message(&profile(7), &learn).contains("Bab: Bab 5"));
    }

    #[test]
    fn the_window_is_the_profile_the_history_then_the_question() {
        let history: Vec<Message> = (0..12)
            .map(|i| Message {
                id: Uuid::now_v7(),
                conversation_id: Uuid::now_v7(),
                student_id: Uuid::now_v7(),
                role: if i % 2 == 0 {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                },
                content: format!("pesan {i}"),
                sources: Vec::new(),
                is_grounded: None,
                created_at: Utc
                    .with_ymd_and_hms(2026, 11, 19, 8, i / 60, i % 60)
                    .unwrap(),
            })
            .collect();
        let messages = chat_messages(&profile(7), &ask_conversation(), &history, "baru?");
        assert_eq!(messages.len(), history.len() + 2);
        assert_eq!(messages[0].role, ChatRole::System);
        assert_eq!(messages[1].content, "pesan 0");
        assert_eq!(
            messages.last().unwrap().role,
            ChatRole::User,
            "the question is the final message"
        );
    }

    #[test]
    fn a_title_is_trimmed_and_never_longer_than_sixty_characters() {
        assert_eq!(
            title_from_question("  Apa itu pemuaian?  "),
            "Apa itu pemuaian?"
        );
        let long = "x".repeat(64);
        assert_eq!(title_from_question(&long).chars().count(), 60);
        // The cut lands on a character boundary, never inside one.
        let emoji = "é".repeat(61);
        assert_eq!(title_from_question(&emoji).chars().count(), 60);
    }

    #[test]
    fn only_the_first_question_of_an_ask_conversation_names_it() {
        let question = "Kenapa rel kereta api diberi celah?";
        assert_eq!(
            title_if_first(&ask_conversation(), &[], question).as_deref(),
            Some(question)
        );
        let mut learn = ask_conversation();
        learn.mode = ConversationMode::Learn;
        assert_eq!(
            title_if_first(&learn, &[], question),
            None,
            "a lesson keeps its chapter title"
        );
    }

    #[test]
    fn mapel_is_the_upstreams_subject_name_or_none() {
        assert_eq!(mapel_of(&ask_conversation()).as_deref(), Some("IPA"));
        let mut mtk = ask_conversation();
        mtk.subject_id = Some("mtk".into());
        assert_eq!(mapel_of(&mtk).as_deref(), Some("Matematika"));
        let mut all = ask_conversation();
        all.subject_id = None;
        assert_eq!(mapel_of(&all), None);
    }

    #[test]
    fn content_follows_the_question_length_rules() {
        let mut errors = Vec::new();
        bad_length(
            &mut errors,
            "/content",
            "   ".trim().chars().count(),
            1,
            2000,
        );
        assert_eq!(errors[0].code, "too_short");
        let mut errors = Vec::new();
        bad_length(
            &mut errors,
            "/content",
            "x".repeat(2001).chars().count(),
            1,
            2000,
        );
        assert_eq!(errors[0].code, "too_long");
        let mut errors = Vec::new();
        bad_length(
            &mut errors,
            "/content",
            "x".repeat(2000).chars().count(),
            1,
            2000,
        );
        assert!(errors.is_empty(), "2000 is the largest answer");
    }
}
