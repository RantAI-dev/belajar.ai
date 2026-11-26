//! `POST /api/v1/practice/generate` and `/grade`: multiple-choice questions written by the model
//! from the books, checked here before they reach a student, and graded against their own key.

use std::collections::HashMap;
use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::rag::Passage;
use crate::{AppState, Grade, error_frame, error_response, event_frame, prompt, sse_channel};

/// The model writes this many questions per call: small batches keep each call well inside
/// its window and let progress move.
const BATCH: usize = 5;
const BATCH_TOKENS: u32 = 2_200;
const LABELS: [&str; 4] = ["A", "B", "C", "D"];
/// Every batch re-reads its passages; three keep each call fast and still give enough to ask.
const PRACTICE_PASSAGES: usize = 3;

#[derive(Deserialize)]
pub(crate) struct GenerateIn {
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    topics: Option<Vec<String>>,
    #[serde(default)]
    mapel: Option<String>,
    #[serde(default)]
    practice_type: Option<String>,
    #[serde(default)]
    total_questions: Option<u8>,
}

/// One question as the model writes it (before checks).
#[derive(Debug, Deserialize)]
struct Draft {
    #[serde(default)]
    story: Option<String>,
    question: String,
    options: Vec<String>,
    answer: Vec<String>,
    #[serde(default)]
    explanation: String,
    #[serde(default)]
    source: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Drafts {
    questions: Vec<Draft>,
}

/// The shape the model must write, per kind: the answer count is fixed by the schema, and a
/// story question cannot leave its story out.
fn schema(kind: &str) -> Value {
    let answers = if kind == "multiple" { 2 } else { 1 };
    let mut properties = json!({
        "question": { "type": "string", "minLength": 5 },
        "options": { "type": "array", "items": { "type": "string", "minLength": 1 }, "minItems": 4, "maxItems": 4 },
        "answer": { "type": "array", "items": { "type": "string", "enum": LABELS }, "minItems": answers, "maxItems": answers },
        "explanation": { "type": "string" },
        "source": { "type": "integer" }
    });
    let mut required = vec!["question", "options", "answer", "explanation", "source"];
    if kind == "story"
        && let Some(map) = properties.as_object_mut()
    {
        map.insert("story".into(), json!({ "type": "string", "minLength": 40 }));
        required.insert(0, "story");
    }
    json!({
        "type": "object",
        "properties": {
            "questions": {
                "type": "array",
                "items": { "type": "object", "properties": properties, "required": required }
            }
        },
        "required": ["questions"]
    })
}

/// The kind of each question of a set: one kind, or the three in equal blocks (one block per
/// model call, since the model keeps one shape better than a mix).
fn kinds(practice_type: &str, total: usize) -> Vec<&'static str> {
    (0..total)
        .map(|i| match practice_type {
            "mcq_multiple_correct" => "multiple",
            "mcq_story" => "story",
            "mcq_mixed" => match i * 3 / total.max(1) {
                0 => "single",
                1 => "multiple",
                _ => "story",
            },
            _ => "single",
        })
        .collect()
}

fn wire_type(kind: &str) -> &'static str {
    match kind {
        "multiple" => "mcq_multiple_correct",
        "story" => "mcq_story",
        _ => "mcq_normal",
    }
}

/// A draft that a student can answer, or None: four options, the answer labels among them,
/// exactly two answers for a multiple-answer question and one otherwise, a story when asked.
fn check(draft: Draft, kind: &str) -> Option<Draft> {
    let mut draft = draft;
    let options: Vec<String> = draft
        .options
        .iter()
        // The model sometimes repeats the label inside the option: "A. Termometer".
        .map(|o| {
            let o = o.trim();
            match o.get(..2) {
                Some(head)
                    if LABELS
                        .iter()
                        .any(|l| head == format!("{l}.") || head == format!("{l})")) =>
                {
                    o.get(2..).unwrap_or_default().trim().to_owned()
                }
                _ => o.to_owned(),
            }
        })
        .collect();
    if options.len() != 4
        || options.iter().any(String::is_empty)
        || draft.question.trim().is_empty()
    {
        return None;
    }
    let mut answer: Vec<String> = draft
        .answer
        .iter()
        .map(|a| a.trim().to_uppercase())
        .collect();
    answer.sort();
    answer.dedup();
    if answer.iter().any(|a| !LABELS.contains(&a.as_str())) {
        return None;
    }
    let wanted = if kind == "multiple" { 2 } else { 1 };
    if answer.len() != wanted {
        return None;
    }
    if kind == "story" && draft.story.as_deref().is_none_or(|s| s.trim().is_empty()) {
        return None;
    }
    if kind != "story" {
        draft.story = None;
    }
    draft.options = options;
    draft.answer = answer;
    Some(draft)
}

