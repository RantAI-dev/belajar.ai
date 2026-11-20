//! `ProgressStore` on Postgres: read-only counts over the activity tables, in one repeatable-read
//! transaction so every number on the page comes from the same moment. Every query filters by
//! `student_id`; a join also requires the joined row to be the same student's.

use async_trait::async_trait;
use belajar_ports::lessons::LessonStep;
use belajar_ports::progress::{
    ChapterProgress, DayActivity, ProgressData, ProgressStore, ProgressWindows, RECENT_SESSIONS,
    RecentSession, SessionKind, Tally, TopicProgress,
};
use belajar_ports::store::StoreError;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::Postgres;
use uuid::Uuid;

use crate::Db;
use crate::activity::active_days;

fn internal(err: sqlx::Error) -> StoreError {
    StoreError::Internal(err.to_string())
}

/// A COUNT or SUM as the u32 the domain uses; a negative or huge one means a drifted database.
fn count(column: &str, value: i64) -> Result<u32, StoreError> {
    u32::try_from(value).map_err(|_| StoreError::Internal(format!("{column} is out of range")))
}

fn tally(correct: i64, attempts: i64) -> Result<Tally, StoreError> {
    Ok(Tally {
        correct: count("correct", correct)?,
        attempts: count("attempts", attempts)?,
    })
}

