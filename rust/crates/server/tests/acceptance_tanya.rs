//! Acceptance tests for Tanya: conversations, the streamed answer, cited sources with a page
//! preview, memory, rename and history. Written by the lead before the slice started; the worker
//! must make them pass without editing this file.
#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]
#![allow(clippy::panic, reason = "tests")]

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use axum::http::StatusCode;
use belajar_core::grade::Grade;
use belajar_fake_upstream::{FakeConfig, FakeServer};
use belajar_ports::upstream::{
    Assistant, Chapter, ChatRequest, ChatRole, ChatStream, Mapel, PageImage, PracticeAnswer,
    PracticeRequest, PracticeSet, PracticeStream, QuestionScore, UpstreamError,
};
use belajar_ports::{SystemClock, Upstream};
use belajar_upstream::{HttpUpstream, UpstreamConfig, UpstreamKeys};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{TestApp, TestResponse};

const KEYS: &str = "7.ask=fake-7-ask,7.learn=fake-7-learn,7.practice=fake-7-practice";
const MISSING: &str = "/v1/conversations/0193b2c4-7a1e-7c3f-9d2a-5e8f1b6c4d90";

/// Delegates to the fake-backed client and remembers every chat request.
struct Recording {
    inner: HttpUpstream,
    chats: Mutex<Vec<ChatRequest>>,
}

#[async_trait]
impl Upstream for Recording {
    async fn mapel(&self, grade: Grade) -> Result<Vec<Mapel>, UpstreamError> {
        self.inner.mapel(grade).await
    }
    async fn chapters(&self, grade: Grade, mapel: &str) -> Result<Vec<Chapter>, UpstreamError> {
        self.inner.chapters(grade, mapel).await
    }
    async fn chat(&self, request: ChatRequest) -> Result<ChatStream, UpstreamError> {
        self.chats.lock().unwrap().push(request.clone());
        self.inner.chat(request).await
    }
    async fn generate_practice(&self, r: PracticeRequest) -> Result<PracticeStream, UpstreamError> {
        self.inner.generate_practice(r).await
    }
    async fn grade_practice(
        &self,
        grade: Grade,
        set: &PracticeSet,
        answers: &[PracticeAnswer],
    ) -> Result<Vec<QuestionScore>, UpstreamError> {
        self.inner.grade_practice(grade, set, answers).await
    }
    async fn page(&self, page_url: &str) -> Result<PageImage, UpstreamError> {
        self.inner.page(page_url).await
    }
}

struct World {
    app: TestApp,
    fake: FakeServer,
    upstream: Arc<Recording>,
}

async fn world(pool: PgPool) -> World {
    let fake = belajar_fake_upstream::spawn(FakeConfig::default())
        .await
        .unwrap();
    let inner = HttpUpstream::new(UpstreamConfig {
        base_url: fake.base_url.clone(),
        keys: UpstreamKeys::parse(KEYS).unwrap(),
        timeout: Duration::from_secs(10),
    })
    .unwrap();
    let upstream = Arc::new(Recording {
        inner,
        chats: Mutex::new(Vec::new()),
    });
    let app = TestApp::with_parts(pool, upstream.clone(), Arc::new(SystemClock)).await;
    World {
        app,
        fake,
        upstream,
    }
}

/// The events of an SSE body, in order: (event name, parsed data).
fn events(res: &TestResponse) -> Vec<(String, Value)> {
    res.text
        .split("\n\n")
        .filter_map(|block| {
            let mut name = String::from("message");
            let mut data = String::new();
            for line in block.lines() {
                if let Some(v) = line.strip_prefix("event:") {
                    name = v.trim().to_owned();
                } else if let Some(v) = line.strip_prefix("data:") {
                    data.push_str(v.trim_start());
                }
            }
            (!data.is_empty()).then(|| (name, serde_json::from_str(&data).unwrap()))
        })
        .collect()
}

fn event<'a>(all: &'a [(String, Value)], name: &str) -> &'a Value {
    &all.iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("no {name} event"))
        .1
}

async fn new_conversation(app: &TestApp, cookie: &str, subject: Value) -> String {
    let res = app
        .post(
            "/v1/conversations",
            json!({ "subjectId": subject }),
            Some(cookie),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text);
    res.body["id"].as_str().unwrap().to_owned()
}

