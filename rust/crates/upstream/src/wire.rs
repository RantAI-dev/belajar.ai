//! Every upstream JSON shape, and its conversion to and from the ports types. Nothing else in
//! this crate names an upstream field: when the real API differs, this file is the only thing that
//! changes.
//!
//! The upstream mixes naming styles where its real API does (`documentId` camelCase beside
//! `page_url` snake_case); the tests pin those exact spellings.

use belajar_ports::upstream::{
    ChatEvent, ChatOutcome, ChatRequest, ChatRole, Mapel, PracticeAnswer, PracticeEvent,
    PracticeOption, PracticeProgress, PracticeQuestion, PracticeRequest, PracticeSet,
    PracticeStage, PracticeType, QuestionKind, QuestionScore, Source, UpstreamError,
};
use std::collections::BTreeMap;

use eventsource_stream::Event;
use serde::{Deserialize, Serialize};

use crate::streams::SseStep;

// ===== mapel =======================================================================================================

#[derive(Debug, Deserialize)]
pub(crate) struct MapelWire {
    mapel: String,
    knowledge_base_id: String,
    document_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum MapelResponseWire {
    Bare(Vec<MapelWire>),
    Wrapped { data: Vec<MapelWire> },
}

impl MapelResponseWire {
    pub(crate) fn into_mapel(self) -> Vec<Mapel> {
        let list = match self {
            Self::Bare(list) | Self::Wrapped { data: list } => list,
        };
        list.into_iter()
            .map(|wire| Mapel {
                name: wire.mapel,
                knowledge_base_id: wire.knowledge_base_id,
                document_count: wire.document_count,
            })
            .collect()
    }
}

// ===== chat request =================================================================================================

#[derive(Debug, Serialize)]
pub(crate) struct ChatBody {
    messages: Vec<MessageWire>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    mapel: Option<String>,
    sources: &'static str,
}

impl ChatBody {
    pub(crate) fn from(request: &ChatRequest) -> Self {
        Self {
            messages: request
                .messages
                .iter()
                .map(|message| MessageWire {
                    role: message.role.into(),
                    content: message.content.clone(),
                })
                .collect(),
            stream: true,
            mapel: request.mapel.clone(),
            sources: "cited",
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct MessageWire {
    role: RoleWire,
    content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum RoleWire {
    System,
    User,
    Assistant,
}

impl From<ChatRole> for RoleWire {
    fn from(role: ChatRole) -> Self {
        match role {
            ChatRole::System => RoleWire::System,
            ChatRole::User => RoleWire::User,
            ChatRole::Assistant => RoleWire::Assistant,
        }
    }
}

// ===== chat chunks ==================================================================================================

#[derive(Debug, Deserialize)]
pub(crate) struct ChatChunkWire {
    #[serde(default)]
    choices: Vec<ChoiceWire>,
    #[serde(default)]
    sources: Vec<SourceWire>,
    grounded: Option<bool>,
    retrieval_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChoiceWire {
    delta: DeltaWire,
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct DeltaWire {
    content: Option<String>,
}

// ===== sources (the upstream's own mixed-case spelling) ==============================================================

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct SourceWire {
    title: String,
    section: Option<String>,
    // The upstream spells this field camelCase; serde must match it exactly.
    #[allow(non_snake_case, reason = "the upstream's own field name on the wire")]
    documentId: String,
    page: Option<u32>,
    #[serde(default)]
    number: u32,
    cited: bool,
    page_url: Option<String>,
}

impl SourceWire {
    fn into_source(self) -> Source {
        Source {
            number: self.number,
            title: self.title,
            section: self.section,
            document_id: self.documentId,
            page: self.page,
            cited: self.cited,
            page_url: self.page_url,
        }
    }
}

// ===== practice request ==============================================================================================

#[derive(Debug, Serialize)]
pub(crate) struct GenerateBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mapel: Option<String>,
    practice_type: &'static str,
    total_questions: u8,
    stream: bool,
}

impl GenerateBody {
    pub(crate) fn from(request: &PracticeRequest) -> Self {
        // One topic is sent as `topic`, several as `topics`; the upstream reads just one of them.
        let (topic, topics) = match request.topics.as_slice() {
            [one] => (Some(one.clone()), None),
            many => (None, Some(many.to_vec())),
        };
        Self {
            topic,
            topics,
            mapel: request.mapel.clone(),
            practice_type: practice_type_name(request.practice_type),
            total_questions: request.total_questions,
            stream: true,
        }
    }
}

fn practice_type_name(practice_type: PracticeType) -> &'static str {
    match practice_type {
        PracticeType::McqNormal => "mcq_normal",
        PracticeType::McqMultipleCorrect => "mcq_multiple_correct",
        PracticeType::McqStory => "mcq_story",
        PracticeType::McqMixed => "mcq_mixed",
    }
}

// ===== practice progress ============================================================================================

#[derive(Debug, Deserialize)]
pub(crate) struct ProgressWire {
    stage: String,
    delivered: u32,
    total: u32,
    elapsed_s: f64,
}

impl ProgressWire {
    fn into_progress(self) -> Result<PracticeProgress, String> {
        let stage = match self.stage.as_str() {
            "retrieval" => PracticeStage::Retrieval,
            "generating" => PracticeStage::Generating,
            "checking" => PracticeStage::Checking,
            "labelling" => PracticeStage::Labelling,
            other => return Err(other.to_owned()),
        };
        let elapsed_ms = (self.elapsed_s * 1000.0).round().max(0.0) as u64;
        Ok(PracticeProgress {
            stage,
            delivered: self.delivered,
            total: self.total,
            elapsed_ms,
        })
    }
}

// ===== practice set =================================================================================================

// The real set, as recorded in `fake-upstream/fixtures/practice/`: a question is `question_id`,
// `type` (`mcq_single` | `mcq_multi`), `question`, an optional `scenario` (story questions),
// `options` as a map from letter to text, `answer_key` as one letter or a list, and `pembahasan`.
// Fields we do not show (`materi`, `topic`, `report`, ...) stay in `raw` for grading.

#[derive(Debug, Deserialize)]
pub(crate) struct PracticeSetWire {
    questions: Vec<QuestionWire>,
    /// Older sets put one shared story on the set instead of a `scenario` per question.
    story: Option<SharedStoryWire>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SharedStoryWire {
    title: String,
    text: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuestionWire {
    question_id: String,
    #[serde(rename = "type")]
    kind: String,
    question: String,
    scenario: Option<String>,
    /// A `BTreeMap` puts the options in letter order, whatever order the object had.
    options: BTreeMap<String, String>,
    answer_key: AnswerKeyWire,
    pembahasan: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum AnswerKeyWire {
    One(String),
    Many(Vec<String>),
}

impl AnswerKeyWire {
    fn into_labels(self) -> Vec<String> {
        match self {
            Self::One(label) => vec![label],
            Self::Many(labels) => labels,
        }
    }
}

impl PracticeSetWire {
    fn into_set(self, raw: serde_json::Value) -> Result<PracticeSet, String> {
        let shared_story = self
            .story
            .map(|story| format!("{}\n\n{}", story.title, story.text));
        let questions = self
            .questions
            .into_iter()
            .map(|wire| wire.into_question(shared_story.as_deref()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PracticeSet { questions, raw })
    }
}

impl QuestionWire {
    fn into_question(self, shared_story: Option<&str>) -> Result<PracticeQuestion, String> {
        let story = self
            .scenario
            .filter(|scenario| !scenario.trim().is_empty())
            .or_else(|| shared_story.map(str::to_owned));
        let kind = match (self.kind.as_str(), &story) {
            ("mcq_multi", _) => QuestionKind::Multiple,
            ("mcq_single", Some(_)) => QuestionKind::Story,
            ("mcq_single", None) => QuestionKind::Single,
            (other, _) => return Err(other.to_owned()),
        };
        Ok(PracticeQuestion {
            id: self.question_id,
            kind,
            story,
            stem: self.question,
            options: self
                .options
                .into_iter()
                .map(|(label, text)| PracticeOption { label, text })
                .collect(),
            answer: self.answer_key.into_labels(),
            explanation: self.pembahasan,
            // The practice API cites no page per question.
            source: None,
        })
    }
}

// ===== errors =======================================================================================================

#[derive(Debug, Deserialize)]
pub(crate) struct ErrorBodyWire {
    pub(crate) error: ErrorDetailWire,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ErrorDetailWire {
    message: String,
    code: Option<String>,
}

impl ErrorDetailWire {
    pub(crate) fn detail(self) -> String {
        match self.code {
            Some(code) => format!("{} (code: {code})", self.message),
            None => self.message,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct StreamErrorWire {
    error: ErrorDetailWire,
    status: Option<u16>,
}

// ===== grade ========================================================================================================

#[derive(Debug, Serialize)]
pub(crate) struct GradeBody {
    practice: serde_json::Value,
    /// `question_id` -> the letters picked; the real grade reads an object, not a list.
    answers: BTreeMap<String, Vec<String>>,
}

impl GradeBody {
    pub(crate) fn from(set: &PracticeSet, answers: &[PracticeAnswer]) -> Self {
        Self {
            practice: set.raw.clone(),
            answers: answers
                .iter()
                .map(|answer| (answer.question_id.clone(), answer.selected.clone()))
                .collect(),
        }
    }
}

/// Only the per-question verdicts are read: the server derives stars and mastery from them. The
/// response also carries `score`, `materi` and `recommendations`, which we do not use.
#[derive(Debug, Deserialize)]
pub(crate) struct GradeResponseWire {
    questions: Vec<QuestionScoreWire>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuestionScoreWire {
    question_id: String,
    correct: bool,
}

impl GradeResponseWire {
    pub(crate) fn into_scores(self) -> Vec<QuestionScore> {
        self.questions
            .into_iter()
            .map(|wire| QuestionScore {
                question_id: wire.question_id,
                correct: wire.correct,
                score: if wire.correct { 1.0 } else { 0.0 },
            })
            .collect()
    }
}

// ===== the two stream translators ====================================================================================

/// A `data:` chunk of an OpenAI-style chat stream, or the stream's `event: error`.
pub(crate) fn chat_translate(event: Event) -> SseStep<ChatEvent> {
    match event.event.as_str() {
        // Unnamed events are named "message" by the SSE parser.
        "message" => {
            let data = event.data.trim();
            if data == "[DONE]" {
                return SseStep::Emit(Vec::new());
            }
            let chunk: ChatChunkWire = match serde_json::from_str(data) {
                Ok(chunk) => chunk,
                Err(error) => {
                    return SseStep::Fail(UpstreamError::Protocol(format!(
                        "could not read a chat chunk: {error}"
                    )));
                }
            };
            let Some(choice) = chunk.choices.first() else {
                return SseStep::Emit(Vec::new());
            };
            let mut items = Vec::new();
            if let Some(content) = choice.delta.content.as_deref()
                && !content.is_empty()
            {
                items.push(ChatEvent::Delta(content.to_owned()));
            }
            if choice.finish_reason.as_deref() == Some("stop") {
                let outcome = ChatOutcome {
                    sources: chunk
                        .sources
                        .into_iter()
                        .map(|wire| wire.into_source())
                        .collect(),
                    grounded: chunk.grounded.unwrap_or(false),
                    retrieval_score: chunk.retrieval_score,
                };
                // The final chunk may still carry a piece of text before its outcome.
                items.push(ChatEvent::Done(outcome));
                return SseStep::Finish(items);
            }
            SseStep::Emit(items)
        }
        "error" => failed_step(event.data.trim()),
        _ => SseStep::Emit(Vec::new()),
    }
}

/// A practice stream's `progress`, `result` or `error` event.
pub(crate) fn practice_translate(event: Event) -> SseStep<PracticeEvent> {
    let data = event.data.trim();
    match event.event.as_str() {
        "progress" => match serde_json::from_str::<ProgressWire>(data) {
            Ok(wire) => match wire.into_progress() {
                Ok(progress) => SseStep::Emit(vec![PracticeEvent::Progress(progress)]),
                Err(unknown) => SseStep::Fail(UpstreamError::Protocol(format!(
                    "unknown practice stage: {unknown}"
                ))),
            },
            Err(error) => SseStep::Fail(UpstreamError::Protocol(format!(
                "could not read a progress event: {error}"
            ))),
        },
        "result" => {
            let raw = match serde_json::from_str::<serde_json::Value>(data) {
                Ok(raw) => raw,
                Err(error) => {
                    return SseStep::Fail(UpstreamError::Protocol(format!(
                        "could not read the practice result: {error}"
                    )));
                }
            };
            // The whole object is kept as `raw` for grading, so read the questions from it once.
            let wire = match serde_json::from_value::<PracticeSetWire>(raw.clone()) {
                Ok(wire) => wire,
                Err(error) => {
                    return SseStep::Fail(UpstreamError::Protocol(format!(
                        "could not read the practice questions: {error}"
                    )));
                }
            };
            match wire.into_set(raw) {
                Ok(set) => SseStep::Finish(vec![PracticeEvent::Result(set)]),
                Err(unknown) => SseStep::Fail(UpstreamError::Protocol(format!(
                    "unknown question type: {unknown}"
                ))),
            }
        }
        "error" => failed_step(data),
        // Unnamed events are not part of the practice protocol; pings never reach a translate fn.
        _ => SseStep::Emit(Vec::new()),
    }
}

fn failed_step<T>(data: &str) -> SseStep<T> {
    match serde_json::from_str::<StreamErrorWire>(data) {
        Ok(wire) => SseStep::Fail(UpstreamError::Failed {
            status: wire.status.unwrap_or(502),
            detail: wire.error.detail(),
        }),
        Err(error) => SseStep::Fail(UpstreamError::Protocol(format!(
            "could not read the stream's error event: {error}"
        ))),
    }
}

// ===== tests ========================================================================================================

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        reason = "tests"
    )]

    use belajar_core::grade::Grade;
    use belajar_ports::upstream::{
        Assistant, ChatMessage, ChatRequest, PracticeRequest, PracticeType,
    };
    use eventsource_stream::Event;

    use super::*;

    fn event(name: &str, data: &str) -> Event {
        Event {
            event: name.to_owned(),
            data: data.to_owned(),
            id: String::new(),
            retry: None,
        }
    }

    fn chat_request(mapel: Option<&str>) -> ChatRequest {
        ChatRequest {
            grade: Grade::new(7).unwrap(),
            assistant: Assistant::Ask,
            messages: vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: "Siswa kelas 7.".into(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: "Kenapa rel diberi celah?".into(),
                },
            ],
            mapel: mapel.map(str::to_owned),
        }
    }

    #[test]
    fn delta_chunks_yield_text_and_the_final_chunk_yields_the_outcome() {
        let first = chat_translate(event(
            "message",
            r#"{"id":"c1","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"Rel memuai"},"finish_reason":null}]}"#,
        ));
        let SseStep::Emit(items) = first else {
            panic!("a delta chunk does not finish the stream");
        };
        assert_eq!(items, vec![ChatEvent::Delta("Rel memuai".into())]);

        // The final chunk may carry one last piece of text beside its outcome.
        let last = chat_translate(event(
            "message",
            r#"{"id":"c1","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":" saat panas."},"finish_reason":"stop"}],"sources":[],"grounded":true,"retrieval_score":0.5}"#,
        ));
        let SseStep::Finish(items) = last else {
            panic!("the final chunk finishes the stream");
        };
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ChatEvent::Delta(" saat panas.".into()));
        let ChatEvent::Done(outcome) = &items[1] else {
            panic!("the last item is the outcome");
        };
        assert!(outcome.grounded);
        assert_eq!(outcome.retrieval_score, Some(0.5));
        assert!(outcome.sources.is_empty());
    }

    #[test]
    fn the_final_chunk_carries_the_upstreams_mixed_case_source_fields() {
        let step = chat_translate(event(
            "message",
            r#"{"choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"sources":[{"title":"IPA Kelas VII","section":"Bab 5","documentId":"kb-ipa-7","page":78,"number":1,"cited":true,"page_url":"http://u/pages/kb-ipa-7/78.svg?sig=t"},{"title":"IPA Kelas VII","section":"Bab 5","documentId":"kb-ipa-7","page":81,"number":2,"cited":false,"page_url":null}],"grounded":true,"retrieval_score":null}"#,
        ));
        let SseStep::Finish(items) = step else {
            panic!("the final chunk finishes the stream");
        };
        let ChatEvent::Done(outcome) = &items[0] else {
            panic!("the last item is the outcome");
        };
        assert_eq!(outcome.retrieval_score, None);
        let first = &outcome.sources[0];
        assert_eq!(
            (
                first.number,
                first.title.as_str(),
                first.document_id.as_str()
            ),
            (1, "IPA Kelas VII", "kb-ipa-7")
        );
        assert_eq!(first.section.as_deref(), Some("Bab 5"));
        assert_eq!(first.page, Some(78));
        assert!(first.cited);
        assert_eq!(
            first.page_url.as_deref(),
            Some("http://u/pages/kb-ipa-7/78.svg?sig=t")
        );
        assert!(!outcome.sources[1].cited);
        assert_eq!(outcome.sources[1].page_url, None);
    }

    #[test]
    fn the_done_marker_and_unnamed_noise_are_skipped() {
        // The SSE parser hands the translate fn the payload, never the `data:` line itself.
        assert!(matches!(
            chat_translate(event("message", "[DONE]")),
            SseStep::Emit(items) if items.is_empty()
        ));
        assert!(matches!(
            chat_translate(event("info", "{}")),
            SseStep::Emit(items) if items.is_empty()
        ));
    }

    #[test]
    fn a_chunk_we_cannot_parse_is_a_protocol_error() {
        assert!(matches!(
            chat_translate(event("message", "not json")),
            SseStep::Fail(UpstreamError::Protocol(_))
        ));
    }

    #[test]
    fn an_error_event_maps_to_failed_with_its_own_status() {
        let data = r#"{"error":{"message":"quota habis","type":"fake","code":"rate_limited"},"status":429}"#;
        let SseStep::Fail(error) = chat_translate(event("error", data)) else {
            panic!("an error event fails the stream");
        };
        let UpstreamError::Failed { status, detail } = error else {
            panic!("an error event maps to Failed: {error:?}")
        };
        assert_eq!(status, 429);
        assert_eq!(detail, "quota habis (code: rate_limited)");

        // Without a status the plan says 502.
        let SseStep::Fail(error) =
            chat_translate(event("error", r#"{"error":{"message":"boom"}}"#))
        else {
            panic!("an error event fails the stream");
        };
        assert!(matches!(error, UpstreamError::Failed { status: 502, .. }));
    }

    #[test]
    fn mapel_accepts_a_bare_array_and_a_data_wrapped_one() {
        let bare: MapelResponseWire = serde_json::from_str(
            r#"[{"mapel":"IPA","knowledge_base_id":"kb","document_count":3}]"#,
        )
        .unwrap();
        assert_eq!(bare.into_mapel()[0].name, "IPA");
        let wrapped: MapelResponseWire = serde_json::from_str(
            r#"{"data":[{"mapel":"IPA","knowledge_base_id":"kb","document_count":3}]}"#,
        )
        .unwrap();
        assert_eq!(wrapped.into_mapel()[0].document_count, 3);
    }

    #[test]
    fn the_chat_body_serializes_roles_mapel_and_sources() {
        let body = serde_json::to_value(ChatBody::from(&chat_request(Some("IPA")))).unwrap();
        assert_eq!(
            body["messages"],
            serde_json::json!([
                {"role": "system", "content": "Siswa kelas 7."},
                {"role": "user", "content": "Kenapa rel diberi celah?"}
            ])
        );
        assert_eq!(body["stream"], serde_json::json!(true));
        assert_eq!(body["mapel"], serde_json::json!("IPA"));
        assert_eq!(body["sources"], serde_json::json!("cited"));

        let without_mapel = serde_json::to_value(ChatBody::from(&chat_request(None))).unwrap();
        assert!(without_mapel.get("mapel").is_none());
    }

    fn practice_request(topics: Vec<&str>) -> PracticeRequest {
        PracticeRequest {
            grade: Grade::new(7).unwrap(),
            topics: topics.into_iter().map(str::to_owned).collect(),
            mapel: None,
            practice_type: PracticeType::McqNormal,
            total_questions: 5,
        }
    }

    #[test]
    fn the_generate_body_sends_topic_for_one_and_topics_for_several() {
        let one =
            serde_json::to_value(GenerateBody::from(&practice_request(vec!["Pemuaian"]))).unwrap();
        assert_eq!(one["topic"], serde_json::json!("Pemuaian"));
        assert!(one.get("topics").is_none());

        let several = serde_json::to_value(GenerateBody::from(&practice_request(vec![
            "Pemuaian", "Kalor",
        ])))
        .unwrap();
        assert_eq!(several["topics"], serde_json::json!(["Pemuaian", "Kalor"]));
        assert!(several.get("topic").is_none());
        assert_eq!(several["practice_type"], serde_json::json!("mcq_normal"));
        assert_eq!(several["total_questions"], serde_json::json!(5));
        assert_eq!(several["stream"], serde_json::json!(true));
    }

    #[test]
    fn progress_events_convert_elapsed_to_ms_and_reject_unknown_stages() {
        let wire: ProgressWire = serde_json::from_str(
            r#"{"stage":"generating","delivered":2,"total":5,"adapter_calls":7,"elapsed_s":1.5}"#,
        )
        .unwrap();
        assert_eq!(
            wire.into_progress().unwrap(),
            PracticeProgress {
                stage: PracticeStage::Generating,
                delivered: 2,
                total: 5,
                elapsed_ms: 1500,
            }
        );
        let bad: ProgressWire =
            serde_json::from_str(r#"{"stage":"dreaming","delivered":0,"total":5,"elapsed_s":0.0}"#)
                .unwrap();
        assert_eq!(bad.into_progress().unwrap_err(), "dreaming");
    }

    // Cut down from `fake-upstream/fixtures/practice/generate-mcq_mixed-json.json`.
    const RESULT: &str = r#"{"practice_type":"mcq_mixed","topic":"Pemuaian","topics":["Pemuaian"],"total_questions":3,"materi_titles":["Koefisien Muai","Gaya Angkat","Pemuaian"],"questions":[{"question_id":"q1","type":"mcq_multi","materi":"Koefisien Muai","question":"Manakah dua pernyataan berikut yang benar? (pilih dua)","options":{"C":"Benda yang sulit memuai hampir tidak berubah","A":"Benda yang mudah memuai cepat berubah ukuran","B":"Pemuaian hanya diukur dengan mata","D":"Pemuaian hanya untuk keperluan ilmiah"},"answer_key":["A","C"],"pembahasan":"Jawaban yang benar adalah A dan C.","topic":"Pemuaian"},{"question_id":"q2","type":"mcq_single","materi":"Gaya Angkat","question":"Apa yang menyebabkan wadah tersebut bisa terbang?","scenario":"Sinta membuat model balon udara dari kertas bekas.","options":{"A":"Api lilin mendorong wadah","B":"Udara panas di dalam wadah memuai dan menjadi lebih ringan"},"answer_key":"B","pembahasan":"Jawaban yang benar adalah B.","topic":"Pemuaian"},{"question_id":"q3","type":"mcq_single","materi":"Pemuaian","question":"Bertambahnya ukuran benda karena kenaikan suhu disebut ...","options":{"A":"Pemuaian","B":"Penguapan"},"answer_key":"A","pembahasan":"Jawaban yang benar adalah A.","topic":"Pemuaian"}],"report":{"requested":5,"delivered":3,"adapter_calls":6,"checks_run":6,"seconds":8.4,"rejected":{"format":0,"key_unsupported":0,"twin_keys":0,"ambiguous":0,"ai_checker":1,"duplicate":0},"skipped_topics":[]}}"#;

    fn result_set(data: &str) -> PracticeSet {
        let SseStep::Finish(items) = practice_translate(event("result", data)) else {
            panic!("the result event finishes the stream");
        };
        let Some(PracticeEvent::Result(set)) = items.into_iter().next() else {
            panic!("the result item is a set");
        };
        set
    }

    #[test]
    fn the_result_event_keeps_the_raw_set_and_converts_questions() {
        let set = result_set(RESULT);
        assert_eq!(set.raw["report"]["delivered"], serde_json::json!(3));
        assert_eq!(set.raw["topic"], serde_json::json!("Pemuaian"));

        let multi = &set.questions[0];
        assert_eq!(multi.id, "q1");
        assert_eq!(multi.kind, QuestionKind::Multiple);
        assert_eq!(multi.answer, vec!["A".to_owned(), "C".to_owned()]);
        let labels: Vec<&str> = multi.options.iter().map(|o| o.label.as_str()).collect();
        assert_eq!(labels, ["A", "B", "C", "D"], "options in letter order");
        assert_eq!(
            multi.options[2].text,
            "Benda yang sulit memuai hampir tidak berubah"
        );
        assert_eq!(multi.explanation, "Jawaban yang benar adalah A dan C.");
        assert_eq!(multi.source, None);

        let story = &set.questions[1];
        assert_eq!(story.kind, QuestionKind::Story);
        assert_eq!(
            story.story.as_deref(),
            Some("Sinta membuat model balon udara dari kertas bekas.")
        );
        assert_eq!(story.answer, vec!["B".to_owned()]);

        let single = &set.questions[2];
        assert_eq!(single.kind, QuestionKind::Single);
        assert_eq!(single.story, None);
        assert_eq!(
            single.stem,
            "Bertambahnya ukuran benda karena kenaikan suhu disebut ..."
        );
    }

    #[test]
    fn a_legacy_shared_story_becomes_the_story_of_each_question() {
        let legacy = RESULT.replacen(
            r#""total_questions":3,"#,
            r#""total_questions":3,"story":{"title":"Balon","text":"Sinta menerbangkan balon."},"#,
            1,
        );
        let set = result_set(&legacy);
        assert_eq!(set.questions[2].kind, QuestionKind::Story);
        assert_eq!(
            set.questions[2].story.as_deref(),
            Some("Balon\n\nSinta menerbangkan balon.")
        );
        // A question's own scenario wins over the shared story.
        assert_eq!(
            set.questions[1].story.as_deref(),
            Some("Sinta membuat model balon udara dari kertas bekas.")
        );
    }

    #[test]
    fn an_unknown_question_type_or_stage_is_a_protocol_error() {
        let broken = RESULT.replace("mcq_single", "mcq_esai");
        assert!(matches!(
            practice_translate(event("result", &broken)),
            SseStep::Fail(UpstreamError::Protocol(_))
        ));
        // The shape belajar once assumed (options as a list) is not the real one.
        let invented = RESULT.replace(
            r#""options":{"A":"Pemuaian","B":"Penguapan"}"#,
            r#""options":[{"label":"A","text":"Pemuaian"}]"#,
        );
        assert!(matches!(
            practice_translate(event("result", &invented)),
            SseStep::Fail(UpstreamError::Protocol(_))
        ));
    }

    #[test]
    fn the_grade_body_keeps_the_raw_set_and_sends_answers_as_an_object() {
        let set = result_set(RESULT);
        let body = serde_json::to_value(GradeBody::from(
            &set,
            &[
                PracticeAnswer {
                    question_id: "q2".into(),
                    selected: vec!["B".into()],
                },
                PracticeAnswer {
                    question_id: "q1".into(),
                    selected: vec!["C".into(), "A".into()],
                },
            ],
        ))
        .unwrap();
        assert_eq!(body["practice"], set.raw);
        assert_eq!(
            body["answers"],
            serde_json::json!({ "q1": ["C", "A"], "q2": ["B"] })
        );
    }

    #[test]
    fn grade_verdicts_map_to_scores() {
        // Cut down from `fake-upstream/fixtures/practice/grade-mcq_multiple_correct.json`.
        let wire: GradeResponseWire = serde_json::from_str(
            r#"{"score":{"correct":1,"total":2,"percentage":50},"questions":[{"question_id":"q1","materi":"Koefisien Muai","correct":true,"answered":true,"your_answer":["A","C"],"answer_key":["A","C"],"pembahasan":"..."},{"question_id":"q2","materi":"Perubahan Benda","correct":false,"answered":true,"your_answer":["A"],"answer_key":["C","D"],"pembahasan":"..."}],"materi":[],"recommendations":[]}"#,
        )
        .unwrap();
        assert_eq!(
            wire.into_scores(),
            vec![
                QuestionScore {
                    question_id: "q1".into(),
                    correct: true,
                    score: 1.0,
                },
                QuestionScore {
                    question_id: "q2".into(),
                    correct: false,
                    score: 0.0,
                },
            ]
        );
        // The invented `results` list is not the real response.
        assert!(serde_json::from_str::<GradeResponseWire>(r#"{"results":[]}"#).is_err());
    }
}
