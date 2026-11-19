//! `GET /v1/home` (Beranda): the student's stars, streak, today's missions and what to continue.
//! Progress is counted in the store; the mission titles and caps live here, at the edge.

use axum::Extension;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use belajar_core::calendar;
use belajar_ports::home::{ContinueItem, HomeData, MissionKind};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::RequestId;
use crate::api_error::ErrorBody;
use crate::current_student::CurrentStudent;
use crate::state::AppState;
use crate::store_errors;
use crate::wire::instant;

/// The target of every mission: one thing today.
const TARGET: u32 = 1;

/// Stars for completing any daily mission.
const REWARD_STARS: u32 = 10;

/// One daily mission, with today's progress.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MissionDto {
    /// `ask`, `lesson` or `practice`.
    pub kind: String,
    pub title: String,
    /// Capped at the target.
    pub progress: u32,
    pub target: u32,
    pub is_completed: bool,
    pub reward_stars: u32,
}

/// One unfinished lesson to continue, most recently updated first.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContinueItemDto {
    pub lesson_id: uuid::Uuid,
    pub subject_id: String,
    pub chapter_id: String,
    pub chapter_title: String,
    pub step: String,
    pub stars: u8,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

impl From<ContinueItem> for ContinueItemDto {
    fn from(item: ContinueItem) -> Self {
        Self {
            lesson_id: item.lesson_id,
            subject_id: item.subject_id,
            chapter_id: item.chapter_id,
            chapter_title: item.chapter_title,
            step: item.step.as_str().to_owned(),
            stars: item.stars,
            updated_at: item.updated_at,
        }
    }
}

/// The home screen, everything in one response.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HomeDto {
    pub nickname: String,
    pub grade: u8,
    pub stars: u32,
    pub streak_days: u32,
    pub missions: Vec<MissionDto>,
    pub continue_learning: Vec<ContinueItemDto>,
}

fn mission(kind: MissionKind, data: &HomeData) -> MissionDto {
    let count = match kind {
        MissionKind::Ask => data.mission_counts.ask,
        MissionKind::Lesson => data.mission_counts.lesson,
        MissionKind::Practice => data.mission_counts.practice,
    };
    // The plan fixes the three missions, their order and their titles.
    let title = match kind {
        MissionKind::Ask => "Tanya 1 pertanyaan",
        MissionKind::Lesson => "Selesaikan 1 bab",
        MissionKind::Practice => "Selesaikan 1 latihan",
    };
    let progress = count.min(TARGET);
    MissionDto {
        kind: kind.as_str().to_owned(),
        title: title.to_owned(),
        progress,
        target: TARGET,
        is_completed: progress >= TARGET,
        reward_stars: REWARD_STARS,
    }
}

/// `GET /v1/home`.
#[utoipa::path(
    get,
    path = "/v1/home",
    responses(
        (status = 200, body = HomeDto),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn get_home(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
) -> Response {
    let account = match state.accounts.get_student(student.student_id).await {
        Ok(account) => account,
        Err(err) => return store_errors::into_response(err, &id.0),
    };
    // "Today" is the Jakarta calendar date of the clock's instant, and the store reads activity
    // days by the same rule.
    let now = state.clock.now();
    let today = calendar::jakarta_date(now);
    let data = match state.home.home(student.student_id, today, now).await {
        Ok(data) => data,
        Err(err) => return store_errors::into_response(err, &id.0),
    };

    let body = HomeDto {
        nickname: account.nickname,
        grade: account.grade.get(),
        stars: data.stars,
        streak_days: data.streak_days,
        missions: MissionKind::ALL
            .iter()
            .map(|&kind| mission(kind, &data))
            .collect(),
        continue_learning: data
            .continue_learning
            .into_iter()
            .map(ContinueItemDto::from)
            .collect(),
    };
    (StatusCode::OK, Json(body)).into_response()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::expect_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use belajar_ports::home::MissionCounts;

    use super::*;

    fn home_data(counts: MissionCounts) -> HomeData {
        HomeData {
            stars: 0,
            streak_days: 0,
            mission_counts: counts,
            continue_learning: Vec::new(),
        }
    }

    #[test]
    fn missions_come_in_the_fixed_order_with_capped_progress() {
        let data = home_data(MissionCounts {
            ask: 0,
            lesson: 3,
            practice: 1,
        });
        let missions: Vec<MissionDto> = MissionKind::ALL
            .iter()
            .map(|&kind| mission(kind, &data))
            .collect();
        let kinds: Vec<&str> = missions.iter().map(|m| m.kind.as_str()).collect();
        assert_eq!(kinds, vec!["ask", "lesson", "practice"]);
        assert_eq!(missions[0].progress, 0);
        assert!(!missions[0].is_completed);
        assert_eq!(missions[1].progress, 1, "progress is capped at the target");
        assert!(missions[1].is_completed);
        assert!(missions[2].is_completed);
        for m in &missions {
            assert_eq!(m.target, 1);
            assert_eq!(m.reward_stars, 10);
            assert!(!m.title.is_empty());
        }
    }
}
