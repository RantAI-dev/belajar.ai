//! Checks a Google ID token: RS256 signature by a key from Google's JWKS, issuer, audience,
//! expiry and the nonce this attempt sent. Pure: the caller fetches the keys and passes the time.

use chrono::{DateTime, Utc};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;

use super::flow::constant_time_eq;

/// Clock skew allowed between Google and us, in seconds.
const LEEWAY_SECONDS: i64 = 60;

/// The claims we use. Everything else in the token (email, picture, ...) is ignored and never kept.
#[derive(Debug, Clone, Deserialize)]
pub struct IdClaims {
    /// Google's stable account id: what a student is linked by.
    pub sub: String,
    /// The profile name, with the `profile` scope. Only a nickname suggestion.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    pub exp: i64,
}

/// Why a token was refused. For the log; the student only learns that it failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdTokenError {
    /// Not a JWT, or not RS256.
    Malformed,
    /// No key in the set has the token's `kid`. The caller may refresh the keys once.
    UnknownKey,
    /// Signature, issuer, audience or a required claim failed.
    Invalid,
    Expired,
    NonceMismatch,
}

impl IdTokenError {
    #[must_use]
    pub const fn reason(self) -> &'static str {
        match self {
            Self::Malformed => "id_token_malformed",
            Self::UnknownKey => "id_token_unknown_key",
            Self::Invalid => "id_token_invalid",
            Self::Expired => "id_token_expired",
            Self::NonceMismatch => "id_token_nonce_mismatch",
        }
    }
}

/// What a token must match besides its signature.
pub struct Expected<'a> {
    pub client_id: &'a str,
    /// Every accepted `iss` value.
    pub issuers: &'a [String],
    pub nonce: &'a str,
    pub now: DateTime<Utc>,
}

pub fn verify(
    token: &str,
    keys: &JwkSet,
    expected: &Expected<'_>,
) -> Result<IdClaims, IdTokenError> {
    let header = decode_header(token).map_err(|_| IdTokenError::Malformed)?;
    // Only RS256: never let the token pick a weaker algorithm (HS256 with the public key, none).
    if header.alg != Algorithm::RS256 {
        return Err(IdTokenError::Malformed);
    }
    let kid = header.kid.ok_or(IdTokenError::UnknownKey)?;
    let jwk = keys.find(&kid).ok_or(IdTokenError::UnknownKey)?;
    let key = DecodingKey::from_jwk(jwk).map_err(|_| IdTokenError::UnknownKey)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[expected.client_id]);
    validation.set_issuer(expected.issuers);
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
    // Expiry is checked below against our clock, so tests and the server agree on "now".
    validation.validate_exp = false;
    let claims = decode::<IdClaims>(token, &key, &validation)
        .map_err(|_| IdTokenError::Invalid)?
        .claims;

    if claims.exp + LEEWAY_SECONDS <= expected.now.timestamp() {
        return Err(IdTokenError::Expired);
    }
    let nonce_ok = claims
        .nonce
        .as_deref()
        .is_some_and(|nonce| constant_time_eq(nonce, expected.nonce));
    if !nonce_ok {
        return Err(IdTokenError::NonceMismatch);
    }
    Ok(claims)
}

/// The configured issuer plus its bare-host form: Google sends `https://accounts.google.com` or
/// `accounts.google.com`.
#[must_use]
pub fn accepted_issuers(issuer: &str) -> Vec<String> {
    let mut issuers = vec![issuer.to_owned()];
    if let Some(bare) = issuer.strip_prefix("https://") {
        issuers.push(bare.to_owned());
    }
    issuers
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
mod tests {
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde_json::json;

    use super::*;

    fn expected(issuers: &[String]) -> Expected<'_> {
        Expected {
            client_id: "client",
            issuers,
            nonce: "n",
            now: Utc::now(),
        }
    }

    #[test]
    fn a_token_that_is_not_rs256_is_refused_before_any_key_is_looked_up() {
        // The classic confusion: HS256 "signed" with something the attacker knows.
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("k".into());
        let claims = json!({ "sub": "1", "iss": "https://accounts.google.com", "aud": "client",
                             "exp": Utc::now().timestamp() + 600, "nonce": "n" });
        let token = encode(&header, &claims, &EncodingKey::from_secret(b"public")).unwrap();
        let issuers = accepted_issuers("https://accounts.google.com");
        let keys = JwkSet { keys: vec![] };
        assert_eq!(
            verify(&token, &keys, &expected(&issuers)).unwrap_err(),
            IdTokenError::Malformed
        );
    }

    #[test]
    fn garbage_is_malformed() {
        let issuers = accepted_issuers("https://accounts.google.com");
        let keys = JwkSet { keys: vec![] };
        assert_eq!(
            verify("not.a.jwt", &keys, &expected(&issuers)).unwrap_err(),
            IdTokenError::Malformed
        );
    }

    #[test]
    fn both_issuer_forms_are_accepted() {
        assert_eq!(
            accepted_issuers("https://accounts.google.com"),
            vec!["https://accounts.google.com", "accounts.google.com"]
        );
        assert_eq!(
            accepted_issuers("http://127.0.0.1:9"),
            vec!["http://127.0.0.1:9"]
        );
    }
}
