//! `POST /v1/practice-sets`: validate every field, ask the upstream to generate a set, then
//! stream the progress live as SSE and store the set only once it has arrived complete. The
//! answer key reaches the browser only inside the `ready` event's questions — which never
//! include it.

use std::convert::Infallible;
use std::time::Duration;

use axum::Extension;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, KeepAliveStream, Sse};
use axum::response::{IntoResponse, Response};
use belajar_core::subject;
use belajar_ports::practice::NewPracticeSet;
use belajar_ports::upstream::{
    PracticeEvent, PracticeRequest, PracticeStage, PracticeStream, PracticeType,
};
use futures::SinkExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody, FieldError};
use crate::current_student::CurrentStudent;
use crate::routes::practice::practice_dto::{PracticeSetDto, set_dto};
use crate::routes::practice::{internal, practice_dto};
use crate::state::AppState;
use crate::store_errors;
use crate::upstream_errors;
use crate::wire::extract::ApiJson;
use crate::wire::fields;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratePracticeRequest {
    /// One of our eight subject ids, e.g. `ipa`.
    pub subject_id: String,
    pub topic: String,
    /// One of `mcq_normal`, `mcq_multiple_correct`, `mcq_story`, `mcq_mixed`.
    pub practice_type: String,
    /// 5, 10 or 15.
    pub total_questions: u64,
}

/// One `progress` event of the generation stream.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEventDto {
    pub stage: String,
    pub delivered: u32,
    pub total: u32,
    pub elapsed_ms: u64,
}

/// The last event of a successful generation.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadyEventDto {
    pub practice_set: PracticeSetDto,
}

/// The one line of a streamed event's data; never a bare error body without its envelope.
type EventSender = futures::channel::mpsc::Sender<Result<Event, Infallible>>;

/// The SSE body: the channel's events, with a keep-alive comment while the caller waits.
type SseBody = Sse<KeepAliveStream<futures::channel::mpsc::Receiver<Result<Event, Infallible>>>>;

/// One `POST /v1/practice-sets` worth of validated fields, on their way into the stream task.
struct Generation {
    subject_id: String,
    topic: String,
    practice_type: PracticeType,
    total_questions: u16,
}

/// Generates a practice set, streaming the upstream's progress to the caller.
#[utoipa::path(
    post,
    path = "/v1/practice-sets",
    request_body = GeneratePracticeRequest,
    responses(
        (status = 200, description = "SSE stream: `progress` events, then one `ready` event with the set — or one `error` event with the standard error body", content_type = "text/event-stream"),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
        (status = 502, body = ErrorBody),
        (status = 503, body = ErrorBody),
        (status = 504, body = ErrorBody),
    )
)]
pub(crate) async fn create_practice_set(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    ApiJson(req): ApiJson<GeneratePracticeRequest>,
) -> Response {
    let errors = validate_generate(&req);
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }
    // Validation has already bounded these; each arm only keeps the type honest.
    let Some(subject) = subject::by_id(&req.subject_id) else {
        return internal(&id, "subject id out of range after validation");
    };
    let Some(practice_type) = practice_dto::practice_type_from_name(&req.practice_type) else {
        return internal(&id, "practice type out of range after validation");
    };
    let Some(total) = u8::try_from(req.total_questions).ok() else {
        return internal(&id, "total questions out of range after validation");
    };
    let account = match state.accounts.get_student(student.student_id).await {
        Ok(account) => account,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let upstream_request = PracticeRequest {
        grade: account.grade,
        topics: vec![req.topic.trim().to_owned()],
        mapel: Some(subject.mapel.to_owned()),
        practice_type,
        total_questions: total,
    };
    // A failure to start (no key, upstream down) is a normal JSON error: nothing was streamed.
    let stream = match state.upstream.generate_practice(upstream_request).await {
        Ok(stream) => stream,
        Err(err) => return upstream_errors::into_response(&err, &id.0),
    };
    let generation = Generation {
        subject_id: subject.id.to_owned(),
        topic: req.topic.trim().to_owned(),
        practice_type,
        total_questions: u16::from(total),
    };
    sse_response(&state, student.student_id, generation, stream, id.0.clone()).into_response()
}

