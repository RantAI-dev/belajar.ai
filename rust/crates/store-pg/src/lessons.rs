//! `LessonStore` on Postgres. A lesson and its learn conversation are written together, in one
//! transaction; completing a lesson writes its star ledger rows in the same transaction as the
//! progress update, so stars are never awarded for a result that was not recorded.

use async_trait::async_trait;
use belajar_ports::lessons::{Lesson, LessonStep, LessonStore};
use belajar_ports::store::StoreError;
use chrono::{DateTime, Utc};
use sqlx::Postgres;
use uuid::Uuid;

use crate::Db;

/// Stars per newly reached level, the amount of a `star_ledger` row for a lesson.
const STARS_PER_LEVEL: u32 = 5;

/// One lesson row, joined with its conversation for the chapter title. An inner join is exact:
/// a lesson always has its learn conversation, and nothing in the MVP deletes conversations.
#[derive(Debug, sqlx::FromRow)]
struct LessonRow {
    id: Uuid,
    subject_id: String,
    chapter_id: String,
    conversation_id: Option<Uuid>,
    step: String,
    stars: i16,
    completed_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
    chapter_title: String,
}

impl TryFrom<LessonRow> for Lesson {
    type Error = StoreError;

    fn try_from(row: LessonRow) -> Result<Self, Self::Error> {
        // The CHECK constraints bound step and stars; this only keeps a drifted database from
        // turning into a panic or a silently wrong response.
        let step = LessonStep::parse(&row.step).ok_or_else(|| {
            StoreError::Internal(format!("lesson_progress.step is unknown: {}", row.step))
        })?;
        let stars = u8::try_from(row.stars).map_err(|_| {
            StoreError::Internal("lesson_progress.stars is outside 0..=3".to_owned())
        })?;
        let conversation_id = row.conversation_id.ok_or_else(|| {
            StoreError::Internal("lesson_progress.conversation_id is missing".to_owned())
        })?;
        Ok(Lesson {
            id: row.id,
            subject_id: row.subject_id,
            chapter_id: row.chapter_id,
            chapter_title: row.chapter_title,
            conversation_id,
            step,
            stars,
            completed_at: row.completed_at,
            updated_at: row.updated_at,
        })
    }
}

/// The SELECT every read shares: the lesson joined with its conversation's title.
const LESSON_FROM: &str =
    "FROM lesson_progress lp JOIN conversations c ON c.id = lp.conversation_id";
const LESSON_COLUMNS: &str = "lp.id, lp.subject_id, lp.chapter_id, lp.conversation_id, \
     lp.step, lp.stars, lp.completed_at, lp.updated_at, c.title AS chapter_title";

fn lesson(row: LessonRow) -> Result<Lesson, StoreError> {
    row.try_into()
}

/// One place that turns a driver error into a `StoreError`. A concurrent `start` for the same
/// (student, subject, chapter) is the only conflict a caller can cause here; the ledger and
/// mission inserts all use `ON CONFLICT DO NOTHING` and never reach this mapping.
fn map_db_err(err: sqlx::Error) -> StoreError {
    let raced_lesson = err
        .as_database_error()
        .is_some_and(|e| e.is_unique_violation())
        && err
            .as_database_error()
            .and_then(|e| e.constraint())
            .is_some_and(|c| c == "uq_lesson_progress_student_id_subject_id_chapter_id");
    if raced_lesson {
        StoreError::Conflict
    } else {
        StoreError::Internal(err.to_string())
    }
}

fn internal(err: sqlx::Error) -> StoreError {
    StoreError::Internal(err.to_string())
}

#[async_trait]
impl LessonStore for Db {
    async fn get(&self, student_id: Uuid, lesson_id: Uuid) -> Result<Lesson, StoreError> {
        let row = sqlx::query_as::<Postgres, LessonRow>(&format!(
            "SELECT {LESSON_COLUMNS} {LESSON_FROM} WHERE lp.student_id = $1 AND lp.id = $2"
        ))
        .bind(student_id)
        .bind(lesson_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        lesson(row.ok_or(StoreError::NotFound)?)
    }

    async fn list_by_subject(
        &self,
        student_id: Uuid,
        subject_id: &str,
    ) -> Result<Vec<Lesson>, StoreError> {
        let rows = sqlx::query_as::<Postgres, LessonRow>(&format!(
            "SELECT {LESSON_COLUMNS} {LESSON_FROM} \
             WHERE lp.student_id = $1 AND lp.subject_id = $2"
        ))
        .bind(student_id)
        .bind(subject_id)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        rows.into_iter().map(lesson).collect()
    }

    async fn start(
        &self,
        student_id: Uuid,
        subject_id: &str,
        chapter_id: &str,
        chapter_title: &str,
        now: DateTime<Utc>,
    ) -> Result<(Lesson, bool), StoreError> {
        let mut tx = self.pool.begin().await.map_err(internal)?;

        let existing = sqlx::query_as::<Postgres, LessonRow>(&format!(
            "SELECT {LESSON_COLUMNS} {LESSON_FROM} \
             WHERE lp.student_id = $1 AND lp.subject_id = $2 AND lp.chapter_id = $3"
        ))
        .bind(student_id)
        .bind(subject_id)
        .bind(chapter_id)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(internal)?;
        if let Some(row) = existing {
            // Nothing to write: commit releases the transaction without changing a row.
            tx.commit().await.map_err(internal)?;
            return Ok((lesson(row)?, false));
        }

        // The conversation and its lesson row are created together, so one never exists alone.
        let conversation_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO conversations \
             (id, student_id, mode, subject_id, chapter_id, title, created_at, updated_at) \
             VALUES ($1, $2, 'learn', $3, $4, $5, $6, $6)",
        )
        .bind(conversation_id)
        .bind(student_id)
        .bind(subject_id)
        .bind(chapter_id)
        .bind(chapter_title)
        .bind(now)
        .execute(tx.as_mut())
        .await
        .map_err(map_db_err)?;

        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO lesson_progress \
             (id, student_id, subject_id, chapter_id, conversation_id, step, stars, \
              created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, 'explain', 0, $6, $6)",
        )
        .bind(id)
        .bind(student_id)
        .bind(subject_id)
        .bind(chapter_id)
        .bind(conversation_id)
        .bind(now)
        .execute(tx.as_mut())
        .await
        .map_err(map_db_err)?;

