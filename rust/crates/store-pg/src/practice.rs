//! `PracticeStore` on Postgres. One submission is one transaction: the answer rows, the set's
//! new status and score, the topic mastery upsert and the star ledger row all commit together or
//! not at all.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Postgres;
use sqlx::types::Json;
use uuid::Uuid;

use belajar_ports::practice::{
    NewPracticeSet, PracticeSetStatus, PracticeSetSummary, PracticeStore, StoredPracticeAnswer,
    StoredPracticeSet, SubmitRecord, TopicMasteryEntry,
};
use belajar_ports::store::StoreError;
use belajar_ports::upstream::{PracticeSet, PracticeType};

use crate::Db;

/// Every column of one `practice_sets` row, in the database's own types.
#[derive(Debug, sqlx::FromRow)]
struct SetRow {
    id: Uuid,
    subject_id: String,
    topic: String,
    practice_type: String,
    total_questions: i16,
    status: String,
    practice: Json<PracticeSet>,
    score_correct: Option<i16>,
    submitted_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct SummaryRow {
    id: Uuid,
    subject_id: String,
    topic: String,
    practice_type: String,
    total_questions: i16,
    status: String,
    score_correct: Option<i16>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct AnswerRow {
    question_id: String,
    selected: Json<Vec<String>>,
    is_correct: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct MasteryRow {
    subject_id: String,
    topic: String,
    attempts: i32,
    correct: i32,
}

// The columns' CHECK constraints already bound these values; the parse helpers only keep a
// drifted database from becoming a panic. Their text goes to the log, never to a response.

fn internal(cause: impl Into<String>) -> StoreError {
    StoreError::Internal(cause.into())
}

fn parse_practice_type(name: &str) -> Result<PracticeType, StoreError> {
    // The enum's serde names are exactly the column's CHECK values.
    match name {
        "mcq_normal" => Ok(PracticeType::McqNormal),
        "mcq_multiple_correct" => Ok(PracticeType::McqMultipleCorrect),
        "mcq_story" => Ok(PracticeType::McqStory),
        "mcq_mixed" => Ok(PracticeType::McqMixed),
        other => Err(internal(format!(
            "unknown practice_sets.practice_type: {other:?}"
        ))),
    }
}

fn practice_type_name(practice_type: PracticeType) -> &'static str {
    match practice_type {
        PracticeType::McqNormal => "mcq_normal",
        PracticeType::McqMultipleCorrect => "mcq_multiple_correct",
        PracticeType::McqStory => "mcq_story",
        PracticeType::McqMixed => "mcq_mixed",
    }
}

fn parse_status(name: &str) -> Result<PracticeSetStatus, StoreError> {
    match name {
        "ready" => Ok(PracticeSetStatus::Ready),
        "submitted" => Ok(PracticeSetStatus::Submitted),
        other => Err(internal(format!("unknown practice_sets.status: {other:?}"))),
    }
}

fn small_uint(column: &str, value: i16) -> Result<u16, StoreError> {
    u16::try_from(value).map_err(|_| internal(format!("{column} is negative")))
}

fn into_small(column: &str, value: u16) -> Result<i16, StoreError> {
    i16::try_from(value).map_err(|_| internal(format!("{column} does not fit the column")))
}

/// One place that turns a driver error into a `StoreError`; the text is for the log only.
fn map_db_err(err: sqlx::Error) -> StoreError {
    // The two unique indexes here both mean "this submission already exists".
    let already_submitted = err.as_database_error().is_some_and(|db| {
        db.is_unique_violation()
            && matches!(
                db.constraint(),
                Some("uq_practice_answers_practice_set_id_question_id")
                    | Some("uq_star_ledger_student_id_reason_ref_id")
            )
    });
    if already_submitted {
        StoreError::Conflict
    } else {
        StoreError::Internal(err.to_string())
    }
}

impl TryFrom<SetRow> for StoredPracticeSet {
    type Error = StoreError;

    fn try_from(row: SetRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            subject_id: row.subject_id,
            topic: row.topic,
            practice_type: parse_practice_type(&row.practice_type)?,
            total_questions: small_uint("total_questions", row.total_questions)?,
            status: parse_status(&row.status)?,
            practice: row.practice.0,
            score_correct: row
                .score_correct
                .map(|value| small_uint("score_correct", value))
                .transpose()?,
            submitted_at: row.submitted_at,
            created_at: row.created_at,
        })
    }
}

