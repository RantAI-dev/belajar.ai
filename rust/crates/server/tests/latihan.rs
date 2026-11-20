//! The worker's own HTTP coverage for Latihan, beside the lead's acceptance tests: question
//! kinds, the page behind a submitted answer, list limits and cursors, topic mastery, and
//! isolation of the page endpoint.

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]
#![allow(clippy::panic, reason = "tests")]

mod support;

use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use belajar_fake_upstream::{FakeConfig, FakeServer};
use belajar_ports::SystemClock;
use belajar_upstream::{HttpUpstream, UpstreamConfig, UpstreamKeys};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{TestApp, TestResponse};

const KEYS: &str = "7.ask=fake-7-ask,7.learn=fake-7-learn,7.practice=fake-7-practice";

async fn world(pool: PgPool) -> (TestApp, FakeServer) {
    let fake = belajar_fake_upstream::spawn(FakeConfig::default())
        .await
        .unwrap();
    let upstream = HttpUpstream::new(UpstreamConfig {
        base_url: fake.base_url.clone(),
        keys: UpstreamKeys::parse(KEYS).unwrap(),
        timeout: Duration::from_secs(10),
    })
    .unwrap();
    let app = TestApp::with_parts(pool, Arc::new(upstream), Arc::new(SystemClock)).await;
    (app, fake)
}

fn request(subject: &str, topic: &str, practice_type: &str, total: u64) -> Value {
    json!({
        "subjectId": subject,
        "topic": topic,
        "practiceType": practice_type,
        "totalQuestions": total,
    })
}

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

/// Generates a set and returns the `practiceSet` of the `ready` event.
async fn generate(
    app: &TestApp,
    cookie: &str,
    subject: &str,
    topic: &str,
    practice_type: &str,
    total: u64,
) -> Value {
    let res = app
        .post(
            "/v1/practice-sets",
            request(subject, topic, practice_type, total),
            Some(cookie),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    let all = events(&res);
    let (name, data) = all.last().unwrap();
    assert_eq!(name, "ready", "{}", res.text);
    data["practiceSet"].clone()
}

/// The answer key, read from the database the way only the server can.
async fn key(pool: &PgPool, set_id: &str) -> Vec<(String, Vec<String>)> {
    let practice: Value =
        sqlx::query_scalar("SELECT practice FROM practice_sets WHERE id = $1::uuid")
            .bind(set_id)
            .fetch_one(pool)
            .await
            .unwrap();
    practice["questions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| {
            let answer = q["answer"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_str().unwrap().to_owned());
            (q["id"].as_str().unwrap().to_owned(), answer.collect())
        })
        .collect()
}

/// One answer per question, all correct except the first when `wrong_first`.
async fn answers(pool: &PgPool, set: &Value, wrong_first: bool) -> Vec<Value> {
    let id = set["id"].as_str().unwrap().to_owned();
    let key = key(pool, &id).await;
    set["questions"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, question)| {
            let qid = question["id"].as_str().unwrap();
            let right = &key.iter().find(|(k, _)| k == qid).unwrap().1;
            let selected = if wrong_first && index == 0 {
                vec![
                    question["options"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|o| o["label"].as_str().unwrap().to_owned())
                        .find(|l| !right.contains(l))
                        .unwrap(),
                ]
            } else {
                right.clone()
            };
            json!({ "questionId": qid, "selected": selected })
        })
        .collect()
}

async fn submit(
    app: &TestApp,
    pool: &PgPool,
    cookie: &str,
    set: &Value,
    wrong_first: bool,
) -> TestResponse {
    let id = set["id"].as_str().unwrap().to_owned();
    let body = json!({ "answers": answers(pool, set, wrong_first).await });
    app.post(
        &format!("/v1/practice-sets/{id}/submit"),
        body,
        Some(cookie),
    )
    .await
}