        tx.commit().await.map_err(internal)?;
        Ok((
            Lesson {
                id,
                subject_id: subject_id.to_owned(),
                chapter_id: chapter_id.to_owned(),
                chapter_title: chapter_title.to_owned(),
                conversation_id,
                step: LessonStep::Explain,
                stars: 0,
                completed_at: None,
                updated_at: now,
            },
            true,
        ))
    }

    async fn set_step(
        &self,
        student_id: Uuid,
        lesson_id: Uuid,
        step: LessonStep,
        now: DateTime<Utc>,
    ) -> Result<Lesson, StoreError> {
        let row = sqlx::query_as::<Postgres, LessonRow>(&format!(
            "UPDATE lesson_progress lp SET step = $3, updated_at = $4 \
             FROM conversations c WHERE c.id = lp.conversation_id \
             AND lp.student_id = $1 AND lp.id = $2 \
             RETURNING {LESSON_COLUMNS}"
        ))
        .bind(student_id)
        .bind(lesson_id)
        .bind(step.as_str())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        lesson(row.ok_or(StoreError::NotFound)?)
    }

    async fn complete(
        &self,
        student_id: Uuid,
        lesson_id: Uuid,
        quiz_stars: u8,
        now: DateTime<Utc>,
    ) -> Result<(Lesson, u32), StoreError> {
        let mut tx = self.pool.begin().await.map_err(internal)?;

        // Locked for the rest of the transaction so two completions cannot both award the same
        // newly reached level.
        let row = sqlx::query_as::<Postgres, LessonRow>(&format!(
            "SELECT {LESSON_COLUMNS} {LESSON_FROM} \
             WHERE lp.student_id = $1 AND lp.id = $2 FOR UPDATE OF lp"
        ))
        .bind(student_id)
        .bind(lesson_id)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(internal)?;
        let row = row.ok_or(StoreError::NotFound)?;

        let old_stars = u8::try_from(row.stars).map_err(|_| {
            StoreError::Internal("lesson_progress.stars is outside 0..=3".to_owned())
        })?;
        let new_stars = old_stars.max(quiz_stars);

        let completed_at = row.completed_at.unwrap_or(now);
        let updated = sqlx::query_as::<Postgres, LessonRow>(&format!(
            "UPDATE lesson_progress lp \
             SET stars = $3, step = 'done', completed_at = $4, updated_at = $5 \
             FROM conversations c WHERE c.id = lp.conversation_id \
             AND lp.student_id = $1 AND lp.id = $2 \
             RETURNING {LESSON_COLUMNS}"
        ))
        .bind(student_id)
        .bind(lesson_id)
        .bind(i16::from(new_stars))
        .bind(completed_at)
        .bind(now)
        .fetch_one(tx.as_mut())
        .await
        .map_err(internal)?;

        // A row for every star level newly reached; a lower or equal retake writes none. The
        // unique index makes a retried award a no-op instead of an error.
        let mut stars_earned = 0_u32;
        for level in old_stars.saturating_add(1)..=new_stars {
            let result = sqlx::query(
                "INSERT INTO star_ledger \
                 (id, student_id, amount, reason, ref_id, created_at, updated_at) \
                 VALUES ($1, $2, $3, 'lesson', $4, $5, $5) ON CONFLICT DO NOTHING",
            )
            .bind(Uuid::now_v7())
            .bind(student_id)
            .bind(i32::try_from(STARS_PER_LEVEL).unwrap_or(0))
            .bind(format!("{}/{}/{}", row.subject_id, row.chapter_id, level))
            .bind(now)
            .execute(tx.as_mut())
            .await
            .map_err(internal)?;
            stars_earned += STARS_PER_LEVEL * u32::try_from(result.rows_affected()).unwrap_or(0);
        }

        tx.commit().await.map_err(internal)?;
        Ok((lesson(updated)?, stars_earned))
    }
}
