//! The student's account and session data, as the server sees it. No HTTP, no SQL: those live in
//! `belajar-server` and `belajar-store-pg`.

use async_trait::async_trait;
use belajar_core::grade::Grade;
use belajar_core::theme::ThemePreference;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::store::StoreError;

/// One student, as a response shows them. `username` is stored and served lowercased.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudentAccount {
    pub id: Uuid,
    pub username: String,
    pub nickname: String,
    pub grade: Grade,
    pub theme: ThemePreference,
    pub created_at: DateTime<Utc>,
}

/// What a login attempt needs from the user's row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginRecord {
    pub user_id: Uuid,
    pub password_hash: String,
}

/// What a session cookie proves. `session_id` lets the caller end this one session in particular.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionRecord {
    pub session_id: Uuid,
    pub student_id: Uuid,
}

#[async_trait]
pub trait AccountStore: Send + Sync {
    /// Creates the user (role student) and their profile. A taken username, in any case, is
    /// `StoreError::Conflict`. `username` arrives already lowercased.
    async fn create_student(
        &self,
        username: &str,
        password_hash: &str,
        nickname: &str,
        grade: Grade,
    ) -> Result<StudentAccount, StoreError>;

    /// The user's row for a login attempt. `username` arrives already lowercased; `None` when no
    /// such user exists, or when they have no password (signed up with Google).
    async fn find_login_by_username(
        &self,
        username: &str,
    ) -> Result<Option<LoginRecord>, StoreError>;

    /// The user's current password hash (checked before a change). `NotFound` when the student
    /// has no password (signed up with Google).
    async fn password_hash(&self, student_id: Uuid) -> Result<String, StoreError>;

    /// Replaces the password hash.
    async fn set_password_hash(
        &self,
        student_id: Uuid,
        password_hash: &str,
    ) -> Result<(), StoreError>;

    /// Stores a new session. `token_hash` is the SHA-256 of the cookie's token; the token itself
    /// is never stored. `expires_at` comes from the caller's clock.
    async fn create_session(
        &self,
        student_id: Uuid,
        token_hash: &[u8],
        expires_at: DateTime<Utc>,
    ) -> Result<Uuid, StoreError>;

    /// The live session a cookie points at. `None` when the hash is unknown or the session has
    /// expired (expired before `now`).
    async fn find_session(
        &self,
        token_hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<SessionRecord>, StoreError>;

    /// Ends one session of this student (logout).
    async fn delete_session(&self, student_id: Uuid, session_id: Uuid) -> Result<(), StoreError>;

    /// Ends every session of this student except `keep` (a password change keeps the current one).
    async fn delete_other_sessions(&self, student_id: Uuid, keep: Uuid) -> Result<(), StoreError>;

    /// The student's profile for `GET /v1/me`.
    async fn get_student(&self, student_id: Uuid) -> Result<StudentAccount, StoreError>;

    /// Updates this student's own profile, in one statement. `None` leaves that column alone.
    async fn update_profile(
        &self,
        student_id: Uuid,
        nickname: Option<&str>,
        grade: Option<Grade>,
        theme: Option<ThemePreference>,
    ) -> Result<StudentAccount, StoreError>;
}