/// The real practice API cites no page per question (see `fake-upstream/fixtures/practice/`), so
/// the fake does not invent one: a test of the page behind a reviewed question gives the stored
/// question a source the way a citing upstream would have, pointing at the fake's own page.
async fn cite_first_question(pool: &PgPool, fake: &FakeServer, set_id: &str) {
    let source = json!({
        "number": 1,
        "title": "IPA Buku Siswa Kelas VII",
        "section": "Bab 5: Suhu, Kalor, dan Pemuaian",
        "document_id": "kb-ipa-7",
        "page": 70,
        "cited": true,
        "page_url": format!("{}/pages/kb-ipa-7/70.svg?sig=t", fake.base_url),
    });
    sqlx::query(
        "UPDATE practice_sets SET practice = jsonb_set(practice, '{questions,0,source}', $1) \
         WHERE id = $2::uuid",
    )
    .bind(source)
    .bind(set_id)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_mixed_set_carries_every_kind_and_stories_have_their_text(pool: PgPool) {
    let (app, _fake) = world(pool).await;
    let budi = app.student("budi").await;
    let set = generate(&app, &budi, "ipa", "Pemuaian", "mcq_mixed", 10).await;
    let questions = set["questions"].as_array().unwrap();
    assert_eq!(questions.len(), 10);
    let kinds: Vec<&str> = questions
        .iter()
        .map(|q| q["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"single"));
    assert!(kinds.contains(&"multiple"));
    assert!(kinds.contains(&"story"));
    for (index, question) in questions.iter().enumerate() {
        assert_eq!(question["number"], (index + 1) as u64);
        let story = match question["kind"].as_str().unwrap() {
            "story" => question["story"]
                .as_str()
                .expect("a story question has its story"),
            _ => continue,
        };
        assert!(!story.is_empty());
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_set_delivered_short_counts_the_questions_it_holds(pool: PgPool) {
    // The real API may deliver fewer questions than asked for (recorded: 2 of 5); the fake's
    // "__short__" topic does the same.
    let (app, _fake) = world(pool.clone()).await;
    let budi = app.student("budi").await;
    let set = generate(&app, &budi, "ipa", "__short__", "mcq_multiple_correct", 5).await;
    let held = set["questions"].as_array().unwrap().len() as u64;
    assert_eq!(held, 2);
    assert_eq!(set["totalQuestions"], held, "the set counts what it holds");

    let res = submit(&app, &pool, &budi, &set, false).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    assert_eq!(res.body["totalCount"], held);
    let list = app.get("/v1/practice-sets?limit=10", Some(&budi)).await;
    assert_eq!(
        list.body["data"][0]["totalQuestions"], held,
        "{}",
        list.text
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_page_appears_only_after_submit(pool: PgPool) {
    let (app, fake) = world(pool.clone()).await;
    let budi = app.student("budi").await;
    let set = generate(&app, &budi, "ipa", "Pemuaian", "mcq_normal", 5).await;
    let id = set["id"].as_str().unwrap();
    cite_first_question(&pool, &fake, id).await;
    let qid = set["questions"][0]["id"].as_str().unwrap();
    let path = format!("/v1/practice-sets/{id}/questions/{qid}/page");

    // Before submit: the page is part of the review, so it does not exist yet.
    assert_eq!(
        app.get(&path, Some(&budi)).await.status,
        StatusCode::NOT_FOUND
    );

    let res = submit(&app, &pool, &budi, &set, false).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    let item = &res.body["items"][0];
    assert_eq!(item["questionId"], qid);
    assert_eq!(
        item["source"]["pagePreviewUrl"].as_str().unwrap(),
        path,
        "the review points at our own endpoint, never at the upstream"
    );

    // After submit: the page is fetched server-side and served as the image it is.
    let page = app.get(&path, Some(&budi)).await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.text);
    assert_eq!(
        page.header("content-type").as_deref(),
        Some("image/svg+xml")
    );
    assert!(page.text.contains("Halaman"), "{}", page.text);
    assert!(
        !page.text.contains("sig="),
        "no upstream URL leaks: {}",
        page.text
    );

    // A question the set does not have is the same 404.
    let missing = format!("/v1/practice-sets/{id}/questions/nope/page");
    assert_eq!(
        app.get(&missing, Some(&budi)).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_page_of_another_students_set_is_not_found(pool: PgPool) {
    let (app, fake) = world(pool.clone()).await;
    let budi = app.student("budi").await;
    let siti = app.student("siti").await;
    let set = generate(&app, &budi, "ipa", "Pemuaian", "mcq_normal", 5).await;
    let id = set["id"].as_str().unwrap();
    cite_first_question(&pool, &fake, id).await;
    let qid = set["questions"][0]["id"].as_str().unwrap();
    let path = format!("/v1/practice-sets/{id}/questions/{qid}/page");
    let missing_set = "/v1/practice-sets/0193b2c4-7a1e-7c3f-9d2a-5e8f1b6c4d90/questions/q1/page";

    assert_eq!(
        submit(&app, &pool, &budi, &set, false).await.status,
        StatusCode::OK
    );

    // Positive control: the owner reads it.
    assert_eq!(app.get(&path, Some(&budi)).await.status, StatusCode::OK);

    let mut missing = app.get(missing_set, Some(&siti)).await.body;
    missing["error"]["requestId"] = Value::Null;
    assert_eq!(missing["error"]["code"], "not_found");
    let res = app.get(&path, Some(&siti)).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND, "{}", res.text);
    let mut body = res.body;
    body["error"]["requestId"] = Value::Null;
    assert_eq!(body, missing, "another student's page is a plain 404");
}

#[sqlx::test(migrations = "../../migrations")]
async fn limits_are_validated_and_pages_follow_the_cursor(pool: PgPool) {
    let (app, _fake) = world(pool).await;
    let budi = app.student("budi").await;
    let sets: Vec<Value> = vec![
        generate(&app, &budi, "ipa", "Pemuaian", "mcq_normal", 5).await,
        generate(&app, &budi, "mtk", "Pecahan", "mcq_normal", 5).await,
        generate(&app, &budi, "ipa", "Suhu", "mcq_story", 5).await,
    ];

    for bad in ["0", "-3", "banyak"] {
        let res = app
            .get(&format!("/v1/practice-sets?limit={bad}"), Some(&budi))
            .await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{bad}: {}", res.text);
        assert_eq!(res.body["error"]["code"], "invalid_request");
        let field = res.body["error"]["errors"][0]["field"].as_str().unwrap();
        assert_eq!(field, "limit", "{bad} names its field");
    }
    let garbage = app
        .get("/v1/practice-sets?cursor=not-a-cursor", Some(&budi))
        .await;
    assert_eq!(garbage.status, StatusCode::BAD_REQUEST);
    assert_eq!(garbage.body["error"]["errors"][0]["field"], "cursor");

    // A limit beyond the maximum is lowered, not rejected.
    let big = app.get("/v1/practice-sets?limit=999", Some(&budi)).await;
    assert_eq!(big.status, StatusCode::OK);
    assert_eq!(big.body["data"].as_array().unwrap().len(), 3);

    let page1 = app.get("/v1/practice-sets?limit=2", Some(&budi)).await;
    assert_eq!(page1.status, StatusCode::OK);
    assert_eq!(page1.body["data"][0]["id"], sets[2]["id"], "newest first");
    assert_eq!(page1.body["data"][1]["id"], sets[1]["id"]);
    let cursor = page1.body["nextCursor"].as_str().unwrap();
    let page2 = app
        .get(
            &format!("/v1/practice-sets?limit=2&cursor={cursor}"),
            Some(&budi),
        )
        .await;
    assert_eq!(page2.body["data"][0]["id"], sets[0]["id"]);
    assert_eq!(page2.body["nextCursor"], Value::Null, "the last page ends");

    // The cursor is opaque: it names no table, column or timestamp.
    assert!(!cursor.contains(|c: char| c.is_whitespace()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn topic_mastery_aggregates_by_subject_and_filters(pool: PgPool) {
    let (app, _fake) = world(pool.clone()).await;
    let budi = app.student("budi").await;
    let ipa = generate(&app, &budi, "ipa", "Pemuaian", "mcq_normal", 5).await;
    let mtk = generate(&app, &budi, "mtk", "Pecahan", "mcq_normal", 5).await;

    // One wrong answer in the IPA set, a perfect run in Matematika.
    let ipa_res = submit(&app, &pool, &budi, &ipa, true).await;
    assert_eq!(ipa_res.status, StatusCode::OK, "{}", ipa_res.text);
    assert_eq!(ipa_res.body["correctCount"], 4);
    let mtk_res = submit(&app, &pool, &budi, &mtk, false).await;
    assert_eq!(mtk_res.status, StatusCode::OK, "{}", mtk_res.text);
    assert_eq!(mtk_res.body["starsEarned"], 25, "5 correct, 5 stars each");

    let all = app.get("/v1/topic-mastery", Some(&budi)).await;
    assert_eq!(all.status, StatusCode::OK);
    let data = all.body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2, "one row per subject and topic");
    assert_eq!(data[0]["subjectId"], "ipa", "sorted by subject id");
    assert_eq!(data[0]["topic"], "Pemuaian");
    assert_eq!(data[0]["attempts"], 5);
    assert_eq!(data[0]["correct"], 4);
    assert_eq!(data[0]["masteryPercent"], 80);
    assert_eq!(data[1]["subjectId"], "mtk");
    assert_eq!(data[1]["masteryPercent"], 100);
    // The level is core's one rule: 80% over 5 answers is already "mastered".
    assert_eq!(data[0]["level"], "mastered");
    assert_eq!(data[1]["level"], "mastered");
    assert_eq!(all.body["nextCursor"], Value::Null);

    let filtered = app
        .get("/v1/topic-mastery?subjectId=mtk", Some(&budi))
        .await;
    let data = filtered.body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["subjectId"], "mtk");

    let bad = app
        .get("/v1/topic-mastery?subjectId=kimia", Some(&budi))
        .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(bad.body["error"]["errors"][0]["field"], "subjectId");

    // Another student's mastery is theirs alone.
    let siti = app.student("siti").await;
    let empty = app.get("/v1/topic-mastery", Some(&siti)).await;
    assert_eq!(empty.body["data"], json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn unauthenticated_calls_are_rejected(pool: PgPool) {
    let (app, _fake) = world(pool).await;
    let res = app
        .post(
            "/v1/practice-sets",
            request("ipa", "Pemuaian", "mcq_normal", 5),
            None,
        )
        .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"]["code"], "unauthenticated");
    for path in ["/v1/practice-sets", "/v1/topic-mastery"] {
        let res = app.get(path, None).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{path}");
    }
}
