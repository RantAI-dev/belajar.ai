//! Every upstream JSON shape, and its conversion to and from the ports types. Nothing else in
//! this crate names an upstream field: when the real API differs, this file is the only thing that
//! changes.
//!
//! The upstream mixes naming styles where its real API does (`documentId` camelCase beside
//! `page_url` snake_case); the tests pin those exact spellings.

use belajar_ports::upstream::{
    Chapter, ChapterTopic, ChatEvent, ChatOutcome, ChatRequest, ChatRole, Mapel, PracticeAnswer,
    PracticeEvent, PracticeOption, PracticeProgress, PracticeQuestion, PracticeRequest,
    PracticeSet, PracticeStage, PracticeType, QuestionKind, QuestionScore, Source, UpstreamError,
};
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

// ===== chapters =====================================================================================================

/// `GET /api/v1/mapel/chapters` response. `object` and `mapel` are not needed by the app, so serde
/// ignores them along with any other field the upstream adds.
#[derive(Debug, Deserialize)]
pub(crate) struct ChaptersResponseWire {
    #[serde(default)]
    data: Vec<ChapterWire>,
}

#[derive(Debug, Deserialize)]
struct ChapterWire {
    title: String,
    number: u32,
    page: Option<u32>,
    #[serde(rename = "documentId")]
    document_id: String,
    #[serde(rename = "documentTitle")]
    document_title: String,
    #[serde(default)]
    topics: Vec<TopicWire>,
}

#[derive(Debug, Deserialize)]
struct TopicWire {
    title: String,
    page: Option<u32>,
}

