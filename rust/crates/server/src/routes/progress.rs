//! `GET /v1/progress` (Progres): the signed-in student's summary, mastery per subject and topic
//! with this week's trend, activity, recent sessions and what to do next. The windows come from
//! the clock through `belajar_core::period`; levels and trends are `belajar_core::mastery`'s.
//! The endpoint takes no student id: the session decides whose numbers these are.

use std::cmp::Ordering;

use axum::Extension;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use belajar_core::calendar::jakarta_date;
use belajar_core::mastery::{mastery_level, mastery_percent, trend};
use belajar_core::period::{self, Window};
use belajar_core::streak::streak_days;
use belajar_core::subject::{self, SUBJECTS};
use belajar_ports::lessons::LessonStep;
use belajar_ports::progress::{
    ChapterProgress, ProgressData, ProgressWindows, RecentSession, SessionKind, Tally,
    TopicProgress,
};
use chrono::{DateTime, Utc};

use crate::RequestId;
use crate::api_error::ErrorBody;
use crate::current_student::CurrentStudent;
use crate::routes::progress_dto::{
    ActivityDto, ActivityWindowDto, ChapterProgressDto, DayActivityDto, PeriodSummaryDto,
    ProgressDto, RecentSessionDto, SessionKindDto, SubjectProgressDto, TopicProgressDto,
};
use crate::state::AppState;
use crate::store_errors;

/// How many topics "Latih lagi" and chapters "Lanjutkan" show.
const NEXT_STEPS: usize = 3;

/// `GET /v1/progress`.
#[utoipa::path(
    get,
    path = "/v1/progress",
    responses(
        (status = 200, body = ProgressDto),
        (status = 401, body = ErrorBody),
    )
)]
pub(crate) async fn get_progress(
    state: State<AppState>,
    Extension(id): Extension<RequestId>,
    student: CurrentStudent,
) -> Response {
    let now = state.clock.now();
    let windows = ProgressWindows {
        this_week: period::this_week(now),
        last_week: period::last_week(now),
        activity: period::last_days(now, 30),
    };
    match state.progress.progress(student.student_id, &windows).await {
        Ok(data) => (StatusCode::OK, Json(progress_dto(&data, &windows, now))).into_response(),
        Err(err) => store_errors::into_response(err, &id.0),
    }
}

fn percent_or_null(tally: Tally) -> Option<u32> {
    (tally.attempts > 0).then(|| mastery_percent(tally.correct, tally.attempts))
}

fn topic_dto(topic: &TopicProgress) -> TopicProgressDto {
    TopicProgressDto {
        subject_id: topic.subject_id.clone(),
        topic: topic.topic.clone(),
        attempts: topic.total.attempts,
        correct: topic.total.correct,
        mastery_percent: mastery_percent(topic.total.correct, topic.total.attempts),
        level: mastery_level(topic.total.correct, topic.total.attempts).into(),
        trend: trend(
            (topic.this_week.correct, topic.this_week.attempts),
            (topic.last_week.correct, topic.last_week.attempts),
        )
        .into(),
        this_week_percent: percent_or_null(topic.this_week),
        last_week_percent: percent_or_null(topic.last_week),
        last_practiced_at: topic.last_practiced_at,
    }
}

fn chapter_dto(chapter: &ChapterProgress) -> ChapterProgressDto {
    ChapterProgressDto {
        lesson_id: chapter.lesson_id,
        subject_id: chapter.subject_id.clone(),
        chapter_id: chapter.chapter_id.clone(),
        chapter_title: chapter.chapter_title.clone(),
        step: chapter.step.as_str().to_owned(),
        stars: chapter.stars,
        is_completed: chapter.step == LessonStep::Done,
        updated_at: chapter.updated_at,
    }
}

fn session_dto(session: &RecentSession) -> RecentSessionDto {
    RecentSessionDto {
        kind: match session.kind {
            SessionKind::Practice => SessionKindDto::Practice,
            SessionKind::Lesson => SessionKindDto::Lesson,
            SessionKind::Ask => SessionKindDto::Ask,
        },
        id: session.id,
        subject_id: session.subject_id.clone(),
        title: session.title.clone(),
        correct_count: session.score.map(|s| s.correct),
        total_count: session.score.map(|s| s.attempts),
        stars: session.stars,
        occurred_at: session.occurred_at,
    }
}

/// Weakest first: the lower level, then the lower percentage, then the topic with more answers
/// (a low score over many answers needs practice more than one unlucky set), then by name.
fn weakest_first(a: &TopicProgressDto, b: &TopicProgressDto) -> Ordering {
    a.level
        .cmp(&b.level)
        .then(a.mastery_percent.cmp(&b.mastery_percent))
        .then(b.attempts.cmp(&a.attempts))
        .then_with(|| a.topic.cmp(&b.topic))
}