impl TryFrom<SummaryRow> for PracticeSetSummary {
    type Error = StoreError;

    fn try_from(row: SummaryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            subject_id: row.subject_id,
            topic: row.topic,
            practice_type: parse_practice_type(&row.practice_type)?,
            total_questions: small_uint("total_questions", row.total_questions)?,
            status: parse_status(&row.status)?,
            score_correct: row
                .score_correct
                .map(|value| small_uint("score_correct", value))
                .transpose()?,
            created_at: row.created_at,
        })
    }
}

impl TryFrom<MasteryRow> for TopicMasteryEntry {
    type Error = StoreError;

    fn try_from(row: MasteryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            subject_id: row.subject_id,
            topic: row.topic,
            attempts: u32::try_from(row.attempts)
                .map_err(|_| internal("topic_mastery.attempts is negative"))?,
            correct: u32::try_from(row.correct)
                .map_err(|_| internal("topic_mastery.correct is negative"))?,
        })
    }
}

#[async_trait]
impl PracticeStore for Db {
    async fn create_set(
        &self,
        student_id: Uuid,
        set: &NewPracticeSet<'_>,
    ) -> Result<StoredPracticeSet, StoreError> {
        // The stored practice set is our own type round-tripped through the column, answer keys
        // and all; `Json` does the (de)serialization sqlx already ships with.
        let row = sqlx::query_as::<Postgres, SetRow>(
            "INSERT INTO practice_sets \
                 (id, student_id, subject_id, topic, practice_type, total_questions, practice, \
                  created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8) \
             RETURNING id, subject_id, topic, practice_type, total_questions, status, practice, \
                       score_correct, submitted_at, created_at",
        )
        .bind(Uuid::now_v7())
        .bind(student_id)
        .bind(set.subject_id)
        .bind(set.topic)
        .bind(practice_type_name(set.practice_type))
        .bind(into_small("total_questions", set.total_questions)?)
        .bind(Json(set.practice))
        .bind(set.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err)?;
        row.try_into()
    }

    async fn get_set(
        &self,
        student_id: Uuid,
        set_id: Uuid,
    ) -> Result<Option<StoredPracticeSet>, StoreError> {
        let row = sqlx::query_as::<Postgres, SetRow>(
            "SELECT id, subject_id, topic, practice_type, total_questions, status, practice, \
                    score_correct, submitted_at, created_at \
             FROM practice_sets WHERE id = $1 AND student_id = $2",
        )
        .bind(set_id)
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?;
        row.map(TryInto::try_into).transpose()
    }

