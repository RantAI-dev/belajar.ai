//! A llama-server `--reranking` endpoint: score each passage against the question.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct RerankBody<'a> {
    query: &'a str,
    documents: &'a [String],
}

#[derive(Deserialize)]
struct RerankOut {
    #[serde(default)]
    results: Vec<RerankResult>,
}

#[derive(Deserialize)]
struct RerankResult {
    index: usize,
    relevance_score: f64,
}

#[derive(Clone)]
pub struct Reranker {
    http: reqwest::Client,
    url: String,
}

impl Reranker {
    pub fn new(url: &str, timeout: Duration) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(timeout)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            http,
            url: url.to_owned(),
        })
    }

    /// One score per document, in the documents' order.
    pub async fn scores(&self, query: &str, documents: &[String]) -> Result<Vec<f64>, String> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }
        let response = self
            .http
            .post(format!("{}/v1/rerank", self.url))
            .json(&RerankBody { query, documents })
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err(format!("the reranker answered {}", response.status()));
        }
        let out: RerankOut = response.json().await.map_err(|e| e.to_string())?;
        Ok(scores_in_order(out.results, documents.len()))
    }
}

/// The reranker returns results sorted by score; put them back in document order. A document
/// it left out scores below everything else.
fn scores_in_order(results: Vec<RerankResult>, len: usize) -> Vec<f64> {
    let mut scores = vec![f64::NEG_INFINITY; len];
    for r in results {
        if let Some(slot) = scores.get_mut(r.index) {
            *slot = r.relevance_score;
        }
    }
    scores
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_come_back_in_document_order() {
        let results = vec![
            RerankResult {
                index: 1,
                relevance_score: 0.9,
            },
            RerankResult {
                index: 0,
                relevance_score: 0.1,
            },
            RerankResult {
                index: 7,
                relevance_score: 1.0,
            },
        ];
        assert_eq!(
            scores_in_order(results, 3),
            vec![0.1, 0.9, f64::NEG_INFINITY]
        );
    }
}
