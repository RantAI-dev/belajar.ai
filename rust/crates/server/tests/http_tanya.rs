//! The worker's own HTTP tests for tanya: query validation, page cursors on messages, paths
//! that are not even the right shape, and sources without a page image. The lead's acceptance
//! file covers the main flow; these cover the edges around it.

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]
#![allow(clippy::panic, reason = "tests")]

mod support;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::http::StatusCode;
use belajar_core::grade::Grade;
use belajar_fake_upstream::{FakeConfig, FakeServer};
use belajar_ports::SystemClock;
use belajar_ports::upstream::{
    Assistant, Chapter, ChatEvent, ChatOutcome, ChatRequest, ChatStream, Mapel, PageImage,
    PracticeAnswer, PracticeRequest, PracticeSet, PracticeStream, QuestionScore, Source, Upstream,
    UpstreamError,
};
use belajar_upstream::{HttpUpstream, UpstreamConfig, UpstreamKeys};
use futures::stream;
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{TestApp, TestResponse};

const KEYS: &str = "7.ask=fake-7-ask,7.learn=fake-7-learn,7.practice=fake-7-practice";

/// One canned answer, with one source that has no page image and one that does. What the real
/// upstream never sends in one stream, this does, so the server's edge paths have a test.
struct CannedUpstream {
    page: Option<PageImage>,
}

impl CannedUpstream {
    fn canned_sources() -> Vec<Source> {
        vec![
            Source {
                number: 1,
                title: "IPA Kelas VII".into(),
                section: Some("Bab 5".into()),
                document_id: "kb-ipa-7".into(),
                page: None,
                cited: true,
                page_url: None,
            },
            Source {
                number: 2,
                title: "IPA Kelas VII".into(),
                section: None,
                document_id: "kb-ipa-7".into(),
                page: Some(81),
                cited: false,
                page_url: Some("http://127.0.0.1:9/pages/kb-ipa-7/81.svg".into()),
            },
        ]
    }
}

#[async_trait]
impl Upstream for CannedUpstream {
    async fn mapel(&self, _grade: Grade) -> Result<Vec<Mapel>, UpstreamError> {
        Err(UpstreamError::NotConfigured {
            grade: 7,
            assistant: Assistant::Ask,
        })
    }