async fn ask(app: &TestApp, cookie: &str, id: &str, question: &str) -> TestResponse {
    app.post(
        &format!("/v1/conversations/{id}/messages"),
        json!({ "content": question }),
        Some(cookie),
    )
    .await
}

fn without_request_id(mut body: Value) -> Value {
    body["error"]["requestId"] = Value::Null;
    body
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_conversation_starts_empty_with_a_default_title(pool: PgPool) {
    let w = world(pool).await;
    let budi = w.app.student("budi").await;
    let res = w
        .app
        .post(
            "/v1/conversations",
            json!({ "subjectId": "ipa" }),
            Some(&budi),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_eq!(res.body["mode"], "ask");
    assert_eq!(res.body["subjectId"], "ipa");
    assert!(res.body["chapterId"].is_null());
    assert!(!res.body["title"].as_str().unwrap().is_empty());
    assert!(res.body["createdAt"].is_string() && res.body["updatedAt"].is_string());

    let all = w
        .app
        .post(
            "/v1/conversations",
            json!({ "subjectId": null }),
            Some(&budi),
        )
        .await;
    assert_eq!(all.status, StatusCode::CREATED);
    assert!(all.body["subjectId"].is_null());

    let bad = w
        .app
        .post(
            "/v1/conversations",
            json!({ "subjectId": "kimia" }),
            Some(&budi),
        )
        .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(bad.body["error"]["errors"][0]["field"], "/subjectId");
}

#[sqlx::test(migrations = "../../migrations")]
async fn asking_streams_the_answer_and_stores_both_messages(pool: PgPool) {
    let w = world(pool).await;
    let budi = w.app.student("budi").await;
    let id = new_conversation(&w.app, &budi, json!("ipa")).await;

    let res = ask(&w.app, &budi, &id, "Kenapa rel kereta api diberi celah?").await;
    assert_eq!(res.status, StatusCode::OK);
    let content_type = res.headers["content-type"].to_str().unwrap();
    assert!(
        content_type.starts_with("text/event-stream"),
        "{content_type}"
    );

    let all = events(&res);
    assert_eq!(
        all.first().unwrap().0,
        "message",
        "the stored question comes first"
    );
    let question = event(&all, "message");
    assert_eq!(question["role"], "user");
    assert_eq!(question["content"], "Kenapa rel kereta api diberi celah?");
    let deltas: String = all
        .iter()
        .filter(|(n, _)| n == "delta")
        .map(|(_, d)| d["text"].as_str().unwrap())
        .collect();
    assert!(!deltas.is_empty());
    assert_eq!(all.last().unwrap().0, "done", "done is the last event");
    let answer = event(&all, "done");
    assert_eq!(answer["role"], "assistant");
    assert_eq!(
        answer["content"],
        deltas.as_str(),
        "the stored answer is the streamed text"
    );
    assert_eq!(answer["isGrounded"], true);
    assert_eq!(answer["conversationId"], id.as_str());

    let sources = answer["sources"].as_array().unwrap();
    assert!(!sources.is_empty());
    let cited = sources
        .iter()
        .find(|s| s["isCited"] == true)
        .expect("a cited source");
    assert!(cited["number"].as_u64().unwrap() >= 1);
    assert!(cited["title"].is_string());
    assert!(cited["page"].is_u64());
    let preview = cited["pagePreviewUrl"].as_str().unwrap();
    assert!(
        preview.starts_with("/v1/"),
        "the browser only ever talks to /v1: {preview}"
    );
    assert!(
        !res.text.contains(&w.fake.base_url),
        "the upstream host never reaches the browser"
    );
    assert!(!res.text.contains("page_url") && !res.text.contains("pageUrl"));

    let page = w.app.get(preview, Some(&budi)).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("image/")
    );

    let history = w
        .app
        .get(&format!("/v1/conversations/{id}/messages"), Some(&budi))
        .await;
    assert_eq!(history.status, StatusCode::OK);
    let data = history.body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["role"], "user");
    assert_eq!(data[1]["role"], "assistant");
    assert_eq!(data[1]["id"], answer["id"]);
    assert!(history.body.get("nextCursor").is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_out_of_book_question_is_answered_without_sources(pool: PgPool) {
    let w = world(pool).await;
    let budi = w.app.student("budi").await;
    let id = new_conversation(&w.app, &budi, Value::Null).await;
    let res = ask(&w.app, &budi, &id, "Siapa juara Piala Dunia 2022?").await;
    assert_eq!(res.status, StatusCode::OK);
    let answer = event(&events(&res), "done").clone();
    assert_eq!(answer["isGrounded"], false);
    assert_eq!(answer["sources"], json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_upstream_gets_the_profile_the_subject_and_the_last_ten_messages(pool: PgPool) {
    let w = world(pool).await;
    let budi = w.app.student("budi").await;
    let id = new_conversation(&w.app, &budi, json!("ipa")).await;
    for i in 0..6 {
        let res = ask(
            &w.app,
            &budi,
            &id,
            &format!("Pertanyaan nomor {i} tentang pemuaian"),
        )
        .await;
        assert_eq!(res.status, StatusCode::OK);
    }
    let chats = w.upstream.chats.lock().unwrap().clone();
    assert_eq!(chats.len(), 6);
    let first = &chats[0];
    assert_eq!(first.assistant, Assistant::Ask);
    assert_eq!(first.mapel.as_deref(), Some("IPA"));
    assert_eq!(first.grade.get(), 7);
    assert_eq!(first.messages.len(), 2, "system + the question");
    assert_eq!(first.messages[0].role, ChatRole::System);
    assert!(
        first.messages[0].content.contains("budi"),
        "the nickname is in the profile"
    );
    assert!(
        first.messages[0].content.contains('7'),
        "the grade is in the profile"
    );

    let last = &chats[5];
    // 5 earlier exchanges = 10 messages, all re-sent; then the new question.
    assert_eq!(last.messages.len(), 1 + 10 + 1);
    assert_eq!(last.messages[0].role, ChatRole::System);
    assert_eq!(last.messages[1].role, ChatRole::User);
    assert!(last.messages[1].content.contains("nomor 0"));
    assert_eq!(last.messages[11].role, ChatRole::User);
    assert!(last.messages[11].content.contains("nomor 5"));

    let res = ask(&w.app, &budi, &id, "Pertanyaan nomor 6 tentang pemuaian").await;
    assert_eq!(res.status, StatusCode::OK);
    let seventh = w.upstream.chats.lock().unwrap()[6].clone();
    assert_eq!(
        seventh.messages.len(),
        1 + 10 + 1,
        "only the last 10 messages are re-sent"
    );
    assert!(
        seventh.messages[1].content.contains("nomor 1"),
        "the oldest exchange dropped out"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_first_question_names_the_conversation_and_rename_changes_it(pool: PgPool) {
    let w = world(pool).await;
    let budi = w.app.student("budi").await;
    let id = new_conversation(&w.app, &budi, json!("ipa")).await;
    ask(&w.app, &budi, &id, "Kenapa rel kereta api diberi celah?").await;
    let got = w
        .app
        .get(&format!("/v1/conversations/{id}"), Some(&budi))
        .await;
    assert_eq!(got.body["title"], "Kenapa rel kereta api diberi celah?");

    let renamed = w
        .app
        .patch(
            &format!("/v1/conversations/{id}"),
            json!({ "title": "Pemuaian rel" }),
            Some(&budi),
        )
        .await;
    assert_eq!(renamed.status, StatusCode::OK);
    assert_eq!(renamed.body["title"], "Pemuaian rel");
    ask(&w.app, &budi, &id, "Contoh lain?").await;
    let got = w
        .app
        .get(&format!("/v1/conversations/{id}"), Some(&budi))
        .await;
    assert_eq!(
        got.body["title"], "Pemuaian rel",
        "a renamed title is never overwritten"
    );

    for title in [json!(""), json!("x".repeat(81))] {
        let bad = w
            .app
            .patch(
                &format!("/v1/conversations/{id}"),
                json!({ "title": title }),
                Some(&budi),
            )
            .await;
        assert_eq!(bad.status, StatusCode::BAD_REQUEST);
        assert_eq!(bad.body["error"]["errors"][0]["field"], "/title");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn history_lists_the_newest_conversation_first_in_pages(pool: PgPool) {
    let w = world(pool).await;
    let budi = w.app.student("budi").await;
    let mut ids = Vec::new();
    for _ in 0..3 {
        ids.push(new_conversation(&w.app, &budi, json!("ipa")).await);
    }
    // Activity moves the oldest conversation to the top.
    ask(&w.app, &budi, &ids[0], "Apa itu pemuaian?").await;

    let page1 = w
        .app
        .get("/v1/conversations?mode=ask&limit=2", Some(&budi))
        .await;
    assert_eq!(page1.status, StatusCode::OK);
    let data = page1.body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0]["id"], ids[0].as_str());
    assert_eq!(data[1]["id"], ids[2].as_str());
    let cursor = page1.body["nextCursor"].as_str().expect("a second page");

    let page2 = w
        .app
        .get(
            &format!("/v1/conversations?mode=ask&limit=2&cursor={cursor}"),
            Some(&budi),
        )
        .await;
    let data = page2.body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["id"], ids[1].as_str());
    assert!(page2.body["nextCursor"].is_null());

    let bad = w.app.get("/v1/conversations?limit=-1", Some(&budi)).await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(bad.body["error"]["code"], "invalid_request");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_question_is_validated_before_anything_is_stored(pool: PgPool) {
    let w = world(pool.clone()).await;
    let budi = w.app.student("budi").await;
    let id = new_conversation(&w.app, &budi, json!("ipa")).await;
    for content in [json!(""), json!("   "), json!("x".repeat(2001))] {
        let res = w
            .app
            .post(
                &format!("/v1/conversations/{id}/messages"),
                json!({ "content": content }),
                Some(&budi),
            )
            .await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST);
        assert_eq!(res.body["error"]["errors"][0]["field"], "/content");
    }
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unconfigured_grade_is_unavailable_not_a_crash(pool: PgPool) {
    let w = world(pool).await;
    let budi = w.app.student("budi").await;
    w.app
        .patch("/v1/me", json!({ "grade": 8 }), Some(&budi))
        .await;
    let id = new_conversation(&w.app, &budi, json!("ipa")).await;
    let res = ask(&w.app, &budi, &id, "Apa itu pemuaian?").await;
    assert_eq!(res.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(res.body["error"]["code"], "unavailable");
}

#[sqlx::test(migrations = "../../migrations")]
async fn another_students_conversation_is_not_found(pool: PgPool) {
    let w = world(pool.clone()).await;
    let budi = w.app.student("budi").await;
    let siti = w.app.student("siti").await;
    let id = new_conversation(&w.app, &budi, json!("ipa")).await;
    let answer = event(
        &events(&ask(&w.app, &budi, &id, "Apa itu pemuaian?").await),
        "done",
    )
    .clone();
    let preview = answer["sources"][0]["pagePreviewUrl"]
        .as_str()
        .unwrap()
        .to_owned();
    let conversation = format!("/v1/conversations/{id}");
    let messages = format!("/v1/conversations/{id}/messages");

    // Positive control: the owner reaches every one of these.
    assert_eq!(
        w.app.get(&conversation, Some(&budi)).await.status,
        StatusCode::OK
    );
    assert_eq!(
        w.app.get(&messages, Some(&budi)).await.status,
        StatusCode::OK
    );
    assert_eq!(
        w.app.get(&preview, Some(&budi)).await.status,
        StatusCode::OK
    );
    let rename = w
        .app
        .patch(&conversation, json!({ "title": "Punya Budi" }), Some(&budi))
        .await;
    assert_eq!(rename.status, StatusCode::OK);

    // Another student gets exactly what a missing id gets.
    let missing = without_request_id(w.app.get(MISSING, Some(&siti)).await.body);
    assert_eq!(missing["error"]["code"], "not_found");
    for res in [
        w.app.get(&conversation, Some(&siti)).await,
        w.app.get(&messages, Some(&siti)).await,
        w.app.get(&preview, Some(&siti)).await,
        w.app
            .patch(&conversation, json!({ "title": "Punya Siti" }), Some(&siti))
            .await,
        ask(&w.app, &siti, &id, "Halo?").await,
    ] {
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{}", res.text);
        assert_eq!(without_request_id(res.body), missing);
    }
    let listed = w.app.get("/v1/conversations", Some(&siti)).await;
    assert_eq!(listed.body["data"], json!([]));

    let title: String = sqlx::query_scalar("SELECT title FROM conversations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(title, "Punya Budi");
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        count, 2,
        "nothing was stored for the other student's attempt"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn conversation_endpoints_need_a_session(pool: PgPool) {
    let w = world(pool).await;
    for res in [
        w.app.get("/v1/conversations", None).await,
        w.app
            .post("/v1/conversations", json!({ "subjectId": null }), None)
            .await,
        w.app.get(MISSING, None).await,
    ] {
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert_eq!(res.body["error"]["code"], "unauthenticated");
    }
}
