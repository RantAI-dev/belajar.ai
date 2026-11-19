//! The Beranda (home) read: stars, streak, today's missions and what to continue. Progress is
//! derived from activity rows; `daily_missions` rows exist only so a reward is given once.
//! No HTTP, no SQL: those live in `belajar-server` and `belajar-store-pg`.

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::lessons::LessonStep;
use crate::store::StoreError;

/// The three daily missions, always in this order on the home screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionKind {
    /// "Tanya 1 pertanyaan": a user message today in an ask-mode conversation.
    Ask,
    /// "Selesaikan 1 bab": a lesson completed today.
    Lesson,
    /// "Selesaikan 1 latihan": a practice set submitted today.
    Practice,
}

impl MissionKind {
    /// The three missions, in display order.
    pub const ALL: [MissionKind; 3] =
        [MissionKind::Ask, MissionKind::Lesson, MissionKind::Practice];

    /// The value stored in `daily_missions.kind` and shown on the wire.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Lesson => "lesson",
            Self::Practice => "practice",
        }
    }
}

/// Today's raw progress for the three missions, before capping at the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MissionCounts {
    pub ask: u32,
    pub lesson: u32,
    pub practice: u32,
}

/// One unfinished lesson to continue, most recently updated first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinueItem {
    pub lesson_id: Uuid,
    pub subject_id: String,
    pub chapter_id: String,
    pub chapter_title: String,
    pub step: LessonStep,
    pub stars: u8,
    pub updated_at: DateTime<Utc>,
}

/// Everything the home screen shows that is not the student's own profile.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HomeData {
    /// The student's star total, including any mission reward this call just awarded.
    pub stars: u32,
    pub streak_days: u32,
    pub mission_counts: MissionCounts,
    pub continue_learning: Vec<ContinueItem>,
}

#[async_trait]
pub trait HomeStore: Send + Sync {
    /// Reads the home screen for one student. `today` is the Jakarta date of `now`, both from the
    /// caller's clock. A mission whose count has reached its target and has no `daily_missions`
    /// row for (today, kind) gets that row and its 10-star `star_ledger` reward in this same
    /// call, idempotently; `stars` is summed after them.
    async fn home(
        &self,
        student_id: Uuid,
        today: NaiveDate,
        now: DateTime<Utc>,
    ) -> Result<HomeData, StoreError>;
}
