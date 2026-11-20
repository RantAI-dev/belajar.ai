//! Every recorded response of the real practice API (`fake-upstream/fixtures/practice/`, see its
//! README for where and when) goes through the wire types here. A wire type that drifts from the
//! real contract fails this file, whatever the fake serves.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "tests"
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use belajar_ports::upstream::{
    PracticeAnswer, PracticeEvent, PracticeSet, QuestionKind, UpstreamError,
};
use eventsource_stream::Event;
use serde_json::Value;

use crate::streams::SseStep;
use crate::wire::{ErrorBodyWire, GradeBody, GradeResponseWire, practice_translate};

const TYPES: [&str; 4] = [
    "mcq_normal",
    "mcq_multiple_correct",
    "mcq_story",
    "mcq_mixed",
];

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fake-upstream/fixtures/practice")
}

fn text(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn json(name: &str) -> Value {
    serde_json::from_str(&text(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// The recorded stream as the SSE parser would hand it over: one `Event` per blank-line block.
fn events(name: &str) -> Vec<Event> {
    text(name)
        .split("\n\n")
        .filter(|block| !block.trim().is_empty())
        .map(|block| {
            let mut event = Event {
                event: "message".into(),
                data: String::new(),
                id: String::new(),
                retry: None,
            };
            for line in block.lines() {
                if let Some(name) = line.strip_prefix("event: ") {
                    event.event = name.to_owned();
                } else if let Some(data) = line.strip_prefix("data: ") {
                    event.data.push_str(data);
                }
            }
            event
        })
        .collect()
}

/// Runs a recorded stream through the translator: the progress count and how it ended.
fn replay(name: &str) -> (usize, Result<PracticeSet, UpstreamError>) {
    let mut progress = 0;
    for event in events(name) {
        match practice_translate(event) {
            SseStep::Emit(items) => {
                progress += items
                    .iter()
                    .filter(|item| matches!(item, PracticeEvent::Progress(_)))
                    .count();
            }
            SseStep::Finish(items) => match items.into_iter().next() {
                Some(PracticeEvent::Result(set)) => return (progress, Ok(set)),
                other => panic!("{name}: the stream finished without a set: {other:?}"),
            },
            SseStep::Fail(error) => return (progress, Err(error)),
        }
    }
    panic!("{name}: the stream ended without a result or an error")
}

/// The JSON (non-stream) body is the stream's `result` payload: read it the same way.
fn set_from_json(name: &str) -> PracticeSet {
    let event = Event {
        event: "result".into(),
        data: text(name),
        id: String::new(),
        retry: None,
    };
    match practice_translate(event) {
        SseStep::Finish(items) => match items.into_iter().next() {
            Some(PracticeEvent::Result(set)) => set,
            other => panic!("{name}: no set: {other:?}"),
        },
        SseStep::Fail(error) => panic!("{name}: {error}"),
        SseStep::Emit(_) => panic!("{name}: a result event must finish the stream"),
    }
}

fn kinds(set: &PracticeSet) -> Vec<QuestionKind> {
    set.questions.iter().map(|q| q.kind).collect()
}

/// What every recorded question must turn into, whatever its type.
fn assert_well_formed(name: &str, set: &PracticeSet) {
    let raw_questions = set.raw["questions"].as_array().unwrap();
    assert_eq!(set.questions.len(), raw_questions.len(), "{name}");
    assert!(!set.questions.is_empty(), "{name}");
    for (question, raw) in set.questions.iter().zip(raw_questions) {
        assert_eq!(question.id, raw["question_id"].as_str().unwrap(), "{name}");
        assert_eq!(question.stem, raw["question"].as_str().unwrap(), "{name}");
        assert_eq!(
            question.explanation,
            raw["pembahasan"].as_str().unwrap(),
            "{name}"
        );
        let labels: Vec<&str> = question.options.iter().map(|o| o.label.as_str()).collect();
        let mut sorted = labels.clone();
        sorted.sort_unstable();
        assert_eq!(labels, sorted, "{name}: options in label order");
        assert_eq!(labels.len(), raw["options"].as_object().unwrap().len());
        assert!(!question.answer.is_empty(), "{name}: {}", question.id);
        for label in &question.answer {
            assert!(labels.contains(&label.as_str()), "{name}: key {label}");
        }
        match question.kind {
            QuestionKind::Single => {
                assert_eq!(question.answer.len(), 1, "{name}");
                assert!(question.story.is_none(), "{name}");
            }
            QuestionKind::Story => {
                assert_eq!(question.answer.len(), 1, "{name}");
                assert_eq!(
                    question.story.as_deref(),
                    raw["scenario"].as_str(),
                    "{name}: the scenario is the story"
                );
            }
            QuestionKind::Multiple => assert!(question.answer.len() >= 2, "{name}"),
        }
    }
}

#[test]
fn every_recorded_set_maps_into_questions() {
    for practice_type in TYPES {
        for name in [
            format!("generate-{practice_type}-json.json"),
            format!("generate-{practice_type}-stream.sse"),
        ] {
            let set = if name.ends_with(".sse") {
                let (progress, end) = replay(&name);
                assert!(progress >= 2, "{name}: progress before the result");
                end.unwrap_or_else(|e| panic!("{name}: {e}"))
            } else {
                set_from_json(&name)
            };
            assert_well_formed(&name, &set);
            assert_eq!(set.raw["practice_type"], practice_type, "{name}");
            let kinds = kinds(&set);
            match practice_type {
                "mcq_normal" => assert!(kinds.iter().all(|k| *k == QuestionKind::Single)),
                "mcq_multiple_correct" => {
                    assert!(kinds.iter().all(|k| *k == QuestionKind::Multiple));
                }
                "mcq_story" => assert!(kinds.iter().all(|k| *k == QuestionKind::Story)),
                _ => {
                    for kind in [
                        QuestionKind::Single,
                        QuestionKind::Multiple,
                        QuestionKind::Story,
                    ] {
                        assert!(kinds.contains(&kind), "{name}: a mixed set has {kind:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn every_recorded_stream_error_fails_with_its_own_status() {
    for (name, status) in [
        ("generate-error-unknown-topic-stream.sse", 422),
        ("generate-error-generation-failed-stream.sse", 502),
    ] {
        let (_, end) = replay(name);
        match end {
            Err(UpstreamError::Failed {
                status: got,
                detail,
            }) => {
                assert_eq!(got, status, "{name}");
                assert!(detail.contains("(code: "), "{name}: {detail}");
            }
            other => panic!("{name}: {other:?}"),
        }
    }
}

#[test]
fn every_recorded_error_body_reads() {
    for name in [
        "generate-error-unknown-topic.json",
        "generate-error-generation-failed.json",
        "generate-error-bad-type.json",
        "generate-error-unknown-mapel.json",
        "generate-error-bad-key.json",
        "grade-error-answers-as-list.json",
    ] {
        let body: ErrorBodyWire =
            serde_json::from_value(json(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(!body.error.detail().is_empty(), "{name}");
    }
}

/// The recorded request's answers as the letters a student picked.
fn recorded_answers(request: &Value) -> Vec<PracticeAnswer> {
    request["answers"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(question_id, letters)| PracticeAnswer {
            question_id: question_id.clone(),
            selected: match letters {
                Value::String(one) => vec![one.clone()],
                many => many
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|l| l.as_str().unwrap().to_owned())
                    .collect(),
            },
        })
        .collect()
}

#[test]
fn the_grade_body_is_what_the_real_grade_accepted() {
    let mut names: Vec<String> = TYPES.iter().map(|t| format!("grade-{t}")).collect();
    names.push("grade-mcq_multiple_correct-subset-and-lowercase".into());
    for name in names {
        let meta = json(&format!("{name}.meta.json"));
        assert_eq!(meta["status"], 200, "{name}");
        let request = &meta["request"]["body"];
        let set = set_from_json(&format!(
            "generate-{}-json.json",
            request["practice"]["practice_type"].as_str().unwrap()
        ));
        let answers = recorded_answers(request);
        let body = serde_json::to_value(GradeBody::from(&set, &answers)).unwrap();
        assert_eq!(
            body["practice"], request["practice"],
            "{name}: the set goes back unchanged"
        );
        // The real grade reads `answers` as an object of question id -> letter or letters.
        let sent: BTreeMap<String, Vec<String>> = serde_json::from_value(body["answers"].clone())
            .unwrap_or_else(|e| panic!("{name}: answers must be an object: {e}"));
        let expected: BTreeMap<String, Vec<String>> = answers
            .into_iter()
            .map(|a| (a.question_id, a.selected))
            .collect();
        assert_eq!(sent, expected, "{name}");
    }
}

#[test]
fn every_recorded_grade_maps_to_one_score_per_question() {
    let mut names: Vec<String> = TYPES.iter().map(|t| format!("grade-{t}")).collect();
    names.push("grade-mcq_multiple_correct-subset-and-lowercase".into());
    for name in names {
        let response = json(&format!("{name}.json"));
        let wire: GradeResponseWire =
            serde_json::from_value(response.clone()).unwrap_or_else(|e| panic!("{name}: {e}"));
        let scores = wire.into_scores();
        let recorded = response["questions"].as_array().unwrap();
        assert_eq!(scores.len(), recorded.len(), "{name}");
        for (score, raw) in scores.iter().zip(recorded) {
            assert_eq!(score.question_id, raw["question_id"].as_str().unwrap());
            assert_eq!(score.correct, raw["correct"].as_bool().unwrap(), "{name}");
            let expected = if score.correct { 1.0 } else { 0.0 };
            assert!((score.score - expected).abs() < f64::EPSILON, "{name}");
        }
        // One right, one wrong in every recorded grading.
        assert!(scores.iter().any(|s| s.correct) && scores.iter().any(|s| !s.correct));
    }
}
