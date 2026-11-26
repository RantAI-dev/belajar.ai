//! The knowledge base client: list the bases, search them, and fetch an original book once.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::catalog::KnowledgeBase;

#[derive(Debug, thiserror::Error)]
pub enum KbError {
    #[error("knowledge base request failed: {0}")]
    Http(String),
    #[error("knowledge base answered {status}: {body}")]
    Status { status: u16, body: String },
    #[error("knowledge base cache: {0}")]
    Cache(String),
}

/// One retrieved passage, with what the answer needs to cite it.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub content: String,
    pub document_id: String,
    #[serde(default)]
    pub document_title: String,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub retrieval: Option<Retrieval>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Retrieval {
    #[serde(default)]
    pub rerank_score: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchBody<'a> {
    query: &'a str,
    max_chunks: u32,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    knowledge_base_ids: &'a [String],
}

#[derive(Deserialize)]
struct SearchOut {
    #[serde(default)]
    chunks: Vec<Chunk>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BasesOut {
    #[serde(default)]
    knowledge_bases: Vec<KnowledgeBase>,
}

#[derive(Clone)]
pub struct Kb {
    http: reqwest::Client,
    url: String,
    key: String,
    cache_dir: PathBuf,
}

/// A document id is used as a file name: keep only what an id is made of.
fn safe_id(id: &str) -> Option<&str> {
    let ok = !id.is_empty()
        && id.len() <= 80
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    ok.then_some(id)
}

impl Kb {
    pub fn new(url: &str, key: &str, cache_dir: &Path, timeout: Duration) -> Result<Self, KbError> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(timeout)
            .build()
            .map_err(|e| KbError::Http(e.to_string()))?;
        Ok(Self {
            http,
            url: url.to_owned(),
            key: key.to_owned(),
            cache_dir: cache_dir.to_owned(),
        })
    }

    async fn check(response: reqwest::Response) -> Result<reqwest::Response, KbError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let body: String = response
            .text()
            .await
            .unwrap_or_default()
            .chars()
            .take(300)
            .collect();
        Err(KbError::Status {
            status: status.as_u16(),
            body,
        })
    }

    pub async fn bases(&self) -> Result<Vec<KnowledgeBase>, KbError> {
        let response = self
            .http
            .get(format!("{}/v1/knowledge-bases", self.url))
            .bearer_auth(&self.key)
            .send()
            .await
            .map_err(|e| KbError::Http(e.to_string()))?;
        let out: BasesOut = Self::check(response)
            .await?
            .json()
            .await
            .map_err(|e| KbError::Http(e.to_string()))?;
        Ok(out.knowledge_bases)
    }

    pub async fn search(
        &self,
        query: &str,
        bases: &[String],
        max_chunks: u32,
    ) -> Result<Vec<Chunk>, KbError> {
        let response = self
            .http
            .post(format!("{}/v1/search", self.url))
            .bearer_auth(&self.key)
            .json(&SearchBody {
                query,
                max_chunks,
                knowledge_base_ids: bases,
            })
            .send()
            .await
            .map_err(|e| KbError::Http(e.to_string()))?;
        let out: SearchOut = Self::check(response)
            .await?
            .json()
            .await
            .map_err(|e| KbError::Http(e.to_string()))?;
        Ok(out.chunks)
    }

    /// The original upload of a book, from the local cache or, once, from the knowledge base.
    /// Returns the file's path; the caller streams it.
    pub async fn original(&self, document_id: &str) -> Result<(PathBuf, String), KbError> {
        let id = safe_id(document_id).ok_or_else(|| KbError::Cache("not a document id".into()))?;
        let path = self.cache_dir.join(format!("{id}.bin"));
        let type_path = self.cache_dir.join(format!("{id}.type"));
        if let (Ok(meta), Ok(content_type)) = (
            tokio::fs::metadata(&path).await,
            tokio::fs::read_to_string(&type_path).await,
        ) && meta.len() > 0
        {
            return Ok((path, content_type));
        }
        let response = self
            .http
            .get(format!("{}/v1/documents/{id}/raw", self.url))
            .bearer_auth(&self.key)
            // A whole book is tens of MB over the tailnet: give it longer than a search.
            .timeout(Duration::from_secs(600))
            .send()
            .await
            .map_err(|e| KbError::Http(e.to_string()))?;
        let response = Self::check(response).await?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_owned();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| KbError::Http(e.to_string()))?;
        tokio::fs::create_dir_all(&self.cache_dir)
            .await
            .map_err(|e| KbError::Cache(e.to_string()))?;
        // Write then rename: a half-written book is never served.
        let partial = self
            .cache_dir
            .join(format!("{id}.{}.part", uuid::Uuid::now_v7()));
        tokio::fs::write(&partial, &bytes)
            .await
            .map_err(|e| KbError::Cache(e.to_string()))?;
        tokio::fs::rename(&partial, &path)
            .await
            .map_err(|e| KbError::Cache(e.to_string()))?;
        tokio::fs::write(&type_path, &content_type)
            .await
            .map_err(|e| KbError::Cache(e.to_string()))?;
        Ok((path, content_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_ids_become_file_names() {
        assert_eq!(
            safe_id("bb881961-d988-49c0-93b0-a6add53ad81e"),
            Some("bb881961-d988-49c0-93b0-a6add53ad81e")
        );
        assert_eq!(safe_id("../etc/passwd"), None);
        assert_eq!(safe_id(""), None);
        assert_eq!(safe_id("a/b"), None);
    }
}
