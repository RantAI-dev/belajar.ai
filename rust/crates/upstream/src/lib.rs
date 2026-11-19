//! The HTTP client for the upstream learning API ("Elise"): [`HttpUpstream`], the only
//! implementation of `belajar_ports::Upstream` the server needs.
//!
//! Layout: [`keys`] owns the per-grade assistant keys (a secret — it never reaches a Debug or an
//! error message), [`wire`] is every upstream JSON shape and its conversion to our ports types,
//! [`streams`] drives the SSE streams with timeouts that survive `: ping` comments, and [`chapters`]
//! is the built-in list served until the upstream ships a chapters endpoint.
//!
//! Key choice: `mapel` and `chapters` use the grade's `ask` key, `chat` uses the request's
//! assistant, `generate_practice` and `grade_practice` use `practice`. A missing key answers
//! [`UpstreamError::NotConfigured`] before any request is sent. `page` sends no key (its URL is
//! pre-signed) and refuses URLs outside the configured upstream.

mod chapters;
mod keys;
mod streams;
mod wire;

pub use keys::{KeysError, UpstreamKeys};

use std::time::Duration;

use async_trait::async_trait;
use belajar_core::grade::Grade;
use belajar_ports::upstream::{
    Assistant, Chapter, ChatRequest, ChatStream, Mapel, PageImage, PracticeAnswer, PracticeRequest,
    PracticeSet, PracticeStream, QuestionScore, Upstream, UpstreamError,
};
use bytes::Bytes;
use serde::de::DeserializeOwned;

/// Where the server finds the upstream, its keys, and how long it may stay silent.
#[derive(Debug, Clone)]
pub struct UpstreamConfig {
    /// The upstream's origin, e.g. `http://127.0.0.1:8090` for the in-repo fake. A trailing slash
    /// is tolerated.
    pub base_url: String,
    pub keys: UpstreamKeys,
    /// No response headers, and no SSE event or comment, within this time.
    pub timeout: Duration,
}

/// `Upstream` over one reqwest client. Safe to Debug: the keys print only their grade/assistant
/// pairs, never the key material.
#[derive(Debug)]
pub struct HttpUpstream {
    client: reqwest::Client,
    /// `config.base_url` without a trailing slash, so joining paths never doubles one.
    base_url: String,
    keys: UpstreamKeys,
    timeout: Duration,
}

