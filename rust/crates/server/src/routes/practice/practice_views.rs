//! Reading Latihan: one set (answers hidden until submit), the newest-first list with its opaque
//! cursor, the book page behind a submitted answer, and topic mastery.

use std::collections::HashMap;

use axum::Extension;
use axum::Json;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use belajar_core::subject;
use belajar_ports::practice::{PracticeSetStatus, StoredPracticeSet};

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody, FieldError};
use crate::current_student::CurrentStudent;
use crate::error_codes::ErrorCode;
use crate::routes::practice::practice_dto::{
    GradedQuestion, PracticeResultDto, PracticeSetDto, PracticeSetListDto, TopicMasteryDto,
    TopicMasteryListDto, result_dto, set_dto, summary_dto,
};
use crate::state::AppState;
use crate::store_errors;
use crate::upstream_errors;
use crate::wire::fields;

const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 50;

/// One set, exactly as `POST /v1/practice-sets` delivered it in the `ready` event — the answer
/// key appears only in `result`, and only after the set is submitted.
#[utoipa::path(
    get,
    path = "/v1/practice-sets/{id}",
    params(("id" = Uuid, Path, description = "The practice set")),
    responses(
        (status = 200, body = PracticeSetDto),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
    )
)]
pub(crate) async fn get_practice_set(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path(set_id): Path<Uuid>,
) -> Response {
    let row = match state.practice.get_set(student.student_id, set_id).await {
        Ok(Some(row)) => row,
        Ok(None) => return ApiError::new(ErrorCode::NotFound).into_response(&id.0),
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let result = match review(&state, student.student_id, &row).await {
        Ok(result) => result,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    (StatusCode::OK, Json(set_dto(&row, result))).into_response()
}

/// The stored review of a submitted set, rebuilt from its answer rows; `None` while it is ready.
async fn review(
    state: &AppState,
    student_id: Uuid,
    row: &StoredPracticeSet,
) -> Result<Option<PracticeResultDto>, belajar_ports::StoreError> {
    if row.status != PracticeSetStatus::Submitted {
        return Ok(None);
    }
    let stored = state.practice.answers(student_id, row.id).await?;
    let by_id: HashMap<&str, &belajar_ports::practice::StoredPracticeAnswer> = stored
        .iter()
        .map(|answer| (answer.question_id.as_str(), answer))
        .collect();
    let mut graded: Vec<GradedQuestion<'_>> = Vec::with_capacity(row.practice.questions.len());
    let mut correct_count = 0u32;
    for question in &row.practice.questions {
        // The submission wrote one row per question, so every question has its answer.
        let Some(answer) = by_id.get(question.id.as_str()) else {
            return Err(belajar_ports::StoreError::Internal(
                "a stored answer went missing".into(),
            ));
        };
        if answer.is_correct {
            correct_count += 1;
        }
        graded.push(GradedQuestion {
            question,
            selected: &answer.selected,
            is_correct: answer.is_correct,
        });
    }
    Ok(Some(result_dto(
        row.id,
        &graded,
        correct_count,
        5 * correct_count,
    )))
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListParams {
    limit: Option<String>,
    cursor: Option<String>,
}

/// This student's sets, newest first, paginated.
#[utoipa::path(
    get,
    path = "/v1/practice-sets",
    params(
        ("limit" = Option<u32>, Query, description = "Page size; default 20, at most 50, larger is lowered, 0 or less is invalid"),
        ("cursor" = Option<String>, Query, description = "The nextCursor of the previous page"),
    ),
    responses(
        (status = 200, body = PracticeSetListDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn list_practice_sets(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    query: Result<Query<ListParams>, QueryRejection>,
) -> Response {
    // A rejected query string (bad percent-encoding) has no field to name; the code says it.
    let Ok(Query(params)) = query else {
        return ApiError::new(ErrorCode::InvalidRequest).into_response(&id.0);
    };
    let limit = match parse_limit(params.limit.as_deref()) {
        Ok(limit) => limit,
        Err(error) => return ApiError::invalid(vec![error]).into_response(&id.0),
    };
    let after = match params.cursor.as_deref().map(decode_cursor) {
        None => None,
        Some(Some(after)) => Some(after),
        Some(None) => {
            let error = fields::invalid_format("cursor", "a cursor this API returned");
            return ApiError::invalid(vec![error]).into_response(&id.0);
        }
    };
    // One extra row tells the client whether another page follows.
    let rows = match state
        .practice
        .list_sets(student.student_id, limit + 1, after)
        .await
    {
        Ok(rows) => rows,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let has_more = rows.len() > limit as usize;
    let data: Vec<_> = rows
        .into_iter()
        .take(limit as usize)
        .map(|row| summary_dto(&row))
        .collect();
    let next_cursor = if has_more && let Some(last) = data.last() {
        Some(encode_cursor(last.created_at, last.id))
    } else {
        None
    };
    (
        StatusCode::OK,
        Json(PracticeSetListDto { data, next_cursor }),
    )
        .into_response()
}

fn parse_limit(raw: Option<&str>) -> Result<u32, FieldError> {
    let Some(raw) = raw else {
        return Ok(DEFAULT_LIMIT);
    };
    let value: i64 = raw
        .trim()
        .parse()
        .map_err(|_| fields::invalid_format("limit", "a whole number"))?;
    if value < 1 {
        return Err(fields::out_of_count("limit", 1, None));
    }
    Ok(u32::try_from(value.min(i64::from(MAX_LIMIT))).unwrap_or(MAX_LIMIT))
}

/// The cursor is opaque: base64url of `<unix micros>|<id>`, the list's own sort key. Micros is
/// exact, because the column's precision is microseconds — truncating to the wire format's
/// milliseconds could skip rows created within the same one.
fn encode_cursor(created_at: DateTime<Utc>, id: Uuid) -> String {
    URL_SAFE_NO_PAD.encode(format!("{}|{}", created_at.timestamp_micros(), id))
}

fn decode_cursor(cursor: &str) -> Option<(DateTime<Utc>, Uuid)> {
    let decoded = URL_SAFE_NO_PAD.decode(cursor.trim()).ok()?;
    let text = String::from_utf8(decoded).ok()?;
    let (micros, id) = text.split_once('|')?;
    let micros: i64 = micros.parse().ok()?;
    let created_at = DateTime::from_timestamp_micros(micros)?;
    let id = Uuid::parse_str(id).ok()?;
    Some((created_at, id))
}

/// The book page a cited answer used, only for the owner and only after submit.
#[utoipa::path(
    get,
    path = "/v1/practice-sets/{id}/questions/{questionId}/page",
    params(
        ("id" = Uuid, Path, description = "The practice set"),
        ("questionId" = String, Path, description = "The question whose cited source has a page"),
    ),
    responses(
        (status = 200, description = "The book page image, fetched server-side", content_type = "image/svg+xml"),
        (status = 401, body = ErrorBody),
        (status = 404, description = "No such set or question, not submitted, or no page for it", body = ErrorBody),
        (status = 502, body = ErrorBody),
        (status = 503, body = ErrorBody),
        (status = 504, body = ErrorBody),
    )
)]
pub(crate) async fn question_page(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path((set_id, question_id)): Path<(Uuid, String)>,
) -> Response {
    let row = match state.practice.get_set(student.student_id, set_id).await {
        Ok(Some(row)) => row,
        Ok(None) => return ApiError::new(ErrorCode::NotFound).into_response(&id.0),
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    // The page is part of the review, so it appears only after the set is submitted.
    if row.status != PracticeSetStatus::Submitted {
        return ApiError::new(ErrorCode::NotFound).into_response(&id.0);
    }
    let Some(question) = row
        .practice
        .questions
        .iter()
        .find(|question| question.id == question_id)
    else {
        return ApiError::new(ErrorCode::NotFound).into_response(&id.0);
    };
    let Some(page_url) = question
        .source
        .as_ref()
        .and_then(|source| source.page_url.as_deref())
    else {
        return ApiError::new(ErrorCode::NotFound).into_response(&id.0);
    };
    match state.upstream.page(page_url).await {
        Ok(image) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, image.content_type)],
            image.bytes,
        )
            .into_response(),
        Err(err) => upstream_errors::into_response(&err, &id.0),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MasteryParams {
    subject_id: Option<String>,
}

/// The student's mastery of every topic they have practised.
#[utoipa::path(
    get,
    path = "/v1/topic-mastery",
    params(("subjectId" = Option<String>, Query, description = "One of our subject ids; all of them when omitted")),
    responses(
        (status = 200, body = TopicMasteryListDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn topic_mastery(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    query: Result<Query<MasteryParams>, QueryRejection>,
) -> Response {
    let Ok(Query(params)) = query else {
        return ApiError::new(ErrorCode::InvalidRequest).into_response(&id.0);
    };
    if let Some(subject_id) = params.subject_id.as_deref()
        && subject::by_id(subject_id).is_none()
    {
        let error = fields::invalid_format("subjectId", "one of our subject ids");
        return ApiError::invalid(vec![error]).into_response(&id.0);
    }
    let entries = match state
        .practice
        .list_topic_mastery(student.student_id, params.subject_id.as_deref())
        .await
    {
        Ok(entries) => entries,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let data: Vec<TopicMasteryDto> = entries
        .into_iter()
        .map(|entry| TopicMasteryDto {
            subject_id: entry.subject_id,
            topic: entry.topic,
            attempts: entry.attempts,
            correct: entry.correct,
            mastery_percent: mastery_percent(entry.correct, entry.attempts),
        })
        .collect();
    (
        StatusCode::OK,
        Json(TopicMasteryListDto {
            data,
            next_cursor: None,
        }),
    )
        .into_response()
}

/// Round(100 × correct / attempts). A row only exists after a submission, so attempts is never
/// zero; the guard keeps the arithmetic honest anyway.
fn mastery_percent(correct: u32, attempts: u32) -> u32 {
    if attempts == 0 {
        return 0;
    }
    ((100.0 * correct as f64) / attempts as f64).round() as u32
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use super::{MAX_LIMIT, decode_cursor, encode_cursor, mastery_percent, parse_limit};

    #[test]
    fn a_cursor_round_trips_and_garbage_is_none() {
        let created_at = chrono::Utc::now();
        let id = uuid::Uuid::now_v7();
        let encoded = encode_cursor(created_at, id);
        assert!(!encoded.contains('|'), "the cursor is opaque: {encoded}");
        let (back_at, back_id) = decode_cursor(&encoded).unwrap();
        // Micros is the precision the column has; the cursor keeps all of it.
        assert_eq!(back_at.timestamp_micros(), created_at.timestamp_micros());
        assert_eq!(back_id, id);
        for junk in ["", "not base64!", "only-one-part", "a|b", "2026-13-99|x"] {
            assert!(decode_cursor(junk).is_none(), "{junk}");
        }
    }

    #[test]
    fn the_limit_has_a_default_a_maximum_and_a_floor() {
        assert_eq!(parse_limit(None).unwrap(), 20);
        assert_eq!(parse_limit(Some("8")).unwrap(), 8);
        assert_eq!(parse_limit(Some("5000")).unwrap(), MAX_LIMIT);
        assert_eq!(parse_limit(Some(" 10 ")).unwrap(), 10);
        for bad in ["0", "-3"] {
            let error = parse_limit(Some(bad)).unwrap_err();
            assert_eq!(error.field, "limit");
            assert_eq!(error.code, "out_of_range");
        }
        let error = parse_limit(Some("many")).unwrap_err();
        assert_eq!(error.field, "limit");
        assert_eq!(error.code, "invalid_format");
    }

    #[test]
    fn mastery_percent_rounds_and_survives_zero_attempts() {
        assert_eq!(mastery_percent(4, 5), 80);
        assert_eq!(mastery_percent(1, 3), 33);
        assert_eq!(mastery_percent(2, 3), 67);
        assert_eq!(mastery_percent(5, 5), 100);
        assert_eq!(mastery_percent(0, 7), 0);
        assert_eq!(mastery_percent(0, 0), 0);
    }
}