fn question_json(
    state: &AppState,
    index: usize,
    kind: &str,
    draft: Draft,
    passages: &[Passage],
) -> Value {
    let source = draft
        .source
        .and_then(|n| passages.iter().find(|p| p.number == n))
        .or_else(|| passages.first())
        .map(|p| {
            json!({
                "title": p.chunk.document_title,
                "section": p.chunk.section,
                "documentId": p.chunk.document_id,
                "page": p.pdf_page(),
                "number": 1,
                "cited": true,
                "page_url": state.page_url(&p.chunk.document_id, p.pdf_page()),
            })
        });
    json!({
        "id": format!("q{}", index + 1),
        "type": wire_type(kind),
        "story": draft.story,
        "question": draft.question.trim(),
        "options": draft.options.iter().zip(LABELS).map(|(text, label)| json!({ "label": label, "text": text })).collect::<Vec<_>>(),
        "answer": draft.answer,
        "explanation": draft.explanation.trim(),
        "source": source,
    })
}

fn progress(stage: &str, delivered: usize, total: usize, started: Instant) -> bytes::Bytes {
    event_frame(
        "progress",
        &json!({
            "stage": stage,
            "delivered": delivered,
            "total": total,
            "elapsed_s": started.elapsed().as_secs_f64(),
        }),
    )
}

pub(crate) async fn generate(
    State(state): State<AppState>,
    Extension(Grade(grade)): Extension<Grade>,
    Json(request): Json<GenerateIn>,
) -> Response {
    let (tx, response) = sse_channel();
    tokio::spawn(async move {
        let started = Instant::now();
        let mut topics = request.topics.clone().unwrap_or_default();
        if let Some(topic) = request.topic.clone() {
            topics.push(topic);
        }
        let topics = topics.join(", ");
        let total = usize::from(request.total_questions.unwrap_or(5)).clamp(1, 20);
        let practice_type = request
            .practice_type
            .clone()
            .unwrap_or_else(|| "mcq_normal".into());
        let _ = tx.send(progress("retrieval", 0, total, started)).await;
        let passages = match state
            .retrieve(grade, request.mapel.as_deref(), &topics)
            .await
        {
            Ok(p) if !p.is_empty() => p,
            Ok(_) => {
                let _ = tx
                    .send(error_frame("the books have nothing on this topic"))
                    .await;
                return;
            }
            Err(error) => {
                tracing::error!(%error, "practice retrieval failed");
                let _ = tx
                    .send(error_frame("the books could not be searched"))
                    .await;
                return;
            }
        };
        let wanted = kinds(&practice_type, total);
        let mut questions: Vec<Value> = Vec::new();
        let mut attempts = 0;
        // A batch can come back short (a draft failed its checks): ask again, a few times.
        while questions.len() < total && attempts < total.div_ceil(BATCH) + 4 {
            attempts += 1;
            let kind = wanted.get(questions.len()).copied().unwrap_or("single");
            // One kind per call: the model keeps one shape better than a mix.
            let run = wanted
                .iter()
                .skip(questions.len())
                .take(BATCH)
                .take_while(|k| **k == kind)
                .count()
                .max(1);
            let messages = prompt::practice_messages(
                grade,
                &topics,
                kind,
                run,
                passages
                    .get(..PRACTICE_PASSAGES.min(passages.len()))
                    .unwrap_or_default(),
            );
            let text = match state
                .llm
                .complete(&messages, BATCH_TOKENS, Some(&schema(kind)))
                .await
            {
                Ok(text) => text,
                Err(error) => {
                    tracing::warn!(%error, "a practice batch failed");
                    continue;
                }
            };
            let drafts = match serde_json::from_str::<Drafts>(&text) {
                Ok(d) => d.questions,
                Err(error) => {
                    tracing::warn!(%error, "a practice batch was not valid JSON");
                    continue;
                }
            };
            for draft in drafts.into_iter().take(run) {
                if let Some(draft) = check(draft, kind) {
                    let index = questions.len();
                    questions.push(question_json(&state, index, kind, draft, &passages));
                }
            }
            let _ = tx
                .send(progress("generating", questions.len(), total, started))
                .await;
        }
        if questions.is_empty() {
            let _ = tx.send(error_frame("no question could be written")).await;
            return;
        }
        let _ = tx
            .send(progress("checking", questions.len(), total, started))
            .await;
        let _ = tx
            .send(progress("labelling", questions.len(), total, started))
            .await;
        tracing::info!(
            grade,
            wanted = total,
            got = questions.len(),
            total_ms = started.elapsed().as_millis() as u64,
            "practice written"
        );
        let set = json!({
            "practice_id": format!("pilot-{}", uuid::Uuid::now_v7()),
            "practice_type": practice_type,
            "mapel": request.mapel,
            "topic": topics,
            "questions": questions,
        });
        let _ = tx.send(event_frame("result", &set)).await;
    });
    response
}

