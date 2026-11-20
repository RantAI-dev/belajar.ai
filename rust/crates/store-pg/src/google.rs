//! `GoogleSignInStore` on Postgres. Attempts and pending signups are single use: the read that
//! uses one deletes it in the same statement, so two requests can never both use it.

use async_trait::async_trait;
use belajar_core::theme::ThemePreference;
use belajar_ports::accounts::StudentAccount;
use belajar_ports::google::{
    GoogleAttempt, GoogleSignInStore, NewGoogleStudent, PendingGoogleSignup,
};
use belajar_ports::store::StoreError;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::Db;

/// A unique violation on the link or the username is a conflict; anything else is for the log.
fn map_db_err(err: sqlx::Error) -> StoreError {
    let db_err = err.as_database_error();
    let conflict = db_err.is_some_and(|e| e.is_unique_violation())
        && db_err.and_then(|e| e.constraint()).is_some_and(|c| {
            c == "uq_google_identities_subject"
                || c == "uq_google_identities_student_id"
                || c == "uq_users_username"
        });
    if conflict {
        StoreError::Conflict
    } else {
        StoreError::Internal(err.to_string())
    }
}

#[async_trait]
impl GoogleSignInStore for Db {
    async fn save_attempt(
        &self,
        browser_token_hash: &[u8],
        attempt: &GoogleAttempt,
        expires_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM google_sign_in_attempts WHERE expires_at < now()")
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;
        sqlx::query(
            "INSERT INTO google_sign_in_attempts \
             (id, browser_token_hash, state, nonce, code_verifier, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(Uuid::now_v7())
        .bind(browser_token_hash)
        .bind(&attempt.state)
        .bind(&attempt.nonce)
        .bind(&attempt.code_verifier)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(())
    }

    async fn take_attempt(
        &self,
        browser_token_hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<GoogleAttempt>, StoreError> {
        let row = sqlx::query(
            "DELETE FROM google_sign_in_attempts WHERE browser_token_hash = $1 \
             RETURNING state, nonce, code_verifier, expires_at",
        )
        .bind(browser_token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?;
        let Some(row) = row else { return Ok(None) };
        let expires_at: DateTime<Utc> = row.try_get("expires_at").map_err(map_db_err)?;
        if expires_at <= now {
            return Ok(None);
        }
        Ok(Some(GoogleAttempt {
            state: row.try_get("state").map_err(map_db_err)?,
            nonce: row.try_get("nonce").map_err(map_db_err)?,
            code_verifier: row.try_get("code_verifier").map_err(map_db_err)?,
        }))
    }

    async fn find_student_by_subject(&self, subject: &str) -> Result<Option<Uuid>, StoreError> {
        sqlx::query_scalar::<_, Uuid>("SELECT student_id FROM google_identities WHERE subject = $1")
            .bind(subject)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err)
    }

    async fn save_pending_signup(
        &self,
        token_hash: &[u8],
        signup: &PendingGoogleSignup,
        expires_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM google_pending_signups WHERE expires_at < now()")
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;
        sqlx::query(
            "INSERT INTO google_pending_signups (id, token_hash, subject, display_name, expires_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::now_v7())
        .bind(token_hash)
        .bind(&signup.subject)
        .bind(&signup.display_name)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(())
    }

    async fn find_pending_signup(
        &self,
        token_hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<PendingGoogleSignup>, StoreError> {
        let row = sqlx::query(
            "SELECT subject, display_name FROM google_pending_signups \
             WHERE token_hash = $1 AND expires_at > $2",
        )
        .bind(token_hash)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?;
        row.map(|r| {
            Ok(PendingGoogleSignup {
                subject: r.try_get("subject").map_err(map_db_err)?,
                display_name: r.try_get("display_name").map_err(map_db_err)?,
            })
        })
        .transpose()
    }

    async fn complete_signup(
        &self,
        token_hash: &[u8],
        student: NewGoogleStudent<'_>,
        now: DateTime<Utc>,
    ) -> Result<StudentAccount, StoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        // Deleting first makes it single use; a failure below rolls the delete back with the rest.
        let pending = sqlx::query(
            "DELETE FROM google_pending_signups WHERE token_hash = $1 AND expires_at > $2 \
             RETURNING subject, display_name",
        )
        .bind(token_hash)
        .bind(now)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(map_db_err)?
        .ok_or(StoreError::NotFound)?;
        let subject: String = pending.try_get("subject").map_err(map_db_err)?;
        let display_name: String = pending.try_get("display_name").map_err(map_db_err)?;

        let id = Uuid::now_v7();
        let created_at: DateTime<Utc> = sqlx::query_scalar(
            "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, NULL) \
             RETURNING created_at",
        )
        .bind(id)
        .bind(student.username)
        .fetch_one(tx.as_mut())
        .await
        .map_err(map_db_err)?;

        sqlx::query(
            "INSERT INTO student_profiles (student_id, grade, nickname) VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(i16::from(student.grade.get()))
        .bind(student.nickname)
        .execute(tx.as_mut())
        .await
        .map_err(map_db_err)?;

        sqlx::query(
            "INSERT INTO google_identities (id, student_id, subject, display_name) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::now_v7())
        .bind(id)
        .bind(&subject)
        .bind(&display_name)
        .execute(tx.as_mut())
        .await
        .map_err(map_db_err)?;

        tx.commit()
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?;

        Ok(StudentAccount {
            id,
            username: student.username.to_owned(),
            nickname: student.nickname.to_owned(),
            grade: student.grade,
            // The column default: a new student follows the device until they choose in Akun.
            theme: ThemePreference::default(),
            created_at,
        })
    }
}
