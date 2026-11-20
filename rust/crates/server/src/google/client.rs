//! The calls to Google: the authorization URL the browser is sent to, the code-for-token
//! exchange, and the signing keys (JWKS), cached. Tokens and the client secret never reach a log
//! line or an error text from here.

use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use jsonwebtoken::jwk::JwkSet;
use serde::Deserialize;

use super::id_token::{self, Expected, IdClaims, IdTokenError};
use crate::config::GoogleConfig;

/// Google rotates keys about weekly and publishes the next one early; an hour is safe.
const KEYS_TTL: Duration = Duration::from_secs(3_600);
/// An unknown `kid` refetches the keys, but not more often than this.
const KEYS_MIN_REFRESH: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub enum GoogleError {
    /// Google could not be reached, timed out or answered 5xx. The text is for the log.
    Unavailable(String),
    /// Google refused the code, or answered something OIDC does not allow. For the log.
    Rejected(String),
    /// The ID token failed a check.
    BadIdToken(IdTokenError),
}

struct CachedKeys {
    set: Arc<JwkSet>,
    fetched: Instant,
}

pub struct GoogleClient {
    config: GoogleConfig,
    issuers: Vec<String>,
    http: reqwest::Client,
    keys: tokio::sync::Mutex<Option<CachedKeys>>,
}

#[derive(Deserialize)]
struct TokenResponse {
    id_token: String,
}

impl GoogleClient {
    pub fn new(config: GoogleConfig) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            // Google's endpoints never redirect; a redirect would only leak the code elsewhere.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            issuers: id_token::accepted_issuers(&config.issuer),
            config,
            http,
            keys: tokio::sync::Mutex::new(None),
        })
    }

    /// Where the start step sends the browser. Only `openid profile`: no email is asked for.
    pub fn authorization_url(
        &self,
        state: &str,
        nonce: &str,
        code_challenge: &str,
    ) -> Result<String, String> {
        let mut url = reqwest::Url::parse(&self.config.auth_url).map_err(|e| e.to_string())?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.redirect_url)
            .append_pair("scope", "openid profile")
            .append_pair("state", state)
            .append_pair("nonce", nonce)
            .append_pair("code_challenge", code_challenge)
            .append_pair("code_challenge_method", "S256")
            // School computers are shared: always let the child pick the account.
            .append_pair("prompt", "select_account");
        Ok(url.into())
    }

    /// Trades the authorization code for the ID token (the access token is dropped unread).
    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<String, GoogleError> {
        let form = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("code_verifier", code_verifier),
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.expose()),
            ("redirect_uri", self.config.redirect_url.as_str()),
        ];
        let response = self
            .http
            .post(&self.config.token_url)
            .form(&form)
            .send()
            .await
            .map_err(|e| GoogleError::Unavailable(without_url(&e)))?;
        let status = response.status();
        if status.is_server_error() {
            return Err(GoogleError::Unavailable(format!(
                "token endpoint answered {status}"
            )));
        }
        if !status.is_success() {
            return Err(GoogleError::Rejected(format!(
                "token endpoint answered {status}"
            )));
        }
        let body: TokenResponse = response.json().await.map_err(|_| {
            GoogleError::Rejected("token response is not the OIDC shape".to_owned())
        })?;
        Ok(body.id_token)
    }

    /// Verifies the ID token. An unknown `kid` refreshes the cached keys once, so a key rotation
    /// does not fail sign-ins.
    pub async fn verify_id_token(
        &self,
        token: &str,
        nonce: &str,
        now: DateTime<Utc>,
    ) -> Result<IdClaims, GoogleError> {
        let expected = Expected {
            client_id: &self.config.client_id,
            issuers: &self.issuers,
            nonce,
            now,
        };
        let keys = self.keys(false).await?;
        match id_token::verify(token, &keys, &expected) {
            Err(IdTokenError::UnknownKey) => {
                let keys = self.keys(true).await?;
                id_token::verify(token, &keys, &expected).map_err(GoogleError::BadIdToken)
            }
            other => other.map_err(GoogleError::BadIdToken),
        }
    }

    /// The cached key set, fetched when missing, stale, or (with `refresh`) not fetched in the
    /// last minute.
    async fn keys(&self, refresh: bool) -> Result<Arc<JwkSet>, GoogleError> {
        let mut cache = self.keys.lock().await;
        if let Some(cached) = cache.as_ref() {
            let age = cached.fetched.elapsed();
            let fresh = age < KEYS_TTL && !(refresh && age >= KEYS_MIN_REFRESH);
            if fresh {
                return Ok(cached.set.clone());
            }
        }
        let response = self
            .http
            .get(&self.config.jwks_url)
            .send()
            .await
            .map_err(|e| GoogleError::Unavailable(without_url(&e)))?;
        if !response.status().is_success() {
            return Err(GoogleError::Unavailable(format!(
                "jwks endpoint answered {}",
                response.status()
            )));
        }
        let set: JwkSet = response
            .json()
            .await
            .map_err(|_| GoogleError::Unavailable("jwks response is not a key set".to_owned()))?;
        let set = Arc::new(set);
        *cache = Some(CachedKeys {
            set: set.clone(),
            fetched: Instant::now(),
        });
        Ok(set)
    }
}

/// A reqwest error's text without the URL (the token URL carries no secret, but keep logs plain).
fn without_url(err: &reqwest::Error) -> String {
    if err.is_timeout() {
        "timed out".to_owned()
    } else if err.is_connect() {
        "could not connect".to_owned()
    } else {
        "request failed".to_owned()
    }
}