fn completed_in(chapter: &ChapterProgress, window: &Window) -> bool {
    chapter.completed_at.is_some_and(|at| window.contains(at))
}

fn subjects(data: &ProgressData) -> Vec<SubjectProgressDto> {
    let mut ids: Vec<&str> = data
        .topics
        .iter()
        .map(|t| t.subject_id.as_str())
        .chain(data.chapters.iter().map(|c| c.subject_id.as_str()))
        .collect();
    // The app's subject order; an id we do not know (an older row) sorts last.
    let rank = |id: &str| {
        SUBJECTS
            .iter()
            .position(|s| s.id == id)
            .unwrap_or(SUBJECTS.len())
    };
    ids.sort_by(|a, b| rank(a).cmp(&rank(b)).then(a.cmp(b)));
    ids.dedup();
    ids.into_iter()
        .map(|subject_id| {
            let mut topics: Vec<TopicProgressDto> = data
                .topics
                .iter()
                .filter(|t| t.subject_id == subject_id)
                .map(topic_dto)
                .collect();
            topics.sort_by(weakest_first);
            let chapters: Vec<ChapterProgressDto> = data
                .chapters
                .iter()
                .filter(|c| c.subject_id == subject_id)
                .map(chapter_dto)
                .collect();
            let correct = topics.iter().map(|t| t.correct).sum();
            let attempts = topics.iter().map(|t| t.attempts).sum();
            SubjectProgressDto {
                subject_id: subject_id.to_owned(),
                name: subject::by_id(subject_id)
                    .map_or(subject_id, |s| s.name)
                    .to_owned(),
                attempts,
                correct,
                mastery_percent: mastery_percent(correct, attempts),
                level: mastery_level(correct, attempts).into(),
                lessons_started: u32::try_from(chapters.len()).unwrap_or(u32::MAX),
                lessons_completed: u32::try_from(
                    chapters.iter().filter(|c| c.is_completed).count(),
                )
                .unwrap_or(u32::MAX),
                topics,
                chapters,
            }
        })
        .collect()
}

