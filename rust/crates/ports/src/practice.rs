//! The Latihan slice's stored data: practice sets (answer keys included, so they stay
//! server-side), the answers of one submission, and topic mastery. No HTTP, no SQL: those live in
//! `belajar-server` and `belajar-store-pg`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::store::StoreError;
use crate::upstream::{PracticeSet, PracticeType};

/// A set's lifecycle: stored `ready` once generation delivered it, `submitted` exactly once when
/// it is graded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PracticeSetStatus {
    Ready,
    Submitted,
}

/// One stored answer of one submission, with its grade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradedAnswer {
    pub question_id: String,
    pub selected: Vec<String>,
    pub is_correct: bool,
}

/// A generated set on its way into the database. `practice` carries the answer keys, so it is
/// never serialized to a client as is.
#[derive(Debug)]
pub struct NewPracticeSet<'a> {
    pub subject_id: &'a str,
    pub topic: &'a str,
    pub practice_type: PracticeType,
    pub total_questions: u16,
    pub practice: &'a PracticeSet,
    pub created_at: DateTime<Utc>,
}

/// One row of `practice_sets`: the stored practice set is the whole upstream result, questions
/// with answer keys and the raw JSON grading needs.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredPracticeSet {
    pub id: Uuid,
    pub subject_id: String,
    pub topic: String,
    pub practice_type: PracticeType,
    pub total_questions: u16,
    pub status: PracticeSetStatus,
    pub practice: PracticeSet,
    pub score_correct: Option<u16>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// One row as the list shows it: nothing from the stored practice JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct PracticeSetSummary {
    pub id: Uuid,
    pub subject_id: String,
    pub topic: String,
    pub practice_type: PracticeType,
    pub total_questions: u16,
    pub status: PracticeSetStatus,
    pub score_correct: Option<u16>,
    pub created_at: DateTime<Utc>,
}

/// One row of `practice_answers`. Join to the set's questions by `question_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPracticeAnswer {
    pub question_id: String,
    pub selected: Vec<String>,
    pub is_correct: bool,
}

/// Everything one submission writes. One call, one transaction: the answer rows, the set's new
/// status and score, the topic mastery upsert and, when `stars_earned` > 0, one star ledger row.
#[derive(Debug)]
pub struct SubmitRecord {
    pub student_id: Uuid,
    pub practice_set_id: Uuid,
    pub subject_id: String,
    pub topic: String,
    /// How many questions were graded (the set's total).
    pub attempts: u32,
    /// How many of them were correct.
    pub correct: u32,
    pub stars_earned: u32,
    pub submitted_at: DateTime<Utc>,
    pub answers: Vec<GradedAnswer>,
}

/// One `topic_mastery` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicMasteryEntry {
    pub subject_id: String,
    pub topic: String,
    pub attempts: u32,
    pub correct: u32,
}

#[async_trait]
pub trait PracticeStore: Send + Sync {
    /// Stores one generated set as `ready`. The caller's clock made `created_at`.
    async fn create_set(
        &self,
        student_id: Uuid,
        set: &NewPracticeSet<'_>,
    ) -> Result<StoredPracticeSet, StoreError>;

    /// One of this student's sets, or `None` when it does not exist or is another student's.
    async fn get_set(
        &self,
        student_id: Uuid,
        set_id: Uuid,
    ) -> Result<Option<StoredPracticeSet>, StoreError>;

    /// This student's sets, newest first with a tie-break on id, at most `limit` rows, from
    /// `(created_at, id)` onwards when `after` is given.
    async fn list_sets(
        &self,
        student_id: Uuid,
        limit: u32,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<Vec<PracticeSetSummary>, StoreError>;

    /// The stored answers of one submitted set. No rows while the set is `ready`.
    async fn answers(
        &self,
        student_id: Uuid,
        set_id: Uuid,
    ) -> Result<Vec<StoredPracticeAnswer>, StoreError>;

    /// Writes one whole submission. `StoreError::Conflict` when the set is already submitted,
    /// `NotFound` when it is not this student's.
    async fn submit_set(&self, record: &SubmitRecord) -> Result<(), StoreError>;

    /// The student's mastery rows, sorted by subject id then topic.
    async fn list_topic_mastery(
        &self,
        student_id: Uuid,
        subject_id: Option<&str>,
    ) -> Result<Vec<TopicMasteryEntry>, StoreError>;
}
