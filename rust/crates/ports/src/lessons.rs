//! A student's guided lessons (the Belajar slice): one `lesson_progress` row per chapter, with
//! the learn conversation its chat runs in. No HTTP, no SQL: those live in `belajar-server` and
//! `belajar-store-pg`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::store::StoreError;

/// Where a student is in a chapter's guided session. `Done` is reached only by completing a quiz,
/// never by `set_step`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LessonStep {
    Explain,
    Check,
    Quiz,
    Done,
}

impl LessonStep {
    /// The only values the database's CHECK and the wire allow.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "explain" => Some(Self::Explain),
            "check" => Some(Self::Check),
            "quiz" => Some(Self::Quiz),
            "done" => Some(Self::Done),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Explain => "explain",
            Self::Check => "check",
            Self::Quiz => "quiz",
            Self::Done => "done",
        }
    }
}

/// One guided lesson. `chapter_title` is the conversation's title, written once at creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lesson {
    pub id: Uuid,
    pub subject_id: String,
    pub chapter_id: String,
    pub chapter_title: String,
    pub conversation_id: Uuid,
    pub step: LessonStep,
    /// The best quiz result so far, 0 to 3.
    pub stars: u8,
    /// Set the first time the lesson was completed, never changed after.
    pub completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
pub trait LessonStore: Send + Sync {
    /// The student's lesson by id. Another student's id is `NotFound`, like a missing one.
    async fn get(&self, student_id: Uuid, lesson_id: Uuid) -> Result<Lesson, StoreError>;

    /// Every lesson the student started for one subject, for the chapter map.
    async fn list_by_subject(
        &self,
        student_id: Uuid,
        subject_id: &str,
    ) -> Result<Vec<Lesson>, StoreError>;

    /// Starts the lesson: creates its learn conversation and its progress row in one transaction,
    /// or returns the existing lesson unchanged. `created` is false when it already existed.
    async fn start(
        &self,
        student_id: Uuid,
        subject_id: &str,
        chapter_id: &str,
        chapter_title: &str,
        now: DateTime<Utc>,
    ) -> Result<(Lesson, bool), StoreError>;

    /// Moves the lesson to `step`. Only `Explain`, `Check` and `Quiz` arrive here: the caller
    /// rejects `Done`, which only completing reaches.
    async fn set_step(
        &self,
        student_id: Uuid,
        lesson_id: Uuid,
        step: LessonStep,
        now: DateTime<Utc>,
    ) -> Result<Lesson, StoreError>;

    /// Applies one quiz result: keeps the best stars, sets `Done`, keeps the first
    /// `completed_at`, and awards a `star_ledger` row of 5 stars for every star level newly
    /// reached. Returns the lesson and the stars earned by this call.
    async fn complete(
        &self,
        student_id: Uuid,
        lesson_id: Uuid,
        quiz_stars: u8,
        now: DateTime<Utc>,
    ) -> Result<(Lesson, u32), StoreError>;
}