fn count_days(data: &ProgressData, window: &Window) -> u32 {
    let n = data
        .active_days
        .range(window.first_day..=window.last_day)
        .count();
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn activity_window(data: &ProgressData, window: &Window) -> ActivityWindowDto {
    let days: Vec<DayActivityDto> = window
        .dates()
        .into_iter()
        .map(|on| {
            let day = data.days.iter().find(|d| d.on == on);
            DayActivityDto {
                on,
                is_active: data.active_days.contains(&on),
                questions_answered: day.map_or(0, |d| d.answers.attempts),
                correct_answers: day.map_or(0, |d| d.answers.correct),
                lessons_completed: day.map_or(0, |d| d.lessons_completed),
                questions_asked: day.map_or(0, |d| d.questions_asked),
            }
        })
        .collect();
    ActivityWindowDto {
        starts_on: window.first_day,
        ends_on: window.last_day,
        days_active: count_days(data, window),
        questions_answered: days.iter().map(|d| d.questions_answered).sum(),
        lessons_completed: days.iter().map(|d| d.lessons_completed).sum(),
        questions_asked: days.iter().map(|d| d.questions_asked).sum(),
        days,
    }
}

fn period_summary(
    stars: u32,
    lessons: usize,
    answers: Tally,
    days_active: u32,
) -> PeriodSummaryDto {
    PeriodSummaryDto {
        stars,
        lessons_completed: u32::try_from(lessons).unwrap_or(u32::MAX),
        questions_answered: answers.attempts,
        correct_answers: answers.correct,
        accuracy_percent: percent_or_null(answers),
        days_active,
    }
}

pub(crate) fn progress_dto(
    data: &ProgressData,
    windows: &ProgressWindows,
    now: DateTime<Utc>,
) -> ProgressDto {
    let today = jakarta_date(now);
    let week = &windows.this_week;
    let subjects = subjects(data);
    let mut practice_again: Vec<TopicProgressDto> = subjects
        .iter()
        .flat_map(|s| s.topics.iter())
        .filter(|t| t.level.needs_practice())
        .cloned()
        .collect();
    practice_again.sort_by(weakest_first);
    practice_again.truncate(NEXT_STEPS);
    let continue_lessons: Vec<ChapterProgressDto> = data
        .chapters
        .iter()
        .filter(|c| c.step != LessonStep::Done)
        .take(NEXT_STEPS)
        .map(chapter_dto)
        .collect();
    ProgressDto {
        streak_days: streak_days(&data.active_days, today),
        week_starts_on: week.first_day,
        week_ends_on: week.last_day,
        this_week: period_summary(
            data.stars_this_week,
            data.chapters
                .iter()
                .filter(|c| completed_in(c, week))
                .count(),
            data.answers_this_week,
            count_days(data, week),
        ),
        all_time: period_summary(
            data.stars_total,
            data.chapters
                .iter()
                .filter(|c| c.completed_at.is_some())
                .count(),
            data.answers_total,
            u32::try_from(data.active_days.len()).unwrap_or(u32::MAX),
        ),
        subjects,
        activity: ActivityDto {
            last7_days: activity_window(data, &period::last_days(now, 7)),
            last30_days: activity_window(data, &windows.activity),
        },
        recent_sessions: data.recent.iter().map(session_dto).collect(),
        practice_again,
        continue_lessons,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::wire::mastery::{MasteryLevelDto, TrendDto};

    fn at(text: &str) -> DateTime<Utc> {
        text.parse().unwrap()
    }

    fn topic(subject: &str, name: &str, correct: u32, attempts: u32) -> TopicProgress {
        TopicProgress {
            subject_id: subject.into(),
            topic: name.into(),
            total: Tally { correct, attempts },
            this_week: Tally::default(),
            last_week: Tally::default(),
            last_practiced_at: None,
        }
    }

    fn windows(now: DateTime<Utc>) -> ProgressWindows {
        ProgressWindows {
            this_week: period::this_week(now),
            last_week: period::last_week(now),
            activity: period::last_days(now, 30),
        }
    }

    #[test]
    fn an_empty_student_gets_zeros_and_empty_lists() {
        let now = at("2026-11-19T03:00:00Z");
        let dto = progress_dto(&ProgressData::default(), &windows(now), now);
        assert_eq!(dto.streak_days, 0);
        assert_eq!(dto.this_week.accuracy_percent, None);
        assert_eq!(dto.all_time.questions_answered, 0);
        assert!(dto.subjects.is_empty());
        assert!(dto.practice_again.is_empty());
        assert!(dto.continue_lessons.is_empty());
        assert_eq!(dto.activity.last7_days.days.len(), 7);
        assert_eq!(dto.activity.last30_days.days.len(), 30);
        assert_eq!(
            dto.week_starts_on,
            NaiveDate::from_ymd_opt(2026, 11, 16).unwrap()
        );
    }

    #[test]
    fn practice_again_is_the_weakest_unmastered_topics_first() {
        let now = at("2026-11-19T03:00:00Z");
        let data = ProgressData {
            topics: vec![
                topic("ipa", "Mastered", 9, 10),
                topic("ipa", "Almost", 7, 10),
                topic("mtk", "Weak", 2, 10),
                topic("mtk", "Weaker", 1, 10),
                topic("ips", "Lucky", 1, 1),
            ],
            ..ProgressData::default()
        };
        let dto = progress_dto(&data, &windows(now), now);
        let names: Vec<&str> = dto
            .practice_again
            .iter()
            .map(|t| t.topic.as_str())
            .collect();
        assert_eq!(names, vec!["Weaker", "Weak", "Almost"]);
        assert!(
            dto.practice_again
                .iter()
                .all(|t| t.level != MasteryLevelDto::Mastered)
        );
        // Subjects come in the app's order with their own aggregate level.
        let ids: Vec<&str> = dto.subjects.iter().map(|s| s.subject_id.as_str()).collect();
        assert_eq!(ids, vec!["ipa", "mtk", "ips"]);
        assert_eq!(dto.subjects[0].level, MasteryLevelDto::Mastered, "16/20");
        assert_eq!(dto.subjects[1].level, MasteryLevelDto::Learning, "3/20");
        assert_eq!(dto.subjects[0].name, "IPA");
    }

    #[test]
    fn a_topic_trend_compares_this_week_with_last_week() {
        let now = at("2026-11-19T03:00:00Z");
        let mut t = topic("ipa", "Pemuaian", 6, 10);
        t.this_week = Tally {
            correct: 4,
            attempts: 5,
        };
        t.last_week = Tally {
            correct: 2,
            attempts: 5,
        };
        let data = ProgressData {
            topics: vec![t],
            ..ProgressData::default()
        };
        let dto = progress_dto(&data, &windows(now), now);
        let row = &dto.subjects[0].topics[0];
        assert_eq!(row.trend, TrendDto::Up);
        assert_eq!(row.this_week_percent, Some(80));
        assert_eq!(row.last_week_percent, Some(40));
    }
}
