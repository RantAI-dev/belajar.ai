//! `HomeStore` on Postgres. Every activity day is the Jakarta calendar date of a stored
//! timestamp (`AT TIME ZONE 'Asia/Jakarta'`: fixed UTC+07:00, no DST, the same rule
//! `belajar_core::calendar` applies to the clock); today comes from the caller, from the same
//! rule. Mission rewards are inserted with `ON CONFLICT DO NOTHING` under the unique indexes, so
//! the same home request twice awards them once.

use async_trait::async_trait;
use belajar_core::streak::streak_days;
use belajar_ports::home::{ContinueItem, HomeData, HomeStore, MissionCounts, MissionKind};
use belajar_ports::lessons::LessonStep;
use belajar_ports::store::StoreError;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::Postgres;
use uuid::Uuid;

use crate::Db;
use crate::activity::active_days;

/// Stars for completing any daily mission. `smallint` in `daily_missions`, `integer` in the
/// ledger, so each bind converts to its column's exact width.
const MISSION_STARS: i16 = 10;

fn internal(err: sqlx::Error) -> StoreError {
    StoreError::Internal(err.to_string())
}

fn count(row: i64) -> Result<u32, StoreError> {
    u32::try_from(row).map_err(|_| StoreError::Internal("a COUNT(*) came back negative".to_owned()))
}

/// A lesson row for the continue-learning list: the lesson joined with its conversation's title.
#[derive(Debug, sqlx::FromRow)]
struct ContinueRow {
    id: Uuid,
    subject_id: String,
    chapter_id: String,
    step: String,
    stars: i16,
    updated_at: DateTime<Utc>,
    chapter_title: String,
}

impl TryFrom<ContinueRow> for ContinueItem {
    type Error = StoreError;

    fn try_from(row: ContinueRow) -> Result<Self, Self::Error> {
        let step = LessonStep::parse(&row.step).ok_or_else(|| {
            StoreError::Internal(format!("lesson_progress.step is unknown: {}", row.step))
        })?;
        let stars = u8::try_from(row.stars).map_err(|_| {
            StoreError::Internal("lesson_progress.stars is outside 0..=3".to_owned())
        })?;
        Ok(ContinueItem {
            lesson_id: row.id,
            subject_id: row.subject_id,
            chapter_id: row.chapter_id,
            chapter_title: row.chapter_title,
            step,
            stars,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl HomeStore for Db {
    async fn home(
        &self,
        student_id: Uuid,
        today: NaiveDate,
        now: DateTime<Utc>,
    ) -> Result<HomeData, StoreError> {
        let mut tx = self.pool.begin().await.map_err(internal)?;

        let ask = count(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM messages m \
                 JOIN conversations c ON c.id = m.conversation_id \
                 WHERE m.student_id = $1 AND m.role = 'user' AND c.mode = 'ask' \
                 AND (m.created_at AT TIME ZONE 'Asia/Jakarta')::date = $2",
            )
            .bind(student_id)
            .bind(today)
            .fetch_one(tx.as_mut())
            .await
            .map_err(internal)?,
        )?;
        let lesson_done = count(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM lesson_progress \
                 WHERE student_id = $1 AND step = 'done' \
                 AND (updated_at AT TIME ZONE 'Asia/Jakarta')::date = $2",
            )
            .bind(student_id)
            .bind(today)
            .fetch_one(tx.as_mut())
            .await
            .map_err(internal)?,
        )?;
        let practice_done = count(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM practice_sets \
                 WHERE student_id = $1 AND submitted_at IS NOT NULL \
                 AND (submitted_at AT TIME ZONE 'Asia/Jakarta')::date = $2",
            )
            .bind(student_id)
            .bind(today)
            .fetch_one(tx.as_mut())
            .await
            .map_err(internal)?,
        )?;

        for (kind, done) in [
            (MissionKind::Ask, ask),
            (MissionKind::Lesson, lesson_done),
            (MissionKind::Practice, practice_done),
        ] {
            if done < 1 {
                continue;
            }
            // The mission row is the "reward given" marker: it is inserted once, and the ledger
            // row only alongside it, both no-ops when today's reward already exists.
            let inserted: Option<Uuid> = sqlx::query_scalar(
                "INSERT INTO daily_missions \
                 (id, student_id, on_date, kind, completed_at, stars_awarded, \
                  created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $5, $5) ON CONFLICT DO NOTHING RETURNING id",
            )
            .bind(Uuid::now_v7())
            .bind(student_id)
            .bind(today)
            .bind(kind.as_str())
            .bind(now)
            .bind(MISSION_STARS)
            .fetch_optional(tx.as_mut())
            .await
            .map_err(internal)?;
            if inserted.is_some() {
                sqlx::query(
                    "INSERT INTO star_ledger \
                     (id, student_id, amount, reason, ref_id, created_at, updated_at) \
                     VALUES ($1, $2, $3, 'mission', $4, $5, $5) ON CONFLICT DO NOTHING",
                )
                .bind(Uuid::now_v7())
                .bind(student_id)
                .bind(i32::from(MISSION_STARS))
                .bind(format!("{today}/{}", kind.as_str()))
                .bind(now)
                .execute(tx.as_mut())
                .await
                .map_err(internal)?;
            }
        }

        let stars = count(
            sqlx::query_scalar::<_, i64>(
                "SELECT COALESCE(SUM(amount), 0) FROM star_ledger WHERE student_id = $1",
            )
            .bind(student_id)
            .fetch_one(tx.as_mut())
            .await
            .map_err(internal)?,
        )?;

        // One date per day the student was active, for the streak.
        let active = active_days(tx.as_mut(), student_id).await?;

        let unfinished = sqlx::query_as::<Postgres, ContinueRow>(
            "SELECT lp.id, lp.subject_id, lp.chapter_id, lp.step, lp.stars, lp.updated_at, \
             c.title AS chapter_title \
             FROM lesson_progress lp JOIN conversations c ON c.id = lp.conversation_id \
             WHERE lp.student_id = $1 AND lp.step <> 'done' \
             ORDER BY lp.updated_at DESC, lp.id DESC LIMIT 3",
        )
        .bind(student_id)
        .fetch_all(tx.as_mut())
        .await
        .map_err(internal)?;

        tx.commit().await.map_err(internal)?;

        Ok(HomeData {
            stars,
            streak_days: streak_days(&active, today),
            mission_counts: MissionCounts {
                ask,
                lesson: lesson_done,
                practice: practice_done,
            },
            continue_learning: unfinished
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
