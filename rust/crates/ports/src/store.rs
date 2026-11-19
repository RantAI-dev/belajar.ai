//! The one error type every store trait returns. The server maps it to a response in one place;
//! `Internal`'s text is for the log, never for a response body.

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The row does not exist, or belongs to another student. Same body either way.
    #[error("not found")]
    NotFound,
    /// A unique constraint or a stale state: the row already exists.
    #[error("conflict")]
    Conflict,
    /// The database failed. The text is the driver's error, for the log only.
    #[error("internal store error: {0}")]
    Internal(String),
}
