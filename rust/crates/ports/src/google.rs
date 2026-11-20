//! "Masuk dengan Google": the sign-in attempts in flight, the Google accounts linked to students,
//! and the verified Google accounts still waiting for a grade. No HTTP, no SQL.

use async_trait::async_trait;
use belajar_core::grade::Grade;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::accounts::StudentAccount;
use crate::store::StoreError;

/// What the callback needs to check Google's answer: the values the start step sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoogleAttempt {
    pub state: String,
    pub nonce: String,
    pub code_verifier: String,
    /// `Some` when a signed-in student started it from Akun to link Google; `None` for a sign-in.
    pub link: Option<LinkTarget>,
}

/// Who a link attempt is for, recorded at start from the session (never from the callback).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkTarget {
    pub student_id: Uuid,
    /// The callback must arrive in this same session; a logout or another login refuses it.
    pub session_id: Uuid,
}

/// What linking a Google account to a signed-in student did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoogleLinkResult {
    /// The Google account is now linked to this student.
    Linked,
    /// It already was, to this same student: nothing changed.
    AlreadyLinked,
    /// Another student has this Google account: nothing changed.
    LinkedToAnother,
    /// This student got another Google account meanwhile (another tab): nothing changed.
    StudentHasAnother,
}

/// A verified Google account that has no student yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingGoogleSignup {
    pub subject: String,
    pub display_name: String,
}

/// The new student's account fields; the Google identity comes from the pending signup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewGoogleStudent<'a> {
    pub username: &'a str,
    pub nickname: &'a str,
    pub grade: Grade,
}

#[async_trait]
pub trait GoogleSignInStore: Send + Sync {
    /// Stores an attempt bound to the browser (`browser_token_hash`, the SHA-256 of its cookie).
    /// Expired attempts of any browser are swept on the way.
    async fn save_attempt(
        &self,
        browser_token_hash: &[u8],
        attempt: &GoogleAttempt,
        expires_at: DateTime<Utc>,
    ) -> Result<(), StoreError>;

    /// Removes and returns this browser's attempt: single use. `None` when there is none or it
    /// expired before `now`.
    async fn take_attempt(
        &self,
        browser_token_hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<GoogleAttempt>, StoreError>;

    /// The student this Google account is linked to, if any.
    async fn find_student_by_subject(&self, subject: &str) -> Result<Option<Uuid>, StoreError>;

    /// Links the Google account `subject` to `student_id`. Never moves a Google account from one
    /// student to another and never gives a student a second one.
    async fn link_to_student(
        &self,
        student_id: Uuid,
        subject: &str,
        display_name: &str,
    ) -> Result<GoogleLinkResult, StoreError>;

    /// Parks a verified Google account until the student picks a grade.
    async fn save_pending_signup(
        &self,
        token_hash: &[u8],
        signup: &PendingGoogleSignup,
        expires_at: DateTime<Utc>,
    ) -> Result<(), StoreError>;

    /// This browser's pending signup, without using it up. `None` when missing or expired.
    async fn find_pending_signup(
        &self,
        token_hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<PendingGoogleSignup>, StoreError>;

    /// Uses up the pending signup and creates the student (no password), their profile and the
    /// Google link, in one transaction. A missing or expired signup is `NotFound`; a Google
    /// account linked meanwhile (another tab won the race) or a taken username is `Conflict`.
    async fn complete_signup(
        &self,
        token_hash: &[u8],
        student: NewGoogleStudent<'_>,
        now: DateTime<Utc>,
    ) -> Result<StudentAccount, StoreError>;
}