    async fn list_sets(
        &self,
        student_id: Uuid,
        limit: u32,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<Vec<PracticeSetSummary>, StoreError> {
        let rows = sqlx::query_as::<Postgres, SummaryRow>(
            "SELECT id, subject_id, topic, practice_type, total_questions, status, \
                    score_correct, created_at \
             FROM practice_sets \
             WHERE student_id = $1 \
               AND ($2::timestamptz IS NULL OR (created_at, id) < ($2::timestamptz, $3::uuid)) \
             ORDER BY created_at DESC, id DESC \
             LIMIT $4",
        )
        .bind(student_id)
        .bind(after.map(|(created_at, _)| created_at))
        .bind(after.map(|(_, id)| id))
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;
        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn answers(
        &self,
        student_id: Uuid,
        set_id: Uuid,
    ) -> Result<Vec<StoredPracticeAnswer>, StoreError> {
        let rows = sqlx::query_as::<Postgres, AnswerRow>(
            "SELECT question_id, selected, is_correct FROM practice_answers \
             WHERE practice_set_id = $1 AND student_id = $2",
        )
        .bind(set_id)
        .bind(student_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(rows
            .into_iter()
            .map(|row| StoredPracticeAnswer {
                question_id: row.question_id,
                selected: row.selected.0,
                is_correct: row.is_correct,
            })
            .collect())
    }

    async fn submit_set(&self, record: &SubmitRecord) -> Result<(), StoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        // Lock the row first, so "already submitted" is decided inside the transaction.
        let status: Option<String> = sqlx::query_scalar(
            "SELECT status FROM practice_sets WHERE id = $1 AND student_id = $2 FOR UPDATE",
        )
        .bind(record.practice_set_id)
        .bind(record.student_id)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(map_db_err)?;
        match status.as_deref() {
            None => return Err(StoreError::NotFound),
            Some("ready") => {}
            Some(_) => return Err(StoreError::Conflict),
        }

        let score_correct = into_small(
            "score_correct",
            u16::try_from(record.correct)
                .map_err(|_| internal("the correct count does not fit the column"))?,
        )?;
        let updated = sqlx::query(
            "UPDATE practice_sets \
             SET status = 'submitted', score_correct = $3, submitted_at = $4, updated_at = $4 \
             WHERE id = $1 AND student_id = $2 AND status = 'ready'",
        )
        .bind(record.practice_set_id)
        .bind(record.student_id)
        .bind(score_correct)
        .bind(record.submitted_at)
        .execute(tx.as_mut())
        .await
        .map_err(map_db_err)?;
        if updated.rows_affected() == 0 {
            // The row changed between the lock and the update; only a submission could do that.
            return Err(StoreError::Conflict);
        }

        for answer in &record.answers {
            sqlx::query(
                "INSERT INTO practice_answers \
                     (id, practice_set_id, student_id, question_id, selected, is_correct, \
                      created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
            )
            .bind(Uuid::now_v7())
            .bind(record.practice_set_id)
            .bind(record.student_id)
            .bind(&answer.question_id)
            .bind(Json(&answer.selected))
            .bind(answer.is_correct)
            .bind(record.submitted_at)
            .execute(tx.as_mut())
            .await
            .map_err(map_db_err)?;
        }

        sqlx::query(
            "INSERT INTO topic_mastery \
                 (id, student_id, subject_id, topic, attempts, correct, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $7) \
             ON CONFLICT (student_id, subject_id, topic) DO UPDATE SET \
                 attempts = topic_mastery.attempts + EXCLUDED.attempts, \
                 correct = topic_mastery.correct + EXCLUDED.correct, \
                 updated_at = EXCLUDED.updated_at",
        )
        .bind(Uuid::now_v7())
        .bind(record.student_id)
        .bind(&record.subject_id)
        .bind(&record.topic)
        .bind(record.attempts as i32)
        .bind(record.correct as i32)
        .bind(record.submitted_at)
        .execute(tx.as_mut())
        .await
        .map_err(map_db_err)?;

        // Stars are entries, never a balance: a row only when something was earned.
        if record.stars_earned > 0 {
            sqlx::query(
                "INSERT INTO star_ledger \
                     (id, student_id, amount, reason, ref_id, created_at, updated_at) \
                 VALUES ($1, $2, $3, 'practice', $4, $5, $5)",
            )
            .bind(Uuid::now_v7())
            .bind(record.student_id)
            .bind(record.stars_earned as i32)
            .bind(record.practice_set_id.to_string())
            .bind(record.submitted_at)
            .execute(tx.as_mut())
            .await
            .map_err(map_db_err)?;
        }

        tx.commit()
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))
    }

    async fn list_topic_mastery(
        &self,
        student_id: Uuid,
        subject_id: Option<&str>,
    ) -> Result<Vec<TopicMasteryEntry>, StoreError> {
        let rows = sqlx::query_as::<Postgres, MasteryRow>(
            "SELECT subject_id, topic, attempts, correct FROM topic_mastery \
             WHERE student_id = $1 AND ($2::text IS NULL OR subject_id = $2) \
             ORDER BY subject_id, topic",
        )
        .bind(student_id)
        .bind(subject_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;
        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
    }
}