#[derive(Debug, sqlx::FromRow)]
struct TotalsRow {
    stars_total: i64,
    stars_week: i64,
    answers_total: i64,
    correct_total: i64,
    answers_week: i64,
    correct_week: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct TopicRow {
    subject_id: String,
    topic: String,
    attempts: i32,
    correct: i32,
    this_attempts: i64,
    this_correct: i64,
    last_attempts: i64,
    last_correct: i64,
    last_practiced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
struct ChapterRow {
    id: Uuid,
    subject_id: String,
    chapter_id: String,
    chapter_title: String,
    step: String,
    stars: i16,
    completed_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct DayRow {
    d: NaiveDate,
    answers: i64,
    correct: i64,
    lessons: i64,
    asked: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct PracticeSessionRow {
    id: Uuid,
    subject_id: String,
    topic: String,
    score_correct: Option<i16>,
    total_questions: i16,
    submitted_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct AskSessionRow {
    id: Uuid,
    subject_id: Option<String>,
    title: String,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TopicRow> for TopicProgress {
    type Error = StoreError;

    fn try_from(row: TopicRow) -> Result<Self, Self::Error> {
        Ok(Self {
            subject_id: row.subject_id,
            topic: row.topic,
            total: tally(i64::from(row.correct), i64::from(row.attempts))?,
            this_week: tally(row.this_correct, row.this_attempts)?,
            last_week: tally(row.last_correct, row.last_attempts)?,
            last_practiced_at: row.last_practiced_at,
        })
    }
}

impl TryFrom<ChapterRow> for ChapterProgress {
    type Error = StoreError;

    fn try_from(row: ChapterRow) -> Result<Self, Self::Error> {
        let step = LessonStep::parse(&row.step).ok_or_else(|| {
            StoreError::Internal(format!("lesson_progress.step is unknown: {}", row.step))
        })?;
        let stars = u8::try_from(row.stars).map_err(|_| {
            StoreError::Internal("lesson_progress.stars is outside 0..=3".to_owned())
        })?;
        Ok(Self {
            lesson_id: row.id,
            subject_id: row.subject_id,
            chapter_id: row.chapter_id,
            chapter_title: row.chapter_title,
            step,
            stars,
            completed_at: row.completed_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<DayRow> for DayActivity {
    type Error = StoreError;

    fn try_from(row: DayRow) -> Result<Self, Self::Error> {
        Ok(Self {
            on: row.d,
            answers: tally(row.correct, row.answers)?,
            lessons_completed: count("lessons", row.lessons)?,
            questions_asked: count("asked", row.asked)?,
        })
    }
}

fn practice_session(row: PracticeSessionRow) -> Result<RecentSession, StoreError> {
    let total = u32::try_from(row.total_questions)
        .map_err(|_| StoreError::Internal("practice_sets.total_questions is negative".into()))?;
    let correct = u32::try_from(row.score_correct.unwrap_or(0))
        .map_err(|_| StoreError::Internal("practice_sets.score_correct is negative".into()))?;
    Ok(RecentSession {
        kind: SessionKind::Practice,
        id: row.id,
        subject_id: Some(row.subject_id),
        title: row.topic,
        score: Some(Tally {
            correct,
            attempts: total,
        }),
        stars: None,
        occurred_at: row.submitted_at,
    })
}

fn lesson_session(chapter: &ChapterProgress) -> RecentSession {
    RecentSession {
        kind: SessionKind::Lesson,
        id: chapter.lesson_id,
        subject_id: Some(chapter.subject_id.clone()),
        title: chapter.chapter_title.clone(),
        score: None,
        stars: Some(chapter.stars),
        occurred_at: chapter.updated_at,
    }
}

#[async_trait]
impl ProgressStore for Db {
    async fn progress(
        &self,
        student_id: Uuid,
        windows: &ProgressWindows,
    ) -> Result<ProgressData, StoreError> {
        let mut tx = self.pool.begin().await.map_err(internal)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(tx.as_mut())
            .await
            .map_err(internal)?;
        let week = windows.this_week;

        let totals = sqlx::query_as::<Postgres, TotalsRow>(
            "SELECT \
                 (SELECT COALESCE(SUM(amount), 0)::bigint FROM star_ledger \
                  WHERE student_id = $1) AS stars_total, \
                 (SELECT COALESCE(SUM(amount), 0)::bigint FROM star_ledger \
                  WHERE student_id = $1 AND created_at >= $2 AND created_at < $3) AS stars_week, \
                 COUNT(*) AS answers_total, \
                 COUNT(*) FILTER (WHERE is_correct) AS correct_total, \
                 COUNT(*) FILTER (WHERE created_at >= $2 AND created_at < $3) AS answers_week, \
                 COUNT(*) FILTER (WHERE is_correct AND created_at >= $2 AND created_at < $3) \
                     AS correct_week \
             FROM practice_answers WHERE student_id = $1",
        )
        .bind(student_id)
        .bind(week.start)
        .bind(week.end)
        .fetch_one(tx.as_mut())
        .await
        .map_err(internal)?;

        // All-time numbers come from `topic_mastery`, the same row Latihan shows; the weekly
        // ones from the answers of sets submitted in each window.
        let topics = sqlx::query_as::<Postgres, TopicRow>(
            "SELECT m.subject_id, m.topic, m.attempts, m.correct, \
                    COALESCE(w.this_attempts, 0) AS this_attempts, \
                    COALESCE(w.this_correct, 0) AS this_correct, \
                    COALESCE(w.last_attempts, 0) AS last_attempts, \
                    COALESCE(w.last_correct, 0) AS last_correct, \
                    w.last_practiced_at \
             FROM topic_mastery m \
             LEFT JOIN ( \
                 SELECT s.subject_id, s.topic, \
                        COUNT(a.id) FILTER (WHERE a.created_at >= $2 AND a.created_at < $3) \
                            AS this_attempts, \
                        COUNT(a.id) FILTER (WHERE a.is_correct \
                            AND a.created_at >= $2 AND a.created_at < $3) AS this_correct, \
                        COUNT(a.id) FILTER (WHERE a.created_at >= $4 AND a.created_at < $2) \
                            AS last_attempts, \
                        COUNT(a.id) FILTER (WHERE a.is_correct \
                            AND a.created_at >= $4 AND a.created_at < $2) AS last_correct, \
                        MAX(s.submitted_at) AS last_practiced_at \
                 FROM practice_sets s \
                 LEFT JOIN practice_answers a \
                     ON a.practice_set_id = s.id AND a.student_id = $1 \
                 WHERE s.student_id = $1 AND s.submitted_at IS NOT NULL \
                 GROUP BY s.subject_id, s.topic \
             ) w ON w.subject_id = m.subject_id AND w.topic = m.topic \
             WHERE m.student_id = $1 \
             ORDER BY m.subject_id, m.topic",
        )
        .bind(student_id)
        .bind(week.start)
        .bind(week.end)
        .bind(windows.last_week.start)
        .fetch_all(tx.as_mut())
        .await
        .map_err(internal)?;

        // The conversation is joined for its title only, and only when it is the same student's.
        let chapters = sqlx::query_as::<Postgres, ChapterRow>(
            "SELECT lp.id, lp.subject_id, lp.chapter_id, \
                    COALESCE(c.title, lp.chapter_id) AS chapter_title, \
                    lp.step, lp.stars, lp.completed_at, lp.updated_at \
             FROM lesson_progress lp \
             LEFT JOIN conversations c ON c.id = lp.conversation_id AND c.student_id = $1 \
             WHERE lp.student_id = $1 \
             ORDER BY lp.updated_at DESC, lp.id DESC",
        )
        .bind(student_id)
        .fetch_all(tx.as_mut())
        .await
        .map_err(internal)?;

        let days = sqlx::query_as::<Postgres, DayRow>(
            "SELECT d, SUM(answers)::bigint AS answers, SUM(correct)::bigint AS correct, \
                    SUM(lessons)::bigint AS lessons, SUM(asked)::bigint AS asked \
             FROM ( \
                 SELECT (created_at AT TIME ZONE 'Asia/Jakarta')::date AS d, 1 AS answers, \
                        CASE WHEN is_correct THEN 1 ELSE 0 END AS correct, 0 AS lessons, \
                        0 AS asked \
                 FROM practice_answers \
                 WHERE student_id = $1 AND created_at >= $2 AND created_at < $3 \
                 UNION ALL \
                 SELECT (completed_at AT TIME ZONE 'Asia/Jakarta')::date, 0, 0, 1, 0 \
                 FROM lesson_progress \
                 WHERE student_id = $1 AND completed_at >= $2 AND completed_at < $3 \
                 UNION ALL \
                 SELECT (created_at AT TIME ZONE 'Asia/Jakarta')::date, 0, 0, 0, 1 \
                 FROM messages \
                 WHERE student_id = $1 AND role = 'user' AND created_at >= $2 AND created_at < $3 \
             ) activity \
             GROUP BY d ORDER BY d",
        )
        .bind(student_id)
        .bind(windows.activity.start)
        .bind(windows.activity.end)
        .fetch_all(tx.as_mut())
        .await
        .map_err(internal)?;

        let limit = i64::try_from(RECENT_SESSIONS).unwrap_or(8);
        let practice_sessions = sqlx::query_as::<Postgres, PracticeSessionRow>(
            "SELECT id, subject_id, topic, score_correct, total_questions, submitted_at \
             FROM practice_sets \
             WHERE student_id = $1 AND submitted_at IS NOT NULL \
             ORDER BY submitted_at DESC, id DESC LIMIT $2",
        )
        .bind(student_id)
        .bind(limit)
        .fetch_all(tx.as_mut())
        .await
        .map_err(internal)?;
        let ask_sessions = sqlx::query_as::<Postgres, AskSessionRow>(
            "SELECT id, subject_id, title, updated_at FROM conversations \
             WHERE student_id = $1 AND mode = 'ask' \
             ORDER BY updated_at DESC, id DESC LIMIT $2",
        )
        .bind(student_id)
        .bind(limit)
        .fetch_all(tx.as_mut())
        .await
        .map_err(internal)?;

        let active = active_days(tx.as_mut(), student_id).await?;
        tx.commit().await.map_err(internal)?;

        let chapters = chapters
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<ChapterProgress>, _>>()?;
        let mut recent = practice_sessions
            .into_iter()
            .map(practice_session)
            .collect::<Result<Vec<_>, _>>()?;
        recent.extend(chapters.iter().take(RECENT_SESSIONS).map(lesson_session));
        recent.extend(ask_sessions.into_iter().map(|row| RecentSession {
            kind: SessionKind::Ask,
            id: row.id,
            subject_id: row.subject_id,
            title: row.title,
            score: None,
            stars: None,
            occurred_at: row.updated_at,
        }));
        recent.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at).then(b.id.cmp(&a.id)));
        recent.truncate(RECENT_SESSIONS);

        Ok(ProgressData {
            stars_total: count("stars_total", totals.stars_total)?,
            stars_this_week: count("stars_week", totals.stars_week)?,
            answers_total: tally(totals.correct_total, totals.answers_total)?,
            answers_this_week: tally(totals.correct_week, totals.answers_week)?,
            active_days: active,
            topics: topics
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            chapters,
            days: days
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            recent,
        })
    }
}
