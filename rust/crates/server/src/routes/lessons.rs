//! The lesson routes: start or resume one, read it, move its step, complete it with a quiz
//! result. Handlers only decode, validate, call one store method and map the error; the star and
//! completion rules live in `belajar_core` and `belajar-store-pg`.

use std::collections::HashMap;

use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use belajar_core::stars::quiz_stars;
use belajar_core::subject;
use belajar_ports::lessons::Lesson;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody, FieldError};
use crate::current_student::CurrentStudent;
use crate::error_codes::ErrorCode;
use crate::state::AppState;
use crate::store_errors;
use crate::upstream_errors;
use crate::wire::extract::ApiJson;
use crate::wire::instant;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateLessonRequest {
    pub subject_id: String,
    pub chapter_id: String,
}

/// The body of `PATCH /v1/lessons/{id}`. `step` is a string so a bad value is a field error, not
/// a body parse error.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatchLessonRequest {
    pub step: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteLessonRequest {
    pub quiz_correct: u64,
    pub quiz_total: u64,
}

/// One guided lesson, as every lesson response shows it.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LessonDto {
    pub id: Uuid,
    pub subject_id: String,
    pub chapter_id: String,
    pub chapter_title: String,
    pub conversation_id: Uuid,
    /// `explain`, `check`, `quiz` or `done`.
    pub step: String,
    pub stars: u8,
    #[serde(with = "instant::option")]
    #[schema(format = DateTime, required)]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl From<Lesson> for LessonDto {
    fn from(lesson: Lesson) -> Self {
        Self {
            id: lesson.id,
            subject_id: lesson.subject_id,
            chapter_id: lesson.chapter_id,
            chapter_title: lesson.chapter_title,
            conversation_id: lesson.conversation_id,
            step: lesson.step.as_str().to_owned(),
            stars: lesson.stars,
            completed_at: lesson.completed_at,
            updated_at: lesson.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CompleteLessonResponse {
    pub lesson: LessonDto,
    pub stars_earned: u32,
}

fn invalid(field: &str, what: &str) -> FieldError {
    FieldError {
        field: field.into(),
        code: "invalid_format",
        message: format!("Must be {what}."),
    }
}

/// A path segment that should be a lesson id. A malformed one can never exist, so the caller
/// answers 404 with the standard body, not axum's plain-text extractor rejection.
fn lesson_id(path: &str) -> Option<Uuid> {
    Uuid::parse_str(path).ok()
}

/// `POST /v1/lessons`: starts the chapter's lesson, or resumes the existing one unchanged.
#[utoipa::path(
    post,
    path = "/v1/lessons",
    request_body = CreateLessonRequest,
    responses(
        (status = 201, description = "The lesson was started", body = LessonDto),
        (status = 200, description = "The lesson already existed and is returned unchanged",
         body = LessonDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn create_lesson(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    ApiJson(req): ApiJson<CreateLessonRequest>,
) -> Response {
    // A subject we do not know can never be validated further, so this is the whole validation.
    let Some(subject) = subject::by_id(&req.subject_id) else {
        return ApiError::invalid(vec![invalid("/subjectId", "one of our subject ids")])
            .into_response(&id.0);
    };

    let account = match state.accounts.get_student(student.student_id).await {
        Ok(account) => account,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let chapters = match state.upstream.chapters(account.grade, subject.mapel).await {
        Ok(chapters) => chapters,
        Err(err) => return upstream_errors::into_response(&err, &id.0),
    };
    let Some(chapter) = chapters.iter().find(|chapter| chapter.id == req.chapter_id) else {
        return ApiError::invalid(vec![invalid(
            "/chapterId",
            "one of this subject's chapter ids",
        )])
        .into_response(&id.0);
    };

    let now = state.clock.now();
    match state
        .lessons
        .start(
            student.student_id,
            subject.id,
            &req.chapter_id,
            &chapter.title,
            now,
        )
        .await
    {
        Ok((lesson, true)) => (StatusCode::CREATED, Json(LessonDto::from(lesson))).into_response(),
        Ok((lesson, false)) => (StatusCode::OK, Json(LessonDto::from(lesson))).into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// `GET /v1/lessons/{id}`.
#[utoipa::path(
    get,
    path = "/v1/lessons/{id}",
    params(("id" = Uuid, description = "The lesson's id")),
    responses(
        (status = 200, body = LessonDto),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
    )
)]
pub(crate) async fn get_lesson(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path(path): Path<String>,
) -> Response {
    let Some(lesson_id) = lesson_id(&path) else {
        return ApiError::new(ErrorCode::NotFound).into_response(&id.0);
    };
    match state.lessons.get(student.student_id, lesson_id).await {
        Ok(lesson) => (StatusCode::OK, Json(LessonDto::from(lesson))).into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// `PATCH /v1/lessons/{id}`: moves the lesson's step. `done` is refused: only completing a quiz
/// finishes a lesson.
#[utoipa::path(
    patch,
    path = "/v1/lessons/{id}",
    params(("id" = Uuid, description = "The lesson's id")),
    request_body = PatchLessonRequest,
    responses(
        (status = 200, body = LessonDto),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
    )
)]
pub(crate) async fn patch_lesson(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path(path): Path<String>,
    ApiJson(req): ApiJson<PatchLessonRequest>,
) -> Response {
    // `done` is not patchable: only completing a quiz finishes a lesson.
    let step = match req.step.as_str() {
        "explain" => Some(belajar_ports::lessons::LessonStep::Explain),
        "check" => Some(belajar_ports::lessons::LessonStep::Check),
        "quiz" => Some(belajar_ports::lessons::LessonStep::Quiz),
        _ => None,
    };
    let Some(step) = step else {
        return ApiError::invalid(vec![invalid("/step", "one of: explain, check, quiz")])
            .into_response(&id.0);
    };
    let Some(lesson_id) = lesson_id(&path) else {
        return ApiError::new(ErrorCode::NotFound).into_response(&id.0);
    };
    let now = state.clock.now();
    match state
        .lessons
        .set_step(student.student_id, lesson_id, step, now)
        .await
    {
        Ok(lesson) => (StatusCode::OK, Json(LessonDto::from(lesson))).into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// `POST /v1/lessons/{id}/complete`: applies the guided session's quiz result, keeps the best
/// stars and awards the star ledger rows for newly reached levels.
#[utoipa::path(
    post,
    path = "/v1/lessons/{id}/complete",
    params(("id" = Uuid, description = "The lesson's id")),
    request_body = CompleteLessonRequest,
    responses(
        (status = 200, body = CompleteLessonResponse),
        (status = 400, body = ErrorBody),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
    )
)]
pub(crate) async fn complete_lesson(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path(path): Path<String>,
    ApiJson(req): ApiJson<CompleteLessonRequest>,
) -> Response {
    // Every bad field is collected: a range can break both fields at once.
    let mut errors = Vec::new();
    if !(1..=20).contains(&req.quiz_total) {
        errors.push(FieldError {
            field: "/quizTotal".into(),
            code: "out_of_range",
            message: "Must be between 1 and 20.".into(),
        });
    }
    if req.quiz_correct > req.quiz_total {
        errors.push(FieldError {
            field: "/quizCorrect".into(),
            code: "out_of_range",
            message: "Must not be greater than quizTotal.".into(),
        });
    }
    if !errors.is_empty() {
        return ApiError::invalid(errors).into_response(&id.0);
    }

    let Some(lesson_id) = lesson_id(&path) else {
        return ApiError::new(ErrorCode::NotFound).into_response(&id.0);
    };
    // The score is reported by the client (the quiz runs in the chat); the plan accepts that
    // trust for the MVP.
    let earned = quiz_stars(
        u32::try_from(req.quiz_correct).unwrap_or(u32::MAX),
        u32::try_from(req.quiz_total).unwrap_or(u32::MAX),
    );
    let now = state.clock.now();
    match state
        .lessons
        .complete(student.student_id, lesson_id, earned, now)
        .await
    {
        Ok((lesson, stars_earned)) => (
            StatusCode::OK,
            Json(CompleteLessonResponse {
                lesson: LessonDto::from(lesson),
                stars_earned,
            }),
        )
            .into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

/// Joins the upstream's chapters with the student's own lessons for one subject, by chapter id,
/// without copying either side.
pub(crate) fn chapters_with_progress<'a>(
    chapters: &'a [belajar_ports::upstream::Chapter],
    lessons: &'a [Lesson],
) -> Vec<(&'a belajar_ports::upstream::Chapter, Option<&'a Lesson>)> {
    let by_chapter: HashMap<&str, &Lesson> = lessons
        .iter()
        .map(|lesson| (lesson.chapter_id.as_str(), lesson))
        .collect();
    chapters
        .iter()
        .map(|chapter| (chapter, by_chapter.get(chapter.id.as_str()).copied()))
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::expect_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use belajar_ports::upstream::Chapter;

    use super::*;

    #[test]
    fn a_patch_body_rejects_unknown_fields_and_reads_camel_case() {
        let parsed: PatchLessonRequest =
            serde_json::from_value(serde_json::json!({ "step": "quiz" }))
                .expect("step is the only field");
        assert_eq!(parsed.step, "quiz");
        assert!(
            serde_json::from_value::<PatchLessonRequest>(serde_json::json!({
                "step": "quiz", "stars": 3
            }))
            .is_err(),
            "unknown fields are rejected"
        );
    }

    #[test]
    fn chapters_join_the_students_own_lessons_by_chapter_id() {
        let chapters = vec![
            Chapter {
                id: "bab-1".into(),
                number: 1,
                book_number: 1,
                title: "Satu".into(),
                page: Some(1),
                document_id: "dok".into(),
                document_title: "Buku".into(),
                topics: Vec::new(),
            },
            Chapter {
                id: "bab-2".into(),
                number: 2,
                book_number: 2,
                title: "Dua".into(),
                page: None,
                document_id: "dok".into(),
                document_title: "Buku".into(),
                topics: Vec::new(),
            },
        ];
        let lesson = Lesson {
            id: Uuid::now_v7(),
            subject_id: "ipa".into(),
            chapter_id: "bab-2".into(),
            chapter_title: "Dua".into(),
            conversation_id: Uuid::now_v7(),
            step: belajar_ports::lessons::LessonStep::Quiz,
            stars: 2,
            completed_at: None,
            updated_at: chrono::Utc::now(),
        };
        let joined_lessons = [lesson.clone()];
        let joined = chapters_with_progress(&chapters, &joined_lessons);
        assert_eq!(joined.len(), 2);
        assert!(joined[0].1.is_none(), "bab-1 has no lesson yet");
        assert_eq!(
            joined[1].1,
            Some(&lesson),
            "bab-2 is the student's own lesson"
        );
    }
}
