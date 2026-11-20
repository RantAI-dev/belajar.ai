//! The per-attempt secrets of the authorization-code flow: state, nonce and the PKCE pair, plus
//! the constant-time comparison the callback checks them with.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

use crate::tokens;

/// 32 random bytes, base64url: 43 characters, which is also a valid PKCE `code_verifier`.
#[must_use]
pub fn random_value() -> String {
    tokens::new_token()
}

/// The PKCE `code_challenge` for `code_challenge_method=S256`.
#[must_use]
pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Equality whose time does not depend on where the first difference is.
#[must_use]
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_challenge_is_the_rfc_7636_example() {
        // RFC 7636, appendix B.
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn equal_only_when_every_byte_matches() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "abcd"));
        assert!(!constant_time_eq("", "a"));
        assert!(constant_time_eq("", ""));
    }

    #[test]
    fn values_are_valid_pkce_verifiers_and_do_not_repeat() {
        let v = random_value();
        assert_eq!(v.len(), 43, "RFC 7636 allows 43 to 128 characters");
        assert_ne!(v, random_value());
    }
}
