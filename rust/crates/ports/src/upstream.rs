//! What the server needs from the upstream learning API ("Elise"). These are OUR types, in our
//! naming; the upstream's wire JSON (snake_case, its own field names) is translated in exactly one
//! module of `belajar-upstream`. Nothing here knows about HTTP, keys or URLs.

use async_trait::async_trait;
use belajar_core::grade::Grade;
use bytes::Bytes;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

/// Each grade has three upstream assistants, each with its own key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assistant {
    /// Tanya: free questions.
    Ask,
    /// Belajar: guided chapter sessions.
    Learn,
    /// Latihan: practice generation and grading.
    Practice,
}

#[derive(Debug, thiserror::Error)]
pub enum UpstreamError {
    /// No key is configured for this grade and assistant. Maps to 503 `unavailable`.
    #[error("no upstream key for grade {grade} assistant {assistant:?}")]
    NotConfigured { grade: u8, assistant: Assistant },
    /// The upstream could not be reached or answered 5xx/429. Maps to 503 `unavailable`.
    #[error("upstream unavailable: {0}")]
    Unavailable(String),
    /// The upstream answered with an error it reported itself (4xx, or an `error` event).
    /// Maps to 502 `upstream_failed`. The detail is for the log, never for a response.
    #[error("upstream failed with status {status}: {detail}")]
    Failed { status: u16, detail: String },
    /// No first byte, or no event, within the configured time. Maps to 504 `timeout`.
    #[error("upstream timed out")]
    Timeout,
    /// The upstream answered something we could not parse. Maps to 502 `upstream_failed`.
    #[error("upstream protocol error: {0}")]
    Protocol(String),
}

/// One subject ("mapel") the grade's knowledge base has documents for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mapel {
    /// The upstream's own name, e.g. "IPA", "Matematika". Passed back as `mapel` on requests.
    pub name: String,
    pub knowledge_base_id: String,
    pub document_count: u32,
}

/// A chapter of a subject. The upstream has no endpoint for this yet; the client serves a built-in
/// list until it does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chapter {
    /// Stable within its subject, e.g. "bab-3".
    pub id: String,
    pub number: u32,
    pub title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatRequest {
    pub grade: Grade,
    pub assistant: Assistant,
    /// Oldest first. At most one `System` message, first; then the last turns, then the new
    /// question as the final `User` message.
    pub messages: Vec<ChatMessage>,
    /// Restrict retrieval to one subject (the upstream's `Mapel::name`), or all of them.
    pub mapel: Option<String>,
}

/// A book passage an answer used. `page_url` is the upstream's signed URL: it is fetched by the
/// server (`Upstream::page`) and never sent to the browser.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    /// The citation number used in the answer text, e.g. `[1]`.
    pub number: u32,
    pub title: String,
    pub section: Option<String>,
    pub document_id: String,
    pub page: Option<u32>,
    pub cited: bool,
    pub page_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatOutcome {
    /// Empty when the answer is not grounded in the books.
    pub sources: Vec<Source>,
    pub grounded: bool,
    pub retrieval_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChatEvent {
    /// The next piece of answer text, in order.
    Delta(String),
    /// The last event of a successful answer.
    Done(ChatOutcome),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticeType {
    McqNormal,
    McqMultipleCorrect,
    McqStory,
    McqMixed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeRequest {
    pub grade: Grade,
    /// One or more topics; the upstream receives `topic` for one and `topics` for several.
    pub topics: Vec<String>,
    pub mapel: Option<String>,
    pub practice_type: PracticeType,
    /// 5, 10 or 15 (validated by the server before calling).
    pub total_questions: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PracticeStage {
    Retrieval,
    Generating,
    Checking,
    Labelling,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PracticeProgress {
    pub stage: PracticeStage,
    pub delivered: u32,
    pub total: u32,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    /// Exactly one correct option.
    Single,
    /// One or more correct options; the student must pick all of them.
    Multiple,
    /// A single-answer question with a short story before the stem.
    Story,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PracticeOption {
    /// "A", "B", "C", "D".
    pub label: String,
    pub text: String,
}

/// One generated question, answer key included. The key never leaves the server before the
/// student submits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PracticeQuestion {
    pub id: String,
    pub kind: QuestionKind,
    pub story: Option<String>,
    pub stem: String,
    pub options: Vec<PracticeOption>,
    /// Labels of the correct options.
    pub answer: Vec<String>,
    pub explanation: String,
    pub source: Option<Source>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PracticeSet {
    pub questions: Vec<PracticeQuestion>,
    /// The set exactly as the upstream sent it; `grade_practice` sends it back unchanged.
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PracticeEvent {
    Progress(PracticeProgress),
    /// The last event of a successful generation.
    Result(PracticeSet),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PracticeAnswer {
    pub question_id: String,
    /// Labels the student picked.
    pub selected: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuestionScore {
    pub question_id: String,
    pub correct: bool,
    /// 0.0 to 1.0.
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageImage {
    pub content_type: String,
    pub bytes: Bytes,
}

pub type ChatStream = BoxStream<'static, Result<ChatEvent, UpstreamError>>;
pub type PracticeStream = BoxStream<'static, Result<PracticeEvent, UpstreamError>>;

#[async_trait]
pub trait Upstream: Send + Sync {
    /// `GET /api/v1/mapel` with the grade's `Ask` key.
    async fn mapel(&self, grade: Grade) -> Result<Vec<Mapel>, UpstreamError>;

    /// Chapters of one subject. Built-in list until the upstream ships the endpoint.
    async fn chapters(&self, grade: Grade, mapel: &str) -> Result<Vec<Chapter>, UpstreamError>;

    /// `POST /api/v1/chat/completions`, streamed, `sources: "cited"`. The stream ends after
    /// `ChatEvent::Done` or after the first `Err`.
    async fn chat(&self, request: ChatRequest) -> Result<ChatStream, UpstreamError>;

    /// `POST /api/v1/practice/generate`, streamed. Pings are swallowed. The stream ends after
    /// `PracticeEvent::Result` or after the first `Err`.
    async fn generate_practice(
        &self,
        request: PracticeRequest,
    ) -> Result<PracticeStream, UpstreamError>;

    /// `POST /api/v1/practice/grade` with the set's `raw` JSON and the answers.
    async fn grade_practice(
        &self,
        grade: Grade,
        set: &PracticeSet,
        answers: &[PracticeAnswer],
    ) -> Result<Vec<QuestionScore>, UpstreamError>;

    /// Fetch a page image from a source's signed `page_url`.
    async fn page(&self, page_url: &str) -> Result<PageImage, UpstreamError>;
}