#[derive(Deserialize)]
pub(crate) struct GradeIn {
    practice: Value,
    #[serde(default)]
    answers: Vec<AnswerIn>,
}

#[derive(Deserialize)]
pub(crate) struct AnswerIn {
    question_id: String,
    #[serde(default)]
    selected: Vec<String>,
}

#[derive(Deserialize)]
struct KeyIn {
    questions: Vec<KeyQuestion>,
}

#[derive(Deserialize)]
struct KeyQuestion {
    id: String,
    #[serde(default)]
    answer: Vec<String>,
}

/// The set comes back exactly as it was written, answer key included.
pub(crate) async fn grade(Json(request): Json<GradeIn>) -> Response {
    let Ok(key) = serde_json::from_value::<KeyIn>(request.practice) else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "the practice set could not be read",
            "invalid_practice",
        );
    };
    let keys: HashMap<String, Vec<String>> = key
        .questions
        .into_iter()
        .map(|q| (q.id, q.answer))
        .collect();
    let mut score = 0.0;
    let mut results = Vec::new();
    for answer in &request.answers {
        let Some(expected) = keys.get(&answer.question_id) else {
            continue;
        };
        let mut picked: Vec<String> = answer
            .selected
            .iter()
            .map(|s| s.trim().to_uppercase())
            .collect();
        picked.sort();
        picked.dedup();
        let mut expected = expected.clone();
        expected.sort();
        let correct = picked == expected;
        if correct {
            score += 1.0;
        }
        results.push(json!({ "question_id": answer.question_id, "correct": correct, "score": if correct { 1.0 } else { 0.0 } }));
    }
    Json(json!({ "results": results, "score": score })).into_response()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::indexing_slicing, reason = "tests")]

    use super::*;

    fn draft(options: &[&str], answer: &[&str], story: Option<&str>) -> Draft {
        Draft {
            story: story.map(str::to_owned),
            question: "Alat pengukur suhu adalah ...".into(),
            options: options.iter().map(|s| (*s).to_owned()).collect(),
            answer: answer.iter().map(|s| (*s).to_owned()).collect(),
            explanation: "Termometer mengukur suhu.".into(),
            source: Some(1),
        }
    }

    #[test]
    fn a_good_single_answer_question_passes_and_loses_its_label_prefixes() {
        let d = check(
            draft(
                &["A. Termometer", "B) Barometer", "Higrometer", "Anemometer"],
                &["a"],
                None,
            ),
            "single",
        )
        .unwrap();
        assert_eq!(
            d.options,
            ["Termometer", "Barometer", "Higrometer", "Anemometer"]
        );
        assert_eq!(d.answer, ["A"]);
    }

    #[test]
    fn broken_drafts_are_dropped() {
        assert!(check(draft(&["a", "b", "c"], &["A"], None), "single").is_none());
        assert!(check(draft(&["a", "b", "c", "d"], &["E"], None), "single").is_none());
        assert!(check(draft(&["a", "b", "c", "d"], &["A", "B"], None), "single").is_none());
        assert!(check(draft(&["a", "b", "c", "d"], &["A"], None), "multiple").is_none());
        assert!(check(draft(&["a", "b", "c", "d"], &["A"], None), "story").is_none());
        assert!(check(draft(&["a", "b", "c", "d"], &["B", "A"], None), "multiple").is_some());
        assert!(
            check(
                draft(&["a", "b", "c", "d"], &["A"], Some("Ayu ...")),
                "story"
            )
            .is_some()
        );
    }

    #[test]
    fn the_schema_fixes_the_answer_count_and_requires_a_story_when_asked() {
        let multiple = schema("multiple");
        let item = &multiple["properties"]["questions"]["items"];
        assert_eq!(item["properties"]["answer"]["minItems"], 2);
        assert!(item["properties"].get("story").is_none());
        let story = schema("story");
        let item = &story["properties"]["questions"]["items"];
        assert_eq!(item["required"][0], "story");
        assert_eq!(item["properties"]["answer"]["maxItems"], 1);
    }

    #[test]
    fn a_mixed_set_has_the_three_kinds_in_blocks() {
        assert_eq!(
            kinds("mcq_mixed", 6),
            ["single", "single", "multiple", "multiple", "story", "story"]
        );
        assert_eq!(
            kinds("mcq_mixed", 5),
            ["single", "single", "multiple", "multiple", "story"]
        );
        assert_eq!(kinds("mcq_normal", 2), ["single", "single"]);
    }
}
