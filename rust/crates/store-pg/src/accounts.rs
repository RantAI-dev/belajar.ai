//! `AccountStore` on Postgres. The user and their profile are written together, in one
//! transaction, so a student never exists without a grade and a nickname.

use async_trait::async_trait;
use belajar_core::grade::{Grade, GradeError};
use belajar_core::theme::ThemePreference;
use belajar_ports::accounts::{AccountStore, LoginRecord, SessionRecord, StudentAccount};
use belajar_ports::store::StoreError;
use chrono::{DateTime, Utc};
use sqlx::{Postgres, Row};
use uuid::Uuid;

use crate::Db;

/// Every query's row for a whole student, joined from `users` and `student_profiles`.
#[derive(Debug, sqlx::FromRow)]
struct StudentRow {
    id: Uuid,
    username: String,
    created_at: DateTime<Utc>,
    nickname: String,
    grade: i16,
    theme: String,
}

impl TryFrom<StudentRow> for StudentAccount {
    type Error = StoreError;

    fn try_from(row: StudentRow) -> Result<Self, Self::Error> {
        // The column's CHECK already bounds it; this only keeps a drifted database from panicking.
        let grade = u8::try_from(row.grade)
            .map_err(|_| out_of_range_grade())
            .and_then(|g| Grade::new(g).map_err(|GradeError::OutOfRange| out_of_range_grade()))?;
        let theme = ThemePreference::parse(&row.theme).ok_or_else(|| {
            StoreError::Internal("student_profiles.theme is not system, light or dark".to_owned())
        })?;
        Ok(StudentAccount {
            id: row.id,
            username: row.username,
            nickname: row.nickname,
            grade,
            theme,
            created_at: row.created_at,
        })
    }
}

fn out_of_range_grade() -> StoreError {
    StoreError::Internal("student_profiles.grade is outside 1..=12".to_owned())
}

/// One place that turns a driver error into a `StoreError`; the text is for the log only.
fn map_db_err(err: sqlx::Error) -> StoreError {
    let db_err = err.as_database_error();
    // Two taken-username shapes: the index on lower(username), or the profile of an id that
    // already exists. Both mean "this username is already a student".
    let username_conflict = db_err
        .and_then(|e| e.constraint())
        .is_some_and(|c| c == "uq_users_username" || c == "student_profiles_pkey")
        && db_err.is_some_and(|e| e.is_unique_violation());
    if username_conflict {
        StoreError::Conflict
    } else {
        StoreError::Internal(err.to_string())
    }
}

/// A missing row is 404; everything the caller may have caused is spelled out above.
fn map_option<T>(row: Option<T>) -> Result<T, StoreError> {
    row.ok_or(StoreError::NotFound)
}

#[async_trait]
impl AccountStore for Db {
    async fn create_student(
        &self,
        username: &str,
        password_hash: &str,
        nickname: &str,
        grade: Grade,
    ) -> Result<StudentAccount, StoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        let id = Uuid::now_v7();
        let created_at: DateTime<Utc> = sqlx::query_scalar(
            "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3) \
             RETURNING created_at",
        )
        .bind(id)
        .bind(username)
        .bind(password_hash)
        .fetch_one(tx.as_mut())
        .await
        .map_err(map_db_err)?;

        sqlx::query(
            "INSERT INTO student_profiles (student_id, grade, nickname) VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(i16::from(grade.get()))
        .bind(nickname)
        .execute(tx.as_mut())
        .await
        .map_err(map_db_err)?;

        tx.commit()
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(StudentAccount {
            id,
            username: username.to_owned(),
            nickname: nickname.to_owned(),
            grade,
            theme: ThemePreference::default(),
            created_at,
        })
    }

    async fn find_login_by_username(
        &self,
        username: &str,
    ) -> Result<Option<LoginRecord>, StoreError> {
        // A Google-only student has no password, so a username login never finds them.
        let row = sqlx::query(
            "SELECT id, password_hash FROM users WHERE username = $1 AND password_hash IS NOT NULL",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?;
        let record = row.map(|r| LoginRecord {
            user_id: r.get::<Uuid, _>("id"),
            password_hash: r.get::<String, _>("password_hash"),
        });
        Ok(record)
    }

    async fn password_hash(&self, student_id: Uuid) -> Result<String, StoreError> {
        let hash = sqlx::query_scalar::<_, Option<String>>(
            "SELECT password_hash FROM users WHERE id = $1",
        )
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?;
        // A Google-only student has no password: the same answer as no row.
        map_option(hash.flatten())
    }

    async fn set_password_hash(
        &self,
        student_id: Uuid,
        password_hash: &str,
    ) -> Result<(), StoreError> {
        sqlx::query("UPDATE users SET password_hash = $2, updated_at = now() WHERE id = $1")
            .bind(student_id)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }

    async fn create_session(
        &self,
        student_id: Uuid,
        token_hash: &[u8],
        expires_at: DateTime<Utc>,
    ) -> Result<Uuid, StoreError> {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO sessions (id, user_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(student_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(id)
    }

    async fn find_session(
        &self,
        token_hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<SessionRecord>, StoreError> {
        let row = sqlx::query(
            "SELECT id, user_id FROM sessions WHERE token_hash = $1 \
             AND expires_at > $2",
        )
        .bind(token_hash)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?;
        let record = row.map(|r| SessionRecord {
            session_id: r.get::<Uuid, _>("id"),
            student_id: r.get::<Uuid, _>("user_id"),
        });
        Ok(record)
    }

    async fn delete_session(&self, student_id: Uuid, session_id: Uuid) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id = $2")
            .bind(student_id)
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }

    async fn delete_other_sessions(&self, student_id: Uuid, keep: Uuid) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id <> $2")
            .bind(student_id)
            .bind(keep)
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }

    async fn get_student(&self, student_id: Uuid) -> Result<StudentAccount, StoreError> {
        let row = sqlx::query_as::<Postgres, StudentRow>(
            "SELECT u.id, u.username, u.created_at, p.nickname, p.grade, p.theme \
             FROM users u JOIN student_profiles p ON p.student_id = u.id WHERE u.id = $1",
        )
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?;
        map_option(row)?.try_into()
    }

    async fn update_profile(
        &self,
        student_id: Uuid,
        nickname: Option<&str>,
        grade: Option<Grade>,
        theme: Option<ThemePreference>,
    ) -> Result<StudentAccount, StoreError> {
        sqlx::query(
            "UPDATE student_profiles \
             SET nickname = COALESCE($2, nickname), grade = COALESCE($3, grade), \
                 theme = COALESCE($4, theme), updated_at = now() \
             WHERE student_id = $1",
        )
        .bind(student_id)
        .bind(nickname)
        .bind(grade.map(|g| i16::from(g.get())))
        .bind(theme.map(ThemePreference::as_str))
        .execute(&self.pool)
        .await
        .map_err(map_db_err)?;

        self.get_student(student_id).await
    }
}