impl ChaptersResponseWire {
    pub(crate) fn into_chapters(self) -> Vec<Chapter> {
        self.data
            .into_iter()
            .enumerate()
            .map(|(index, wire)| Chapter {
                // Books share chapter numbers, so a number alone is not unique. We keep the old
                // positional `bab-<n>` shape so existing progress rows still find their chapter.
                id: format!("bab-{}", index + 1),
                number: (index + 1) as u32,
                book_number: wire.number,
                title: wire.title,
                page: wire.page,
                document_id: wire.document_id,
                document_title: wire.document_title,
                topics: wire
                    .topics
                    .into_iter()
                    .map(|topic| ChapterTopic {
                        title: topic.title,
                        page: topic.page,
                    })
                    .collect(),
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

#[derive(Debug, Deserialize)]
pub(crate) struct PracticeSetWire {
    #[serde(default)]
    questions: Vec<QuestionWire>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuestionWire {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    story: Option<String>,
    question: String,
    #[serde(default)]
    options: Vec<OptionWire>,
    #[serde(default)]
    answer: Vec<String>,
    #[serde(default)]
    explanation: String,
    source: Option<SourceWire>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OptionWire {
    label: String,
    text: String,
}

impl PracticeSetWire {
    fn into_set(self, raw: serde_json::Value) -> Result<PracticeSet, String> {
        let questions = self
            .questions
            .into_iter()
            .map(|wire| wire.into_question())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PracticeSet { questions, raw })
    }
}

impl QuestionWire {
    fn into_question(self) -> Result<PracticeQuestion, String> {
        let kind = match self.kind.as_str() {
            "mcq_normal" => QuestionKind::Single,
            "mcq_multiple_correct" => QuestionKind::Multiple,
            "mcq_story" => QuestionKind::Story,
            other => return Err(other.to_owned()),
        };
        Ok(PracticeQuestion {
            id: self.id,
            kind,
            story: self.story,
            stem: self.question,
            options: self
                .options
                .into_iter()
                .map(|option| PracticeOption {
                    label: option.label,
                    text: option.text,
                })
                .collect(),
            answer: self.answer,
            explanation: self.explanation,
            source: self.source.map(|wire| wire.into_source()),
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
    answers: Vec<GradeAnswerWire>,
}

#[derive(Debug, Serialize)]
pub(crate) struct GradeAnswerWire {
    question_id: String,
    selected: Vec<String>,
}

impl GradeBody {
    pub(crate) fn from(set: &PracticeSet, answers: &[PracticeAnswer]) -> Self {
        Self {
            practice: set.raw.clone(),
            answers: answers
                .iter()
                .map(|answer| GradeAnswerWire {
                    question_id: answer.question_id.clone(),
                    selected: answer.selected.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct GradeResponseWire {
    #[serde(default)]
    results: Vec<QuestionScoreWire>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct QuestionScoreWire {
    question_id: String,
    correct: bool,
    score: f64,
}

impl GradeResponseWire {
    pub(crate) fn into_scores(self) -> Vec<QuestionScore> {
        self.results
            .into_iter()
            .map(|wire| QuestionScore {
                question_id: wire.question_id,
                correct: wire.correct,
                score: wire.score,
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

    const IPA_FIXTURE: &str = include_str!("../../../fixtures/chapters/IPA.json");
    const MATEMATIKA_FIXTURE: &str = include_str!("../../../fixtures/chapters/Matematika.json");
    const BING_FIXTURE: &str = include_str!("../../../fixtures/chapters/Bahasa-Inggris.json");

    fn chapters_from(raw: &str) -> Vec<Chapter> {
        serde_json::from_str::<ChaptersResponseWire>(raw)
            .unwrap()
            .into_chapters()
    }

    #[test]
    fn every_recorded_fixture_parses_into_real_chapters() {
        let cases = [
            (
                IPA_FIXTURE,
                "IPA",
                13,
                "Sistem Organisasi Kehidupan Makhluk Hidup",
            ),
            (MATEMATIKA_FIXTURE, "Matematika", 14, "Bilangan"),
            (
                BING_FIXTURE,
                "Bahasa Inggris",
                8,
                "Good morning. How are You?",
            ),
        ];
        for (raw, mapel, count, first_title) in cases {
            let chapters = chapters_from(raw);
            assert_eq!(chapters.len(), count, "{mapel} chapter count");
            assert_eq!(chapters[0].title, first_title, "{mapel} first title");
            // The old built-in list could never produce a real book title.
            assert_ne!(chapters[0].title, "Metode Ilmiah dan Pengukuran");
            for (index, chapter) in chapters.iter().enumerate() {
                assert_eq!(chapter.id, format!("bab-{}", index + 1));
                assert!(!chapter.document_id.is_empty());
                assert!(!chapter.document_title.is_empty());
                assert!(chapter.number > 0);
            }
        }
    }

    #[test]
    fn chapters_keep_the_upstreams_book_order_and_carry_topics_and_pages() {
        let chapters = chapters_from(IPA_FIXTURE);
        // The fixture has two books; their groups stay contiguous in upstream order.
        let first_book = &chapters[0].document_title;
        let (from_first_book, from_second_book) = chapters.split_at(6);
        assert!(
            from_first_book
                .iter()
                .all(|c| &c.document_title == first_book)
        );
        assert!(
            from_second_book
                .iter()
                .all(|c| &c.document_title != first_book)
        );
        // Book numbers repeat across books; the path number and id stay sequential instead.
        assert_eq!(chapters[0].book_number, chapters[6].book_number);
        assert_eq!(chapters[6].number, 7);
        assert_eq!(chapters[0].page, Some(6));
        assert_eq!(
            chapters[0].topics[0],
            ChapterTopic {
                title: "Sel sebagai Unit Struktural dan Fungsional Kehidupan".into(),
                page: Some(9),
            }
        );
        // Bahasa Inggris carries empty topic lists rather than omitting the field.
        let bing = chapters_from(BING_FIXTURE);
        assert!(bing.iter().all(|c| c.topics.is_empty()));
    }

    #[test]
    fn a_chapter_with_no_topics_and_no_page_still_parses() {
        let chapters = chapters_from(
            r#"{"object":"list","mapel":"IPA","data":[{"title":"Tanpa halaman","number":1,"page":null,"documentId":"d","documentTitle":"Buku"}]}"#,
        );
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].page, None);
        assert!(chapters[0].topics.is_empty());
    }

    #[test]
    fn an_empty_chapters_response_is_an_empty_list() {
        assert!(chapters_from(r#"{"object":"list","mapel":"IPA","data":[]}"#).is_empty());
        // A missing `data` is treated the same way.
        assert!(chapters_from(r#"{"object":"list","mapel":"IPA"}"#).is_empty());
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

    const RESULT: &str = r#"{"practice_id":"fake-1","practice_type":"mcq_normal","mapel":"IPA","topic":"Pemuaian","questions":[{"id":"q1","type":"mcq_normal","story":null,"question":"Alat untuk mengukur suhu adalah ...","options":[{"label":"A","text":"Termometer"},{"label":"B","text":"Barometer"}],"answer":["A"],"explanation":"Suhu diukur dengan termometer.","source":{"title":"IPA VII","section":"Bab 5","documentId":"kb-ipa-7","page":70,"number":1,"cited":true,"page_url":"http://u/p.svg"}}]}"#;

    #[test]
    fn the_result_event_keeps_the_raw_set_and_converts_questions() {
        let SseStep::Finish(items) = practice_translate(event("result", RESULT)) else {
            panic!("the result event finishes the stream");
        };
        let PracticeEvent::Result(set) = &items[0] else {
            panic!("the result item is a set");
        };
        assert_eq!(set.raw["practice_id"], serde_json::json!("fake-1"));
        assert_eq!(set.raw["topic"], serde_json::json!("Pemuaian"));
        let question = &set.questions[0];
        assert_eq!(question.id, "q1");
        assert_eq!(question.kind, QuestionKind::Single);
        assert_eq!(question.story, None);
        assert_eq!(question.stem, "Alat untuk mengukur suhu adalah ...");
        assert_eq!(question.answer, vec!["A".to_owned()]);
        assert_eq!(question.options[1].text, "Barometer");
        let source = question.source.as_ref().unwrap();
        assert_eq!(source.document_id, "kb-ipa-7");
        assert_eq!(source.page_url.as_deref(), Some("http://u/p.svg"));
    }

    #[test]
    fn an_unknown_question_type_or_stage_is_a_protocol_error() {
        let broken = RESULT.replace("mcq_normal", "mcq_esai");
        assert!(matches!(
            practice_translate(event("result", &broken)),
            SseStep::Fail(UpstreamError::Protocol(_))
        ));
    }

    #[test]
    fn the_grade_body_keeps_the_raw_set_and_the_answers() {
        let raw: serde_json::Value = serde_json::from_str(RESULT).unwrap();
        let set = PracticeSet {
            questions: Vec::new(),
            raw: raw.clone(),
        };
        let body = serde_json::to_value(GradeBody::from(
            &set,
            &[
                PracticeAnswer {
                    question_id: "q1".into(),
                    selected: vec!["A".into()],
                },
                PracticeAnswer {
                    question_id: "q2".into(),
                    selected: vec!["A".into(), "C".into()],
                },
            ],
        ))
        .unwrap();
        assert_eq!(body["practice"], raw);
        assert_eq!(
            body["answers"],
            serde_json::json!([
                {"question_id": "q1", "selected": ["A"]},
                {"question_id": "q2", "selected": ["A", "C"]}
            ])
        );
    }

    #[test]
    fn grade_results_map_to_scores() {
        let wire: GradeResponseWire = serde_json::from_str(
            r#"{"results":[{"question_id":"q1","correct":true,"score":1.0},{"question_id":"q2","correct":false,"score":0.0}],"score":1.0}"#,
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
    }
}
