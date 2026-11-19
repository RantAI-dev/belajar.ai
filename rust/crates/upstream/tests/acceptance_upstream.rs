//! Acceptance tests for the upstream client and the in-repo fake. Written by the lead before the
//! slice started; the worker must make them pass without editing this file.
#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]
#![allow(clippy::panic, reason = "tests")]

use std::time::Duration;

use belajar_core::grade::Grade;
use belajar_fake_upstream::{FakeConfig, FakeServer};
use belajar_ports::Upstream;
use belajar_ports::upstream::{
    Assistant, ChatEvent, ChatMessage, ChatRequest, ChatRole, PracticeAnswer, PracticeEvent,
    PracticeRequest, PracticeSet, PracticeStage, PracticeType, QuestionKind, UpstreamError,
};
use belajar_upstream::{HttpUpstream, UpstreamConfig, UpstreamKeys};
use futures::StreamExt;

const KEYS: &str = "7.ask=fake-7-ask,7.learn=fake-7-learn,7.practice=fake-7-practice";

fn grade(n: u8) -> Grade {
    Grade::new(n).unwrap()
}

async fn setup(config: FakeConfig, keys: &str, timeout: Duration) -> (FakeServer, HttpUpstream) {
    let fake = belajar_fake_upstream::spawn(config).await.unwrap();
    let client = HttpUpstream::new(UpstreamConfig {
        base_url: fake.base_url.clone(),
        keys: UpstreamKeys::parse(keys).unwrap(),
        timeout,
    })
    .unwrap();
    (fake, client)
}

async fn default_setup() -> (FakeServer, HttpUpstream) {
    setup(FakeConfig::default(), KEYS, Duration::from_secs(10)).await
}

