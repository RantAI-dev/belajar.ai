//! Configuration, all from the environment. No address, key or path is built in: this crate is
//! public, the services it talks to are not.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    /// Where this service listens, e.g. `127.0.0.1:18095`.
    pub addr: String,
    /// The URL belajar-server reaches this service at; page links are built on it.
    pub public_url: String,
    /// The knowledge base (rantai-kb), e.g. `http://kb:8080`.
    pub kb_url: String,
    /// Its read-only key, read from a file so it never sits in a unit file or a shell history.
    pub kb_key: String,
    /// An OpenAI-compatible chat endpoint base, e.g. `http://llm:8000` (`/v1/chat/completions`).
    pub llm_url: String,
    /// The model name to send; llama-server ignores it.
    pub llm_model: String,
    /// A llama-server `--reranking` endpoint base (`/v1/rerank`). Without one, the knowledge
    /// base's own rerank order is used.
    pub rerank_url: Option<String>,
    /// Bearer keys belajar-server sends, each tied to a grade: `7.ask=<key>,7.learn=<key>,...`.
    pub client_keys: HashMap<String, u8>,
    /// Where original books are cached after the first download.
    pub cache_dir: PathBuf,
    /// Per request to the knowledge base, the reranker or the first token of the model.
    pub timeout: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0} is required and is not set")]
    Missing(&'static str),
    #[error("{name} is not valid: {reason}")]
    Invalid { name: &'static str, reason: String },
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    std::env::var(name)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or(ConfigError::Missing(name))
}

fn optional(name: &'static str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// `7.ask=k1, 7.learn=k2` -> {k1: 7, k2: 7}. The assistant part is not needed here: every
/// assistant of a grade reads the same books.
pub fn parse_client_keys(raw: &str) -> Result<HashMap<String, u8>, String> {
    let mut keys = HashMap::new();
    for (index, entry) in raw.split(',').enumerate() {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (left, key) = entry
            .split_once('=')
            .ok_or_else(|| format!("entry {index} has no '='"))?;
        let grade = left
            .trim()
            .split('.')
            .next()
            .and_then(|g| g.parse::<u8>().ok())
            .filter(|g| (1..=12).contains(g))
            .ok_or_else(|| format!("entry {index} does not start with a grade 1-12"))?;
        let key = key.trim();
        if key.is_empty() {
            return Err(format!("entry {index} has an empty key"));
        }
        keys.insert(key.to_owned(), grade);
    }
    if keys.is_empty() {
        return Err("no keys".into());
    }
    Ok(keys)
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let key_file = required("PILOT_KB_KEY_FILE")?;
        let kb_key = std::fs::read_to_string(&key_file)
            .map_err(|e| ConfigError::Invalid {
                name: "PILOT_KB_KEY_FILE",
                reason: format!("cannot read it: {e}"),
            })?
            .trim()
            .to_owned();
        if kb_key.is_empty() {
            return Err(ConfigError::Invalid {
                name: "PILOT_KB_KEY_FILE",
                reason: "the file is empty".into(),
            });
        }
        let client_keys = parse_client_keys(&required("PILOT_CLIENT_KEYS")?).map_err(|reason| {
            ConfigError::Invalid {
                name: "PILOT_CLIENT_KEYS",
                reason,
            }
        })?;
        let timeout_s = optional("PILOT_TIMEOUT_S")
            .map(|v| {
                v.parse::<u64>().map_err(|e| ConfigError::Invalid {
                    name: "PILOT_TIMEOUT_S",
                    reason: e.to_string(),
                })
            })
            .transpose()?
            .unwrap_or(120);
        let addr = optional("PILOT_ADDR").unwrap_or_else(|| "127.0.0.1:18095".into());
        let public_url = optional("PILOT_PUBLIC_URL").unwrap_or_else(|| format!("http://{addr}"));
        Ok(Self {
            addr,
            public_url: public_url.trim_end_matches('/').to_owned(),
            kb_url: required("PILOT_KB_URL")?.trim_end_matches('/').to_owned(),
            kb_key,
            llm_url: required("PILOT_LLM_URL")?.trim_end_matches('/').to_owned(),
            llm_model: optional("PILOT_LLM_MODEL").unwrap_or_else(|| "base".into()),
            rerank_url: optional("PILOT_RERANK_URL").map(|u| u.trim_end_matches('/').to_owned()),
            client_keys,
            cache_dir: PathBuf::from(
                optional("PILOT_CACHE_DIR").unwrap_or_else(|| "pilot-cache".into()),
            ),
            timeout: Duration::from_secs(timeout_s),
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;

    #[test]
    fn keys_map_to_their_grade() {
        let keys = parse_client_keys("7.ask=a7, 7.learn = l7 ,10.practice=p10").unwrap();
        assert_eq!(keys.get("a7"), Some(&7));
        assert_eq!(keys.get("l7"), Some(&7));
        assert_eq!(keys.get("p10"), Some(&10));
    }

    #[test]
    fn a_bad_entry_is_refused() {
        assert!(parse_client_keys("").is_err());
        assert!(parse_client_keys("7.ask").is_err());
        assert!(parse_client_keys("13.ask=x").is_err());
        assert!(parse_client_keys("x.ask=y").is_err());
        assert!(parse_client_keys("7.ask=").is_err());
    }
}