impl HttpUpstream {
    /// Builds the client. Fails on a base URL that cannot be parsed.
    pub fn new(config: UpstreamConfig) -> Result<Self, UpstreamError> {
        let base_url = config.base_url.trim_end_matches('/').to_owned();
        if reqwest::Url::parse(&base_url).is_err() {
            return Err(UpstreamError::Protocol(format!(
                "the upstream base URL is not valid: {base_url}"
            )));
        }
        let client = reqwest::Client::builder()
            // A stored page URL must never make us fetch another host through a redirect.
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(config.timeout)
            .build()
            .map_err(|error| {
                UpstreamError::Unavailable(format!("could not build the HTTP client: {error}"))
            })?;
        Ok(Self {
            client,
            base_url,
            keys: config.keys,
            timeout: config.timeout,
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// The key for one grade and assistant, or `NotConfigured` before any request is sent.
    fn key(&self, grade: Grade, assistant: Assistant) -> Result<String, UpstreamError> {
        self.keys
            .key_for(grade, assistant)
            .ok_or(UpstreamError::NotConfigured {
                grade: grade.get(),
                assistant,
            })
            .map(str::to_owned)
    }

    /// Sends a request and waits for its headers; non-2xx becomes the error it is.
    async fn execute(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, UpstreamError> {
        let response = tokio::time::timeout(self.timeout, request.send())
            .await
            .map_err(|_| UpstreamError::Timeout)?
            .map_err(request_error)?;
        let status = response.status();
        if !status.is_success() {
            let body = self.read_error_body(response).await;
            return Err(error_from(status, &body));
        }
        Ok(response)
    }

    async fn read_error_body(&self, response: reqwest::Response) -> Bytes {
        match tokio::time::timeout(self.timeout, response.bytes()).await {
            Ok(Ok(body)) => body,
            // The status alone classifies the error; the body is only a nicer detail for the log.
            _ => Bytes::new(),
        }
    }

    /// Reads and parses one JSON body.
    async fn json<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, UpstreamError> {
        let body = tokio::time::timeout(self.timeout, response.bytes())
            .await
            .map_err(|_| UpstreamError::Timeout)?
            .map_err(request_error)?;
        serde_json::from_slice(&body).map_err(|error| {
            UpstreamError::Protocol(format!("could not read the upstream body: {error}"))
        })
    }
}

#[async_trait]
impl Upstream for HttpUpstream {
    async fn mapel(&self, grade: Grade) -> Result<Vec<Mapel>, UpstreamError> {
        let key = self.key(grade, Assistant::Ask)?;
        let request = self
            .client
            .get(self.api_url("/api/v1/mapel"))
            .bearer_auth(key);
        let response: wire::MapelResponseWire = self.json(self.execute(request).await?).await?;
        Ok(response.into_mapel())
    }

    async fn chapters(&self, grade: Grade, mapel: &str) -> Result<Vec<Chapter>, UpstreamError> {
        // Built-in list, but the key check keeps "nothing configured" behaving like every call.
        self.key(grade, Assistant::Ask)?;
        Ok(chapters::chapters_for(mapel))
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatStream, UpstreamError> {
        let key = self.key(request.grade, request.assistant)?;
        let url = self.api_url("/api/v1/chat/completions");
        let body = wire::ChatBody::from(&request);
        let response = self
            .execute(self.client.post(&url).bearer_auth(key).json(&body))
            .await?;
        let rx = streams::spawn_sse(
            response,
            self.timeout,
            UpstreamError::Protocol("the chat stream ended without a final chunk".into()),
            wire::chat_translate,
        );
        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn generate_practice(
        &self,
        request: PracticeRequest,
    ) -> Result<PracticeStream, UpstreamError> {
        let key = self.key(request.grade, Assistant::Practice)?;
        let url = self.api_url("/api/v1/practice/generate");
        let body = wire::GenerateBody::from(&request);
        let response = self
            .execute(self.client.post(&url).bearer_auth(key).json(&body))
            .await?;
        let rx = streams::spawn_sse(
            response,
            self.timeout,
            UpstreamError::Protocol("the practice stream ended without a result".into()),
            wire::practice_translate,
        );
        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn grade_practice(
        &self,
        grade: Grade,
        set: &PracticeSet,
        answers: &[PracticeAnswer],
    ) -> Result<Vec<QuestionScore>, UpstreamError> {
        let key = self.key(grade, Assistant::Practice)?;
        let url = self.api_url("/api/v1/practice/grade");
        let body = wire::GradeBody::from(set, answers);
        let response = self
            .execute(self.client.post(&url).bearer_auth(key).json(&body))
            .await?;
        let wire: wire::GradeResponseWire = self.json(response).await?;
        Ok(wire.into_scores())
    }

    async fn page(&self, page_url: &str) -> Result<PageImage, UpstreamError> {
        // The URL came from the upstream's own sources, but we still refuse to be pointed
        // somewhere else: it is stored data, not something the upstream signed for us just now.
        // Compare origins, not string prefixes: "http://up:8090.evil" starts with "http://up:8090".
        // The URL is signed, so it is never put in an error (errors are logged).
        if !same_origin(&self.base_url, page_url) {
            return Err(UpstreamError::Protocol(
                "the page URL is not under the configured upstream".into(),
            ));
        }
        let response = self.execute(self.client.get(page_url)).await?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
            .ok_or_else(|| {
                UpstreamError::Protocol("the page response has no content type".into())
            })?;
        let bytes = tokio::time::timeout(self.timeout, response.bytes())
            .await
            .map_err(|_| UpstreamError::Timeout)?
            .map_err(request_error)?;
        Ok(PageImage {
            content_type,
            bytes,
        })
    }
}

/// Whether `url` has the same scheme, host and port as `base`.
fn same_origin(base: &str, url: &str) -> bool {
    match (reqwest::Url::parse(base), reqwest::Url::parse(url)) {
        (Ok(base), Ok(url)) => {
            base.scheme() == url.scheme()
                && base.host_str() == url.host_str()
                && base.port_or_known_default() == url.port_or_known_default()
        }
        _ => false,
    }
}

/// A request that failed before we had a status.
fn request_error(error: reqwest::Error) -> UpstreamError {
    if error.is_timeout() {
        UpstreamError::Timeout
    } else if error.is_builder() || error.is_request() {
        UpstreamError::Protocol(format!("could not send the request: {error}"))
    } else {
        // Connecting, or a body stream that broke: either way the upstream is not usable now.
        UpstreamError::Unavailable(format!("the upstream connection failed: {error}"))
    }
}

/// The one status-to-error mapping: 429 and 5xx are unavailable, the rest carry the status and
/// whatever the upstream said about itself.
fn error_from(status: reqwest::StatusCode, body: &[u8]) -> UpstreamError {
    let code = status.as_u16();
    if code >= 500 || code == 429 {
        return UpstreamError::Unavailable(format!("the upstream responded {code}"));
    }
    let detail = serde_json::from_slice::<wire::ErrorBodyWire>(body)
        .ok()
        .map(|body| body.error.detail())
        .unwrap_or_else(|| {
            // The detail is for the log, never for a response, so a body snippet is fine there.
            let snippet: String = String::from_utf8_lossy(body).chars().take(200).collect();
            format!("the upstream responded {code}: {snippet}")
        });
    UpstreamError::Failed {
        status: code,
        detail,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "tests"
    )]

    use super::*;

    fn status(code: u16) -> reqwest::StatusCode {
        reqwest::StatusCode::from_u16(code).unwrap()
    }

    #[test]
    fn five_hundred_and_four_twenty_nine_are_unavailable() {
        assert!(matches!(
            error_from(status(500), b""),
            UpstreamError::Unavailable(_)
        ));
        assert!(matches!(
            error_from(status(503), b""),
            UpstreamError::Unavailable(_)
        ));
        assert!(matches!(
            error_from(status(429), b""),
            UpstreamError::Unavailable(_)
        ));
    }

    #[test]
    fn other_failures_carry_the_status_and_the_upstreams_message() {
        let error = error_from(
            status(401),
            br#"{"error":{"message":"bad key","type":"auth","code":"invalid_api_key"}}"#,
        );
        let UpstreamError::Failed {
            status: 401,
            detail,
        } = error
        else {
            panic!("a 401 maps to Failed: {error:?}");
        };
        assert_eq!(detail, "bad key (code: invalid_api_key)");
    }

    #[test]
    fn an_unparseable_error_body_becomes_a_short_snippet() {
        let error = error_from(status(404), b"nonsense");
        let UpstreamError::Failed {
            status: 404,
            detail,
        } = error
        else {
            panic!("a 404 maps to Failed: {error:?}");
        };
        assert!(detail.contains("nonsense"));
    }

    #[test]
    fn page_urls_must_share_the_upstream_origin_not_just_a_prefix() {
        let base = "http://127.0.0.1:8090";
        assert!(same_origin(
            base,
            "http://127.0.0.1:8090/pages/d/1.svg?sig=x"
        ));
        assert!(!same_origin(base, "http://127.0.0.1:80901/pages/d/1.svg"));
        assert!(!same_origin(
            base,
            "http://127.0.0.1:8090.evil.example/pages/d/1.svg"
        ));
        assert!(!same_origin(base, "https://127.0.0.1:8090/pages/d/1.svg"));
        assert!(!same_origin(base, "http://other:8090/pages/d/1.svg"));
        assert!(!same_origin(base, "not a url"));
    }

    #[test]
    fn an_invalid_base_url_is_rejected() {
        let error = HttpUpstream::new(UpstreamConfig {
            base_url: "not a url".into(),
            keys: UpstreamKeys::parse("").unwrap(),
            timeout: Duration::from_secs(5),
        })
        .unwrap_err();
        assert!(matches!(error, UpstreamError::Protocol(_)));
    }
}