fn ask(question: &str, mapel: Option<&str>) -> ChatRequest {
    ChatRequest {
        grade: grade(7),
        assistant: Assistant::Ask,
        messages: vec![
            ChatMessage {
                role: ChatRole::System,
                content: "Siswa kelas 7 bernama Budi.".to_owned(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: question.to_owned(),
            },
        ],
        mapel: mapel.map(str::to_owned),
    }
}

fn practice(topic: &str, practice_type: PracticeType, total: u8) -> PracticeRequest {
    PracticeRequest {
        grade: grade(7),
        topics: vec![topic.to_owned()],
        mapel: Some("IPA".to_owned()),
        practice_type,
        total_questions: total,
    }
}

async fn generate(
    client: &HttpUpstream,
    request: PracticeRequest,
) -> (Vec<PracticeEvent>, Option<UpstreamError>) {
    let mut stream = client.generate_practice(request).await.unwrap();
    let mut events = Vec::new();
    while let Some(item) = stream.next().await {
        match item {
            Ok(event) => events.push(event),
            Err(error) => return (events, Some(error)),
        }
    }
    (events, None)
}

fn result_of(events: &[PracticeEvent]) -> PracticeSet {
    match events.last() {
        Some(PracticeEvent::Result(set)) => set.clone(),
        other => panic!("the last event is not a result: {other:?}"),
    }
}

#[tokio::test]
async fn mapel_lists_the_grade_subjects() {
    let (_fake, client) = default_setup().await;
    let mapel = client.mapel(grade(7)).await.unwrap();
    let names: Vec<&str> = mapel.iter().map(|m| m.name.as_str()).collect();
    for expected in ["IPA", "Matematika", "Bahasa Inggris", "Bahasa Indonesia"] {
        assert!(names.contains(&expected), "missing {expected} in {names:?}");
    }
    assert!(mapel.len() >= 8);
    assert!(
        mapel
            .iter()
            .all(|m| m.document_count > 0 && !m.knowledge_base_id.is_empty())
    );
}

#[tokio::test]
async fn a_grade_without_keys_is_not_configured() {
    let (_fake, client) = default_setup().await;
    let error = client.mapel(grade(8)).await.unwrap_err();
    assert!(
        matches!(error, UpstreamError::NotConfigured { grade: 8, .. }),
        "{error:?}"
    );
}

#[tokio::test]
async fn a_key_the_upstream_rejects_is_a_failure_not_a_panic() {
    let (_fake, client) = setup(
        FakeConfig::default(),
        "7.ask=wrong-key",
        Duration::from_secs(10),
    )
    .await;
    let error = client.mapel(grade(7)).await.unwrap_err();
    assert!(
        matches!(error, UpstreamError::Failed { status: 401, .. }),
        "{error:?}"
    );
}

#[tokio::test]
async fn chat_streams_deltas_then_cited_sources() {
    let (_fake, client) = default_setup().await;
    let mut stream = client
        .chat(ask("Kenapa rel kereta api diberi celah?", Some("IPA")))
        .await
        .unwrap();
    let mut text = String::new();
    let mut deltas = 0;
    let mut outcome = None;
    while let Some(event) = stream.next().await {
        match event.unwrap() {
            ChatEvent::Delta(piece) => {
                assert!(outcome.is_none(), "a delta after Done");
                deltas += 1;
                text.push_str(&piece);
            }
            ChatEvent::Done(done) => outcome = Some(done),
        }
    }
    let outcome = outcome.expect("the stream ends with Done");
    assert!(deltas >= 2, "the answer arrives in several pieces");
    assert!(!text.trim().is_empty());
    assert!(outcome.grounded);
    assert!(!outcome.sources.is_empty());
    let cited = outcome
        .sources
        .iter()
        .find(|s| s.cited)
        .expect("one cited source");
    assert!(cited.number >= 1);
    assert!(cited.page.is_some());
    let page_url = cited
        .page_url
        .clone()
        .expect("a cited source has a page_url");

    let page = client.page(&page_url).await.unwrap();
    assert!(
        page.content_type.starts_with("image/"),
        "{}",
        page.content_type
    );
    assert!(!page.bytes.is_empty());
}

#[tokio::test]
async fn an_out_of_book_question_is_ungrounded_with_no_sources() {
    let (_fake, client) = default_setup().await;
    let mut stream = client
        .chat(ask("Siapa juara Piala Dunia 2022?", None))
        .await
        .unwrap();
    let mut outcome = None;
    while let Some(event) = stream.next().await {
        if let ChatEvent::Done(done) = event.unwrap() {
            outcome = Some(done);
        }
    }
    let outcome = outcome.unwrap();
    assert!(!outcome.grounded);
    assert!(outcome.sources.is_empty());
}

#[tokio::test]
async fn practice_generation_reports_progress_then_the_set() {
    let config = FakeConfig {
        step_delay: Duration::from_millis(5),
        ping_every: Duration::from_millis(2),
    };
    let (_fake, client) = setup(config, KEYS, Duration::from_secs(10)).await;
    let (events, error) = generate(&client, practice("Pemuaian", PracticeType::McqNormal, 5)).await;
    assert!(error.is_none(), "{error:?}");

    let progress: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            PracticeEvent::Progress(p) => Some(p.clone()),
            PracticeEvent::Result(_) => None,
        })
        .collect();
    assert!(
        progress.len() >= 3,
        "several progress events before the result"
    );
    assert!(progress.iter().any(|p| p.stage == PracticeStage::Retrieval));
    assert!(
        progress
            .iter()
            .any(|p| p.stage == PracticeStage::Generating)
    );
    assert!(progress.iter().all(|p| p.total == 5 && p.delivered <= 5));
    assert!(
        progress
            .windows(2)
            .all(|w| w[0].delivered <= w[1].delivered),
        "delivered never goes down"
    );

    let set = result_of(&events);
    assert_eq!(set.questions.len(), 5);
    let mut ids: Vec<&str> = set.questions.iter().map(|q| q.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 5, "question ids are unique");
    for q in &set.questions {
        assert_eq!(q.kind, QuestionKind::Single);
        assert_eq!(q.options.len(), 4);
        assert_eq!(q.answer.len(), 1);
        assert!(q.options.iter().any(|o| o.label == q.answer[0]));
        assert!(!q.stem.is_empty() && !q.explanation.is_empty());
    }
    assert!(
        set.raw.is_object(),
        "the raw upstream set is kept for grading"
    );
}

