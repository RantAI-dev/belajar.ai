//! The Progres read: everything the progress page counts, for one student, in the windows the
//! caller's clock decided. Raw counts only; levels, trends and streaks are `belajar_core` rules
//! applied by the server. No HTTP, no SQL: those live in `belajar-server` and `belajar-store-pg`.

use std::collections::BTreeSet;

use async_trait::async_trait;
use belajar_core::period::Window;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::lessons::LessonStep;
use crate::store::StoreError;

/// Answered questions and how many of them were correct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tally {
    pub correct: u32,
    pub attempts: u32,
}

/// The windows one progress read counts in, all from the caller's clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressWindows {
    pub this_week: Window,
    pub last_week: Window,
    /// The longest activity window (the last 30 days); per-day rows start at its first day.
    pub activity: Window,
}

/// One practised topic: all-time answers (the `topic_mastery` row) and this and last week's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicProgress {
    pub subject_id: String,
    pub topic: String,
    pub total: Tally,
    pub this_week: Tally,
    pub last_week: Tally,
    pub last_practiced_at: Option<DateTime<Utc>>,
}

/// One chapter the student started in Belajar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterProgress {
    pub lesson_id: Uuid,
    pub subject_id: String,
    pub chapter_id: String,
    /// The lesson conversation's title; the web app prefers the chapters list's title.
    pub chapter_title: String,
    pub step: LessonStep,
    pub stars: u8,
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// What happened on one Jakarta calendar day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayActivity {
    pub on: NaiveDate,
    pub answers: Tally,
    pub lessons_completed: u32,
    pub questions_asked: u32,
}

/// The kind of a recent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Practice,
    Lesson,
    Ask,
}

/// One recent session: a submitted practice set, a lesson worked on, or a Tanya conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentSession {
    pub kind: SessionKind,
    pub id: Uuid,
    pub subject_id: Option<String>,
    pub title: String,
    /// Practice only: the score.
    pub score: Option<Tally>,
    /// Lesson only: the best stars so far.
    pub stars: Option<u8>,
    pub occurred_at: DateTime<Utc>,
}

/// Everything the progress page counts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProgressData {
    pub stars_total: u32,
    pub stars_this_week: u32,
    pub answers_total: Tally,
    pub answers_this_week: Tally,
    /// Every Jakarta date the student did anything (the same rule the home streak uses).
    pub active_days: BTreeSet<NaiveDate>,
    /// Sorted by subject id, then topic.
    pub topics: Vec<TopicProgress>,
    /// Most recently updated first.
    pub chapters: Vec<ChapterProgress>,
    /// Only days inside `ProgressWindows::activity` that had answers, lessons or questions.
    pub days: Vec<DayActivity>,
    /// Newest first, at most `RECENT_SESSIONS`.
    pub recent: Vec<RecentSession>,
}

/// How many recent sessions a progress read returns.
pub const RECENT_SESSIONS: usize = 8;

#[async_trait]
pub trait ProgressStore: Send + Sync {
    /// Reads the student's progress. Every row read is this student's; another student's rows
    /// never appear in any number.
    async fn progress(
        &self,
        student_id: Uuid,
        windows: &ProgressWindows,
    ) -> Result<ProgressData, StoreError>;
}