/// The stream itself: one task that reads the upstream and fills one channel, one SSE response
/// that drains it, and a keep-alive comment at least every 15 s while the caller waits.
fn sse_response(
    state: &AppState,
    student_id: Uuid,
    generation: Generation,
    stream: PracticeStream,
    request_id: String,
) -> SseBody {
    let (mut tx, rx) = futures::channel::mpsc::channel(64);
    let store = state.practice.clone();
    let clock = state.clock.clone();
    tokio::spawn(async move {
        let mut stream = stream;
        while let Some(event) = stream.next().await {
            match event {
                Ok(PracticeEvent::Progress(progress)) => {
                    let payload = ProgressEventDto {
                        stage: stage_name(progress.stage).to_owned(),
                        delivered: progress.delivered,
                        total: progress.total,
                        elapsed_ms: progress.elapsed_ms,
                    };
                    if send_json(&mut tx, "progress", &payload).await.is_err() {
                        return; // the caller hung up
                    }
                }
                Ok(PracticeEvent::Result(set)) => {
                    let now = clock.now();
                    let new_set = NewPracticeSet {
                        subject_id: &generation.subject_id,
                        topic: &generation.topic,
                        practice_type: generation.practice_type,
                        // The upstream may deliver fewer than asked for (its checks drop
                        // questions); the set counts what it holds.
                        total_questions: u16::try_from(set.questions.len())
                            .unwrap_or(generation.total_questions),
                        practice: &set,
                        created_at: now,
                    };
                    match store.create_set(student_id, &new_set).await {
                        Ok(row) => {
                            let payload = ReadyEventDto {
                                practice_set: set_dto(&row, None),
                            };
                            let _ = send_json(&mut tx, "ready", &payload).await;
                        }
                        Err(err) => send_store_error(&mut tx, err, &request_id).await,
                    }
                    return;
                }
                Err(err) => {
                    // The stream has started, so the failure is one last event with the same
                    // error body a JSON response would carry.
                    let body = upstream_errors::api_error(&err, &request_id).body(&request_id);
                    let _ = send_body(&mut tx, body).await;
                    return;
                }
            }
        }
        // The upstream client ends its streams after the result or the first error; an end
        // without either is its protocol error, already reported as one.
    });
    Sse::new(rx).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/// One event's data is one JSON value, as the plan fixes the framing.
async fn send_json<T: Serialize>(tx: &mut EventSender, name: &str, payload: &T) -> Result<(), ()> {
    let event = Event::default()
        .event(name)
        .json_data(payload)
        .map_err(|_| ())?;
    send(tx, event).await
}

async fn send_body(tx: &mut EventSender, body: ErrorBody) -> Result<(), ()> {
    let event = Event::default()
        .event("error")
        .json_data(body)
        .map_err(|_| ())?;
    send(tx, event).await
}

async fn send(tx: &mut EventSender, event: Event) -> Result<(), ()> {
    tx.send(Ok(event)).await.map_err(|_| ())
}

/// Storing a complete set cannot fail for a reason the caller caused; say `internal`, log the rest.
async fn send_store_error(tx: &mut EventSender, err: belajar_ports::StoreError, request_id: &str) {
    let code = match &err {
        belajar_ports::StoreError::NotFound => crate::error_codes::ErrorCode::NotFound,
        belajar_ports::StoreError::Conflict => crate::error_codes::ErrorCode::Conflict,
        belajar_ports::StoreError::Internal(cause) => {
            tracing::error!(
                request_id,
                code = "internal",
                error = %cause,
                "storing the practice set failed"
            );
            crate::error_codes::ErrorCode::Internal
        }
    };
    let _ = send_body(tx, ApiError::new(code).body(request_id)).await;
}

fn stage_name(stage: PracticeStage) -> &'static str {
    match stage {
        PracticeStage::Retrieval => "retrieval",
        PracticeStage::Generating => "generating",
        PracticeStage::Checking => "checking",
        PracticeStage::Labelling => "labelling",
    }
}

fn validate_generate(req: &GeneratePracticeRequest) -> Vec<FieldError> {
    let mut errors = Vec::new();
    if subject::by_id(&req.subject_id).is_none() {
        errors.push(fields::invalid_format(
            "/subjectId",
            "one of our subject ids",
        ));
    }
    let topic_len = req.topic.trim().chars().count();
    fields::check_length(&mut errors, "/topic", topic_len, 1, 100);
    if practice_dto::practice_type_from_name(&req.practice_type).is_none() {
        errors.push(fields::invalid_format(
            "/practiceType",
            "mcq_normal, mcq_multiple_correct, mcq_story or mcq_mixed",
        ));
    }
    if !matches!(req.total_questions, 5 | 10 | 15) {
        errors.push(fields::out_of_range("/totalQuestions", 5, 15));
    }
    errors
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::expect_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use serde_json::json;

    use super::GeneratePracticeRequest;

    #[test]
    fn the_request_shape_is_camel_case_and_rejects_unknown_fields() {
        let parsed: GeneratePracticeRequest = serde_json::from_value(json!({
            "subjectId": "ipa", "topic": "Pemuaian",
            "practiceType": "mcq_normal", "totalQuestions": 5,
        }))
        .expect("the four fields are exactly the request");
        assert_eq!(parsed.total_questions, 5);

        assert!(
            serde_json::from_value::<GeneratePracticeRequest>(
                json!({ "subjectId": "ipa", "topic": "x", "practiceType": "mcq_normal", "totalQuestions": 5, "extra": 1 })
            )
            .is_err(),
            "an unknown field is rejected"
        );
    }

    #[test]
    fn generation_collects_every_bad_field() {
        let req: GeneratePracticeRequest = serde_json::from_value(json!({
            "subjectId": "kimia", "topic": "", "practiceType": "essay", "totalQuestions": 7,
        }))
        .expect("every field decodes, even the bad ones");
        let errors = super::validate_generate(&req);
        let mut fields: Vec<&str> = errors.iter().map(|error| error.field.as_str()).collect();
        fields.sort_unstable();
        assert_eq!(
            fields,
            vec!["/practiceType", "/subjectId", "/topic", "/totalQuestions"]
        );
    }

    #[test]
    fn a_valid_request_has_no_errors() {
        let req: GeneratePracticeRequest = serde_json::from_value(json!({
            "subjectId": "ipa", "topic": " Pemuaian ", "practiceType": "mcq_mixed", "totalQuestions": 15,
        }))
        .expect("a valid body decodes");
        assert!(super::validate_generate(&req).is_empty());
    }
}
