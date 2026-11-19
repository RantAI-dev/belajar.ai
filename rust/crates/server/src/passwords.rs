//! Password hashing: argon2id with the crate's default params, hashed and verified off the async
//! runtime (`spawn_blocking`, because argon2 is deliberately slow). A password never appears in a
//! log line: only this module ever sees one.

use argon2::password_hash::{PasswordHash, SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher, PasswordVerifier};

#[derive(Debug)]
pub enum PasswordError {
    /// Hashing or verifying failed. The text is for the log, not for a response.
    Internal(String),
}

/// Returns the PHC string to store in `users.password_hash`.
pub async fn hash(password: String) -> Result<String, PasswordError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| PasswordError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| PasswordError::Internal(e.to_string()))?
}

/// Whether `password` matches the stored hash.
pub async fn verify(hash: String, password: String) -> Result<bool, PasswordError> {
    tokio::task::spawn_blocking(move || {
        let parsed =
            PasswordHash::new(&hash).map_err(|e| PasswordError::Internal(e.to_string()))?;
        let ok = Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok();
        Ok(ok)
    })
    .await
    .map_err(|e| PasswordError::Internal(e.to_string()))?
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_hashed_password_verifies_and_a_wrong_one_does_not() {
        let hash = hash("rahasia-123".to_owned()).await.unwrap();
        assert!(
            hash.starts_with("$argon2id$"),
            "argon2id, not another variant: {hash}"
        );
        assert!(
            verify(hash.clone(), "rahasia-123".to_owned())
                .await
                .unwrap()
        );
        assert!(!verify(hash, "salah-salah".to_owned()).await.unwrap());
    }

    #[tokio::test]
    async fn a_corrupt_hash_is_an_internal_error_not_a_match() {
        let err = verify("not a phc string".to_owned(), "x".repeat(8)).await;
        assert!(matches!(err, Err(PasswordError::Internal(_))));
    }
}