#[tokio::test]
async fn multiple_correct_practice_has_questions_with_several_answers() {
    let (_fake, client) = default_setup().await;
    let (events, error) = generate(
        &client,
        practice("Pemuaian", PracticeType::McqMultipleCorrect, 5),
    )
    .await;
    assert!(error.is_none(), "{error:?}");
    let set = result_of(&events);
    assert!(
        set.questions
            .iter()
            .all(|q| q.kind == QuestionKind::Multiple)
    );
    assert!(set.questions.iter().any(|q| q.answer.len() >= 2));
}

#[tokio::test]
async fn story_practice_has_a_story_on_every_question() {
    let (_fake, client) = default_setup().await;
    let (events, _) = generate(&client, practice("Pemuaian", PracticeType::McqStory, 5)).await;
    let set = result_of(&events);
    assert!(
        set.questions
            .iter()
            .all(|q| q.kind == QuestionKind::Story && q.story.is_some())
    );
}

#[tokio::test]
async fn an_error_event_ends_the_stream_with_a_failure() {
    let (_fake, client) = default_setup().await;
    // The fake answers this topic with `event: error`.
    let (events, error) =
        generate(&client, practice("__error__", PracticeType::McqNormal, 5)).await;
    assert!(
        events
            .iter()
            .all(|e| matches!(e, PracticeEvent::Progress(_)))
    );
    assert!(
        matches!(error, Some(UpstreamError::Failed { .. })),
        "{error:?}"
    );
}

#[tokio::test]
async fn a_stalled_stream_times_out() {
    let config = FakeConfig {
        step_delay: Duration::from_millis(400),
        ping_every: Duration::from_secs(10),
    };
    let (_fake, client) = setup(config, KEYS, Duration::from_millis(100)).await;
    let (_, error) = generate(&client, practice("Pemuaian", PracticeType::McqNormal, 5)).await;
    assert!(matches!(error, Some(UpstreamError::Timeout)), "{error:?}");
}

#[tokio::test]
async fn grading_scores_each_question() {
    let (_fake, client) = default_setup().await;
    let (events, _) = generate(&client, practice("Pemuaian", PracticeType::McqNormal, 5)).await;
    let set = result_of(&events);
    let mut answers: Vec<PracticeAnswer> = set
        .questions
        .iter()
        .map(|q| PracticeAnswer {
            question_id: q.id.clone(),
            selected: q.answer.clone(),
        })
        .collect();
    // Make the first answer wrong.
    let wrong = set.questions[0]
        .options
        .iter()
        .find(|o| !set.questions[0].answer.contains(&o.label))
        .unwrap()
        .label
        .clone();
    answers[0].selected = vec![wrong];

    let scores = client
        .grade_practice(grade(7), &set, &answers)
        .await
        .unwrap();
    assert_eq!(scores.len(), 5);
    for score in &scores {
        let expected = score.question_id != set.questions[0].id;
        assert_eq!(score.correct, expected, "question {}", score.question_id);
        assert!((0.0..=1.0).contains(&score.score));
    }
}

#[tokio::test]
async fn chapters_are_served_for_a_subject() {
    let (_fake, client) = default_setup().await;
    let chapters = client.chapters(grade(7), "IPA").await.unwrap();
    assert!(chapters.len() >= 3);
    for (i, chapter) in chapters.iter().enumerate() {
        assert_eq!(chapter.number as usize, i + 1, "numbered from 1 in order");
        assert!(!chapter.id.is_empty() && !chapter.title.is_empty());
    }
}

#[test]
fn keys_parse_and_reject_bad_entries() {
    assert!(UpstreamKeys::parse(KEYS).is_ok());
    assert!(
        UpstreamKeys::parse("").is_ok(),
        "no keys is valid config: every call is NotConfigured"
    );
    for bad in ["7.ask", "13.ask=x", "7.teacher=x", "x.ask=y", "7.ask="] {
        assert!(
            UpstreamKeys::parse(bad).is_err(),
            "{bad} should be rejected"
        );
    }
}
