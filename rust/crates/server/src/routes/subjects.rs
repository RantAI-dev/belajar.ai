//! The Belajar routes for subjects and their chapter maps: the upstream's mapel under our ids,
//! and each subject's chapters joined with the student's own progress.

use axum::Extension;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use belajar_core::subject;
use belajar_ports::lessons::Lesson;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::RequestId;
use crate::api_error::{ApiError, ErrorBody};
use crate::current_student::CurrentStudent;
use crate::error_codes::ErrorCode;
use crate::routes::lessons::chapters_with_progress;
use crate::state::AppState;
use crate::store_errors;
use crate::upstream_errors;

/// One subject, under our id, with the upstream's name passed back on later requests.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubjectDto {
    pub id: String,
    pub name: String,
    /// The upstream's own name for this subject ("mapel"), e.g. as sent on chat requests.
    pub mapel: String,
    pub document_count: u32,
}

/// One topic ("materi") listed inside a chapter, from the book's table of contents.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChapterTopicDto {
    pub title: String,
    /// 0-based page where the topic starts; `null` when the upstream has no page.
    #[schema(required)]
    pub page: Option<u32>,
}

/// One chapter of a subject, with the student's own progress in it.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChapterDto {
    pub id: String,
    /// Position in the subject's chapter path, 1-based.
    pub number: u32,
    /// The number printed in the book; may repeat when a subject has several books.
    pub book_number: u32,
    pub title: String,
    /// 0-based page where the chapter starts; `null` when the upstream has no page.
    #[schema(required)]
    pub page: Option<u32>,
    pub document_id: String,
    pub document_title: String,
    pub topics: Vec<ChapterTopicDto>,
    /// The best stars the student's quizzes earned in this chapter, 0 when none.
    pub stars: u8,
    /// `null` before the student starts the chapter's lesson, else its current step.
    #[schema(required)]
    pub step: Option<String>,
    pub is_completed: bool,
    /// The student's lesson for this chapter, when they started one.
    #[schema(required)]
    pub lesson_id: Option<Uuid>,
}

/// The subjects list: `{ data, nextCursor }`, one complete page.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubjectsDto {
    pub data: Vec<SubjectDto>,
    #[schema(required)]
    pub next_cursor: Option<String>,
}

/// The chapters list: `{ data, nextCursor }`, one complete page.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChaptersDto {
    pub data: Vec<ChapterDto>,
    #[schema(required)]
    pub next_cursor: Option<String>,
}

/// `GET /v1/subjects`: the upstream's mapel for the student's grade, in our fixed order, under
/// our ids. A mapel we do not know is left out.
#[utoipa::path(
    get,
    path = "/v1/subjects",
    responses(
        (status = 200, body = SubjectsDto),
        (status = 401, body = ErrorBody),
        (status = 502, body = ErrorBody),
        (status = 503, body = ErrorBody),
        (status = 504, body = ErrorBody),
    )
)]
pub(crate) async fn list_subjects(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
) -> Response {
    let account = match state.accounts.get_student(student.student_id).await {
        Ok(account) => account,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let mapel = match state.upstream.mapel(account.grade).await {
        Ok(mapel) => mapel,
        Err(err) => return upstream_errors::into_response(&err, &id.0),
    };
    // Our order is the list's order; the first upstream match for a subject wins.
    let data: Vec<SubjectDto> = subject::SUBJECTS
        .iter()
        .filter_map(|subject| {
            let entry = mapel
                .iter()
                .find(|entry| subject::by_mapel(&entry.name) == Some(subject))?;
            Some(SubjectDto {
                id: subject.id.to_owned(),
                name: subject.name.to_owned(),
                mapel: entry.name.clone(),
                document_count: entry.document_count,
            })
        })
        .collect();
    (
        StatusCode::OK,
        Json(SubjectsDto {
            data,
            next_cursor: None,
        }),
    )
        .into_response()
}

/// `GET /v1/subjects/{subjectId}/chapters`: the chapter map, joined with the student's own
/// progress.
#[utoipa::path(
    get,
    path = "/v1/subjects/{subjectId}/chapters",
    params(("subjectId" = String, description = "One of our subject ids")),
    responses(
        (status = 200, body = ChaptersDto),
        (status = 401, body = ErrorBody),
        (status = 404, body = ErrorBody),
        (status = 502, body = ErrorBody),
        (status = 503, body = ErrorBody),
        (status = 504, body = ErrorBody),
    )
)]
pub(crate) async fn list_chapters(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
    Path(subject_id): Path<String>,
) -> Response {
    let Some(subject) = subject::by_id(&subject_id) else {
        return ApiError::new(ErrorCode::NotFound).into_response(&id.0);
    };
    let account = match state.accounts.get_student(student.student_id).await {
        Ok(account) => account,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    let chapters = match state.upstream.chapters(account.grade, subject.mapel).await {
        Ok(chapters) => chapters,
        Err(err) => return upstream_errors::into_response(&err, &id.0),
    };
    let lessons: Vec<Lesson> = match state
        .lessons
        .list_by_subject(student.student_id, subject.id)
        .await
    {
        Ok(lessons) => lessons,
        Err(err) => return store_errors::into_response(err, &id.0),
    };

    let data: Vec<ChapterDto> = chapters_with_progress(&chapters, &lessons)
        .into_iter()
        .map(|(chapter, lesson)| {
            let step = lesson.map(|lesson| lesson.step);
            ChapterDto {
                id: chapter.id.clone(),
                number: chapter.number,
                book_number: chapter.book_number,
                title: chapter.title.clone(),
                page: chapter.page,
                document_id: chapter.document_id.clone(),
                document_title: chapter.document_title.clone(),
                topics: chapter
                    .topics
                    .iter()
                    .map(|topic| ChapterTopicDto {
                        title: topic.title.clone(),
                        page: topic.page,
                    })
                    .collect(),
                stars: lesson.map_or(0, |lesson| lesson.stars),
                step: step.map(|step| step.as_str().to_owned()),
                is_completed: step == Some(belajar_ports::lessons::LessonStep::Done),
                lesson_id: lesson.map(|lesson| lesson.id),
            }
        })
        .collect();
    (
        StatusCode::OK,
        Json(ChaptersDto {
            data,
            next_cursor: None,
        }),
    )
        .into_response()
}