    async fn chapters(&self, _grade: Grade, _mapel: &str) -> Result<Vec<Chapter>, UpstreamError> {
        Err(UpstreamError::NotConfigured {
            grade: 7,
            assistant: Assistant::Ask,
        })
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatStream, UpstreamError> {
        let events = vec![
            Ok(ChatEvent::Delta("Jawaban ".into())),
            Ok(ChatEvent::Delta("lengkap.".into())),
            Ok(ChatEvent::Done(ChatOutcome {
                sources: Self::canned_sources(),
                grounded: true,
                retrieval_score: None,
            })),
        ];
        Ok(Box::pin(stream::iter(events)))
    }

    async fn generate_practice(
        &self,
        _request: PracticeRequest,
    ) -> Result<PracticeStream, UpstreamError> {
        Err(UpstreamError::NotConfigured {
            grade: 7,
            assistant: Assistant::Practice,
        })
    }

    async fn grade_practice(
        &self,
        _grade: Grade,
        _set: &PracticeSet,
        _answers: &[PracticeAnswer],
    ) -> Result<Vec<QuestionScore>, UpstreamError> {
        Err(UpstreamError::NotConfigured {
            grade: 7,
            assistant: Assistant::Practice,
        })
    }

    async fn page(&self, _page_url: &str) -> Result<PageImage, UpstreamError> {
        match &self.page {
            Some(image) => Ok(PageImage {
                content_type: image.content_type.clone(),
                bytes: image.bytes.clone(),
            }),
            None => Err(UpstreamError::Unavailable("no such page".into())),
        }
    }
}

/// The `done` event of an SSE body, parsed.
fn done_event(res: &TestResponse) -> Value {
    res.text
        .split("\n\n")
        .find_map(|block| {
            let is_done = block.lines().any(|line| line == "event: done");
            let data = block.lines().find_map(|line| line.strip_prefix("data: "));
            is_done.then_some(data).flatten()
        })
        .and_then(|data| serde_json::from_str(data).ok())
        .expect("one done event with JSON data")
}

async fn app(pool: PgPool) -> TestApp {
    TestApp::with_parts(
        pool,
        Arc::new(CannedUpstream {
            page: Some(PageImage {
                content_type: "image/png".into(),
                bytes: vec![0x89, b'P', b'N', b'G'].into(),
            }),
        }),
        Arc::new(SystemClock),
    )
    .await
}

async fn canned_conversation(app: &TestApp, cookie: &str) -> (String, Value) {
    let res = app
        .post(
            "/v1/conversations",
            json!({ "subjectId": "ipa" }),
            Some(cookie),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text);
    let id = res.body["id"].as_str().unwrap().to_owned();
    let res = app
        .post(
            &format!("/v1/conversations/{id}/messages"),
            json!({ "content": "Apa itu pemuaian?" }),
            Some(cookie),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    (id, done_event(&res))
}

fn without_request_id(mut body: Value) -> Value {
    body["error"]["requestId"] = Value::Null;
    body
}

#[sqlx::test(migrations = "../../migrations")]
async fn bad_list_queries_name_their_fields(pool: PgPool) {
    let app = app(pool).await;
    let budi = app.student("budi").await;
    for (path, field) in [
        ("/v1/conversations?limit=0", "limit"),
        ("/v1/conversations?limit=abc", "limit"),
        ("/v1/conversations?mode=gabut", "mode"),
        ("/v1/conversations?cursor=falsch", "cursor"),
    ] {
        let res = app.get(path, Some(&budi)).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{path}");
        assert_eq!(res.body["error"]["code"], "invalid_request", "{path}");
        let fields: Vec<&str> = res.body["error"]["errors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["field"].as_str().unwrap())
            .collect();
        assert!(fields.contains(&field), "{path} names {field}");
    }
    // A larger limit is lowered, not refused.
    let res = app.get("/v1/conversations?limit=999", Some(&budi)).await;
    assert_eq!(res.status, StatusCode::OK);
    // A learn filter on a slice that only creates ask conversations: a fine, empty page.
    let res = app.get("/v1/conversations?mode=learn", Some(&budi)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["data"], json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn paths_that_are_not_even_the_right_shape_answer_as_missing(pool: PgPool) {
    let app = app(pool).await;
    let budi = app.student("budi").await;
    let missing = without_request_id(
        app.get("/v1/conversations/not-a-uuid", Some(&budi))
            .await
            .body,
    );
    assert_eq!(missing["error"]["code"], "not_found");
    for res in [
        app.get("/v1/conversations/not-a-uuid/messages", Some(&budi))
            .await,
        app.post(
            "/v1/conversations/not-a-uuid/messages",
            json!({ "content": "halo" }),
            Some(&budi),
        )
        .await,
        app.get("/v1/messages/not-a-uuid/sources/1/page", Some(&budi))
            .await,
        app.get(
            "/v1/messages/0193b2c4-7a1e-7c3f-9d2a-5e8f1b6c4d90/sources/x/page",
            Some(&budi),
        )
        .await,
        app.patch(
            "/v1/conversations/not-a-uuid",
            json!({ "title": "Judul" }),
            Some(&budi),
        )
        .await,
    ] {
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{}", res.text);
        assert_eq!(without_request_id(res.body), missing);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_source_without_a_page_image_has_no_preview_and_no_page_route_answer(pool: PgPool) {
    let app = app(pool).await;
    let budi = app.student("budi").await;
    let (_id, answer) = canned_conversation(&app, &budi).await;

    let sources = answer["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2);
    assert!(
        sources[0]["pagePreviewUrl"].is_null(),
        "no page_url, no preview"
    );
    assert_eq!(
        sources[1]["pagePreviewUrl"].as_str().unwrap(),
        format!(
            "/v1/messages/{}/sources/2/page",
            answer["id"].as_str().unwrap()
        )
    );

    // The owner reaches the one page that exists, and gets the upstream's content type and a
    // private cache header.
    let existing = app
        .get(sources[1]["pagePreviewUrl"].as_str().unwrap(), Some(&budi))
        .await;
    assert_eq!(existing.status, StatusCode::OK);
    assert!(
        existing
            .header("content-type")
            .unwrap()
            .starts_with("image/")
    );
    assert_eq!(
        existing.header("cache-control").as_deref(),
        Some("private, max-age=3600")
    );

    // An unknown number and a source without a page answer exactly like a missing message.
    let missing = without_request_id(
        app.get("/v1/messages/not-a-uuid/sources/1/page", Some(&budi))
            .await
            .body,
    );
    let message_id = answer["id"].as_str().unwrap();
    for res in [
        app.get(
            &format!("/v1/messages/{message_id}/sources/1/page"),
            Some(&budi),
        )
        .await,
        app.get(
            &format!("/v1/messages/{message_id}/sources/9/page"),
            Some(&budi),
        )
        .await,
    ] {
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{}", res.text);
        assert_eq!(without_request_id(res.body), missing);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn another_student_sees_the_same_page_preview_as_a_missing_one(pool: PgPool) {
    let app = app(pool).await;
    let budi = app.student("budi").await;
    let siti = app.student("siti").await;
    let (_id, answer) = canned_conversation(&app, &budi).await;
    let preview = answer["sources"][1]["pagePreviewUrl"]
        .as_str()
        .unwrap()
        .to_owned();

    assert_eq!(
        app.get(&preview, Some(&budi)).await.status,
        StatusCode::OK,
        "owner positive control"
    );
    let missing = without_request_id(
        app.get("/v1/messages/not-a-uuid/sources/1/page", Some(&siti))
            .await
            .body,
    );
    assert_eq!(missing["error"]["code"], "not_found");
    let res = app.get(&preview, Some(&siti)).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(without_request_id(res.body), missing);
}

#[sqlx::test(migrations = "../../migrations")]
async fn message_pages_continue_to_newer_messages_with_the_same_limit_rules(pool: PgPool) {
    let app = app(pool).await;
    let budi = app.student("budi").await;
    let (id, _answer) = canned_conversation(&app, &budi).await;
    // Three questions, six stored messages.
    for question in [
        "Pertanyaan kedua tentang suhu",
        "Pertanyaan ketiga tentang kalor",
    ] {
        let res = app
            .post(
                &format!("/v1/conversations/{id}/messages"),
                json!({ "content": question }),
                Some(&budi),
            )
            .await;
        assert_eq!(res.status, StatusCode::OK);
    }
    let messages = format!("/v1/conversations/{id}/messages");

    let page1 = app.get(&format!("{messages}?limit=4"), Some(&budi)).await;
    assert_eq!(page1.status, StatusCode::OK);
    let data = page1.body["data"].as_array().unwrap();
    assert_eq!(data.len(), 4);
    assert_eq!(data[0]["role"], "user");
    assert!(data[0]["content"].as_str().unwrap().contains("pemuaian"));
    assert!(data[2]["content"].as_str().unwrap().contains("kedua"));
    assert_eq!(data[3]["role"], "assistant");
    let cursor = page1.body["nextCursor"].as_str().expect("a second page");

    let page2 = app
        .get(&format!("{messages}?limit=4&cursor={cursor}"), Some(&budi))
        .await;
    let data = page2.body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2, "the newer tail, in order");
    assert!(data[0]["content"].as_str().unwrap().contains("ketiga"));
    assert!(page2.body["nextCursor"].is_null());

    for bad in ["limit=0", "limit=-3", "limit=abc"] {
        let res = app.get(&format!("{messages}?{bad}"), Some(&budi)).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(res.body["error"]["errors"][0]["field"], "limit");
    }
    let res = app.get(&format!("{messages}?limit=200"), Some(&budi)).await;
    assert_eq!(res.status, StatusCode::OK, "a larger limit is lowered");
    let res = app
        .get(&format!("{messages}?cursor=falsch"), Some(&budi))
        .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"]["errors"][0]["field"], "cursor");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_assistant_message_is_exactly_the_concatenated_deltas(pool: PgPool) {
    let app = app(pool).await;
    let budi = app.student("budi").await;
    let (id, answer) = canned_conversation(&app, &budi).await;
    assert_eq!(answer["role"], "assistant");
    assert_eq!(answer["content"], "Jawaban lengkap.");
    assert_eq!(answer["isGrounded"], true);

    let history = app
        .get(&format!("/v1/conversations/{id}/messages"), Some(&budi))
        .await;
    let data = history.body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[1]["id"], answer["id"]);
    assert_eq!(data[1]["content"], "Jawaban lengkap.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_request_field_is_rejected_before_the_store(pool: PgPool) {
    let app = app(pool).await;
    let budi = app.student("budi").await;
    let res = app
        .post(
            "/v1/conversations",
            json!({ "subjectId": "ipa", "title": "siap" }),
            Some(&budi),
        )
        .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"]["code"], "invalid_request");
    let res = app
        .post(
            "/v1/conversations",
            json!({ "subjectId": "mtk", "extra": 1 }),
            Some(&budi),
        )
        .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_fake_upstream_end_to_end_still_passes_the_worker_paths(pool: PgPool) {
    // The canned upstream pins the shape; one round against the in-repo fake pins the plumbing:
    // a question through the real SSE client, with sources and a real page image.
    let fake: FakeServer = belajar_fake_upstream::spawn(FakeConfig::default())
        .await
        .unwrap();
    let upstream = HttpUpstream::new(UpstreamConfig {
        base_url: fake.base_url.clone(),
        keys: UpstreamKeys::parse(KEYS).unwrap(),
        timeout: Duration::from_secs(10),
    })
    .unwrap();
    let app = TestApp::with_parts(pool, Arc::new(upstream), Arc::new(SystemClock)).await;
    let budi = app.student("budi").await;
    let res = app
        .post(
            "/v1/conversations",
            json!({ "subjectId": "ipa" }),
            Some(&budi),
        )
        .await;
    let id = res.body["id"].as_str().unwrap().to_owned();
    let res = app
        .post(
            &format!("/v1/conversations/{id}/messages"),
            json!({ "content": "Kenapa rel kereta api diberi celah?" }),
            Some(&budi),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    let answer = done_event(&res);
    let preview = answer["sources"][0]["pagePreviewUrl"]
        .as_str()
        .unwrap()
        .to_owned();
    let page = app.get(&preview, Some(&budi)).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.header("content-type").unwrap().starts_with("image/"));
    assert!(!page.text.is_empty());
    assert!(
        !res.text.contains("page_url"),
        "the stored page_url never crosses the wire"
    );
    assert!(!res.text.contains(&fake.base_url));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_book_kept_as_a_whole_pdf_previews_the_file_and_names_the_cited_page(pool: PgPool) {
    let fake: FakeServer = belajar_fake_upstream::spawn(FakeConfig::default())
        .await
        .unwrap();
    let upstream = HttpUpstream::new(UpstreamConfig {
        base_url: fake.base_url.clone(),
        keys: UpstreamKeys::parse(KEYS).unwrap(),
        timeout: Duration::from_secs(10),
    })
    .unwrap();
    let app = TestApp::with_parts(pool, Arc::new(upstream), Arc::new(SystemClock)).await;
    let budi = app.student("budi").await;
    let res = app
        .post(
            "/v1/conversations",
            json!({ "subjectId": "mtk" }),
            Some(&budi),
        )
        .await;
    let id = res.body["id"].as_str().unwrap().to_owned();
    let res = app
        .post(
            &format!("/v1/conversations/{id}/messages"),
            json!({ "content": "Bagaimana cara menjumlahkan pecahan?" }),
            Some(&budi),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK);
    let source = &done_event(&res)["sources"][0];
    assert_eq!(source["page"], 44);
    assert_eq!(source["previewPage"], 44, "the page inside the whole book");
    let pdf = app
        .get(source["pagePreviewUrl"].as_str().unwrap(), Some(&budi))
        .await;
    assert_eq!(pdf.status, StatusCode::OK);
    assert_eq!(
        pdf.header("content-type").as_deref(),
        Some("application/pdf")
    );
    assert!(pdf.text.starts_with("%PDF-"));
}
