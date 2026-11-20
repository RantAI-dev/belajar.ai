//! The wire shapes of `GET /v1/progress`. Built by `progress.rs`; nothing here reads the store.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::wire::instant;
use crate::wire::mastery::{MasteryLevelDto, TrendDto};

/// The headline numbers for one period ("Minggu ini" or "Semua").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PeriodSummaryDto {
    /// Stars earned in the period.
    pub stars: u32,
    /// Chapters completed for the first time in the period.
    pub lessons_completed: u32,
    pub questions_answered: u32,
    pub correct_answers: u32,
    /// Round(100 × correct / answered); `null` when nothing was answered.
    #[schema(required)]
    pub accuracy_percent: Option<u32>,
    /// Jakarta days with any activity in the period.
    pub days_active: u32,
}

/// One practised topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicProgressDto {
    pub subject_id: String,
    pub topic: String,
    /// All answered questions on this topic, ever (the same numbers Latihan shows).
    pub attempts: u32,
    pub correct: u32,
    pub mastery_percent: u32,
    pub level: MasteryLevelDto,
    pub trend: TrendDto,
    /// This Jakarta week's accuracy; `null` without answers this week.
    #[schema(required)]
    pub this_week_percent: Option<u32>,
    /// Last Jakarta week's accuracy; `null` without answers last week.
    #[schema(required)]
    pub last_week_percent: Option<u32>,
    #[serde(with = "instant::option")]
    #[schema(value_type = Option<String>, format = DateTime, required)]
    pub last_practiced_at: Option<DateTime<Utc>>,
}

/// One chapter the student started in Belajar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChapterProgressDto {
    pub lesson_id: Uuid,
    pub subject_id: String,
    pub chapter_id: String,
    pub chapter_title: String,
    /// `explain`, `check`, `quiz` or `done`.
    pub step: String,
    pub stars: u8,
    pub is_completed: bool,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// One subject the student has practised or studied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubjectProgressDto {
    pub subject_id: String,
    pub name: String,
    pub attempts: u32,
    pub correct: u32,
    pub mastery_percent: u32,
    pub level: MasteryLevelDto,
    pub lessons_started: u32,
    pub lessons_completed: u32,
    /// Weakest first.
    pub topics: Vec<TopicProgressDto>,
    /// Most recently updated first.
    pub chapters: Vec<ChapterProgressDto>,
}

/// One Jakarta calendar day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DayActivityDto {
    #[schema(value_type = String, format = Date)]
    pub on: NaiveDate,
    pub is_active: bool,
    pub questions_answered: u32,
    pub correct_answers: u32,
    pub lessons_completed: u32,
    pub questions_asked: u32,
}

/// The last N Jakarta days, today included, oldest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivityWindowDto {
    #[schema(value_type = String, format = Date)]
    pub starts_on: NaiveDate,
    #[schema(value_type = String, format = Date)]
    pub ends_on: NaiveDate,
    pub days_active: u32,
    pub questions_answered: u32,
    pub lessons_completed: u32,
    pub questions_asked: u32,
    pub days: Vec<DayActivityDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDto {
    pub last7_days: ActivityWindowDto,
    pub last30_days: ActivityWindowDto,
}

/// What a recent session was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionKindDto {
    Practice,
    Lesson,
    Ask,
}

/// One recent session, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecentSessionDto {
    pub kind: SessionKindDto,
    /// The practice set, lesson or conversation.
    pub id: Uuid,
    #[schema(required)]
    pub subject_id: Option<String>,
    pub title: String,
    /// Practice only.
    #[schema(required)]
    pub correct_count: Option<u32>,
    /// Practice only.
    #[schema(required)]
    pub total_count: Option<u32>,
    /// Lesson only.
    #[schema(required)]
    pub stars: Option<u8>,
    #[serde(with = "instant")]
    #[schema(format = DateTime)]
    pub occurred_at: DateTime<Utc>,
}

/// The whole Progres page in one response, for the signed-in student only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProgressDto {
    pub streak_days: u32,
    /// The Jakarta week: Monday to Sunday.
    #[schema(value_type = String, format = Date)]
    pub week_starts_on: NaiveDate,
    #[schema(value_type = String, format = Date)]
    pub week_ends_on: NaiveDate,
    pub this_week: PeriodSummaryDto,
    pub all_time: PeriodSummaryDto,
    /// In the app's subject order.
    pub subjects: Vec<SubjectProgressDto>,
    pub activity: ActivityDto,
    pub recent_sessions: Vec<RecentSessionDto>,
    /// The weakest topics not yet mastered, weakest first (at most 3).
    pub practice_again: Vec<TopicProgressDto>,
    /// Unfinished chapters, most recent first (at most 3).
    pub continue_lessons: Vec<ChapterProgressDto>,
}
