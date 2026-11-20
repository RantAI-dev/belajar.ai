//! The typed startup config: every environment variable, parsed once. A missing or invalid
//! variable stops the process with a message that names it (see `docs/conventions.md`).

use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// The Postgres the server (and `--bin seed-demo`) connects to.
    pub database_url: String,
    /// Where the server listens.
    pub addr: SocketAddr,
    /// The built web app (`web/dist`). Unknown paths outside `/v1` get its `index.html`, so the
    /// client-side router owns them.
    pub web_dir: PathBuf,
    /// Whether the session cookie carries `Secure`. `false` only for plain-http local development.
    pub cookie_secure: bool,
    /// The upstream learning API. Optional until the upstream slice lands.
    pub upstream_url: Option<String>,
    /// The upstream keys, raw. The upstream slice parses them.
    pub upstream_keys: Option<String>,
    /// Time to wait for the upstream's first byte, and between stream events.
    pub upstream_timeout_ms: u64,
    /// "Masuk dengan Google". `None` (the client id, secret and redirect URL all unset) hides it.
    pub google: Option<GoogleConfig>,
}

/// Google sign-in. The endpoints default to Google's real ones; tests point them at a fake.
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: Secret,
    /// Our callback, exactly as registered with Google: `https://<host>/v1/auth/google/callback`.
    pub redirect_url: String,
    pub auth_url: String,
    pub token_url: String,
    pub jwks_url: String,
    /// The ID token's `iss`. The same host without `https://` is accepted too, as Google sends
    /// either form.
    pub issuer: String,
    /// Time to wait for Google's token and key endpoints.
    pub timeout_ms: u64,
}

pub const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
pub const GOOGLE_ISSUER: &str = "https://accounts.google.com";

/// A value that must never reach a log line: `Debug` prints a placeholder.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(..)")
    }
}

impl GoogleConfig {
    /// `Ok(None)` when none of the three required variables is set; an error naming the first
    /// missing one when only some are.
    fn from_env() -> Result<Option<Self>, String> {
        const REQUIRED: [&str; 3] = [
            "BELAJAR_GOOGLE_CLIENT_ID",
            "BELAJAR_GOOGLE_CLIENT_SECRET",
            "BELAJAR_GOOGLE_REDIRECT_URL",
        ];
        let [client_id, client_secret, redirect_url] = REQUIRED.map(optional);
        let (Some(client_id), Some(client_secret), Some(redirect_url)) = (
            client_id.clone(),
            client_secret.clone(),
            redirect_url.clone(),
        ) else {
            let set = [&client_id, &client_secret, &redirect_url];
            if set.iter().all(|v| v.is_none()) {
                return Ok(None);
            }
            let missing = REQUIRED
                .iter()
                .zip(set)
                .find_map(|(name, value)| value.is_none().then_some(*name))
                .unwrap_or("BELAJAR_GOOGLE_CLIENT_ID");
            return Err(format!(
                "{missing} is required when any BELAJAR_GOOGLE_* sign-in variable is set."
            ));
        };
        let url = |name: &str, default: &str| {
            let value = optional(name).unwrap_or_else(|| default.to_owned());
            check_url(name, &value).map(|()| value)
        };
        check_url("BELAJAR_GOOGLE_REDIRECT_URL", &redirect_url)?;
        Ok(Some(Self {
            client_id,
            client_secret: Secret::new(client_secret),
            redirect_url,
            auth_url: url("BELAJAR_GOOGLE_AUTH_URL", GOOGLE_AUTH_URL)?,
            token_url: url("BELAJAR_GOOGLE_TOKEN_URL", GOOGLE_TOKEN_URL)?,
            jwks_url: url("BELAJAR_GOOGLE_JWKS_URL", GOOGLE_JWKS_URL)?,
            issuer: optional("BELAJAR_GOOGLE_ISSUER").unwrap_or_else(|| GOOGLE_ISSUER.to_owned()),
            timeout_ms: std::env::var("BELAJAR_GOOGLE_TIMEOUT_MS")
                .ok()
                .map(|v| parse_u64("BELAJAR_GOOGLE_TIMEOUT_MS", &v))
                .transpose()?
                .unwrap_or(10_000),
        }))
    }
}

fn check_url(name: &str, value: &str) -> Result<(), String> {
    if value.starts_with("https://") || value.starts_with("http://") {
        Ok(())
    } else {
        Err(format!("{name} must be an http(s) URL, not {value:?}."))
    }
}

impl Config {
    /// Reads every variable. `Err` is the message to print before exiting; it always names the
    /// variable.
    pub fn from_env() -> Result<Self, String> {
        let database_url = required("DATABASE_URL")?;
        let addr = parse_addr(
            "BELAJAR_ADDR",
            &std::env::var("BELAJAR_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_owned()),
        )?;
        Ok(Self {
            database_url,
            addr,
            web_dir: PathBuf::from(
                std::env::var("BELAJAR_WEB_DIR").unwrap_or_else(|_| "web/dist".to_owned()),
            ),
            cookie_secure: parse_bool("BELAJAR_COOKIE_SECURE", "true")?,
            upstream_url: optional("BELAJAR_UPSTREAM_URL"),
            upstream_keys: optional("BELAJAR_UPSTREAM_KEYS"),
            upstream_timeout_ms: std::env::var("BELAJAR_UPSTREAM_TIMEOUT_MS")
                .ok()
                .map(|v| parse_u64("BELAJAR_UPSTREAM_TIMEOUT_MS", &v))
                .transpose()?
                .unwrap_or(60_000),
            google: GoogleConfig::from_env()?,
        })
    }
}

fn required(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("{name} is required and is not set."))
}

fn optional(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn parse_bool(name: &str, default: &str) -> Result<bool, String> {
    let raw = std::env::var(name).unwrap_or_else(|_| default.to_owned());
    match raw.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("{name} must be true or false, not {raw:?}.")),
    }
}

fn parse_addr(name: &str, raw: &str) -> Result<SocketAddr, String> {
    raw.parse()
        .map_err(|_| format!("{name} must be an ip:port address, not {raw:?}."))
}

fn parse_u64(name: &str, raw: &str) -> Result<u64, String> {
    raw.parse()
        .map_err(|_| format!("{name} must be a number of milliseconds, not {raw:?}."))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::expect_used, reason = "tests")]
mod tests {
    use super::*;

    #[test]
    fn bool_is_true_or_false_and_nothing_else() {
        let parse = |v| parse_bool("BELAJAR_COOKIE_SECURE", v).map_err(std::mem::drop);
        assert_eq!(parse("true"), Ok(true));
        assert_eq!(parse("false"), Ok(false));
        assert!(parse("True").is_err());
        assert!(parse("1").is_err());
    }

    #[test]
    fn a_bad_address_is_rejected_with_the_variable_named() {
        assert!(parse_addr("BELAJAR_ADDR", "127.0.0.1:8080").is_ok());
        let err = parse_addr("BELAJAR_ADDR", "not an address").unwrap_err();
        assert!(err.contains("BELAJAR_ADDR"), "names it: {err}");
    }

    #[test]
    fn a_bad_timeout_is_rejected_with_the_variable_named() {
        assert_eq!(parse_u64("BELAJAR_UPSTREAM_TIMEOUT_MS", "1200"), Ok(1_200));
        let err = parse_u64("BELAJAR_UPSTREAM_TIMEOUT_MS", "soon").unwrap_err();
        assert!(
            err.contains("BELAJAR_UPSTREAM_TIMEOUT_MS"),
            "names it: {err}"
        );
    }
}
