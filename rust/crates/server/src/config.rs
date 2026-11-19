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
