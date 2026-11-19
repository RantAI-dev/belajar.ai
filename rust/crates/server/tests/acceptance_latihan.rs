//! Acceptance tests for Latihan: generation with live progress, answer keys kept server-side until
//! submit, grading, stars and topic mastery. Written by the lead before the slice started; the
//! worker must make them pass without editing this file.
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
const MISSING: &str = "/v1/practice-sets/0193b2c4-7a1e-7c3f-9d2a-5e8f1b6c4d90";

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

fn request(practice_type: &str, total: u64) -> Value {
    json!({
        "subjectId": "ipa",
        "topic": "Pemuaian",
        "practiceType": practice_type,
        "totalQuestions": total,
    })
}

/// Generates a set and returns the `practiceSet` of the `ready` event.
async fn generate(app: &TestApp, cookie: &str, practice_type: &str, total: u64) -> Value {
    let res = app
        .post(
            "/v1/practice-sets",
            request(practice_type, total),
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

fn assert_no_answer_material(value: &Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                assert!(
                    !matches!(
                        k.as_str(),
                        "answer" | "correctAnswer" | "explanation" | "isCorrect" | "correct"
                    ),
                    "answer material `{k}` was sent before submit"
                );
                assert_no_answer_material(v);
            }
        }
        Value::Array(items) => items.iter().for_each(assert_no_answer_material),
        _ => {}
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn generation_streams_progress_then_a_set_without_answers(pool: PgPool) {
    let (app, _fake) = world(pool).await;
    let budi = app.student("budi").await;
    let res = app
        .post("/v1/practice-sets", request("mcq_normal", 5), Some(&budi))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(
        res.headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/event-stream")
    );
    let all = events(&res);

    let progress: Vec<&Value> = all
        .iter()
        .filter(|(n, _)| n == "progress")
        .map(|(_, d)| d)
        .collect();
    assert!(progress.len() >= 3);
    for p in &progress {
        assert!(
            ["retrieval", "generating", "checking", "labelling"]
                .contains(&p["stage"].as_str().unwrap())
        );
        assert_eq!(p["total"], 5);
        assert!(p["delivered"].as_u64().unwrap() <= 5);
        assert!(p["elapsedMs"].is_u64());
    }
    assert_eq!(all.last().unwrap().0, "ready");
    assert_no_answer_material(&Value::Array(all.iter().map(|(_, d)| d.clone()).collect()));

    let set = &all.last().unwrap().1["practiceSet"];
    assert_eq!(set["status"], "ready");
    assert_eq!(set["subjectId"], "ipa");
    assert_eq!(set["topic"], "Pemuaian");
    assert_eq!(set["practiceType"], "mcq_normal");
    assert_eq!(set["totalQuestions"], 5);
    assert!(set["result"].is_null());
    let questions = set["questions"].as_array().unwrap();
    assert_eq!(questions.len(), 5);
    for (i, q) in questions.iter().enumerate() {
        assert_eq!(q["number"], i as u64 + 1);
        assert_eq!(q["kind"], "single");
        assert!(q["stem"].is_string() && q["id"].is_string());
        assert_eq!(q["options"].as_array().unwrap().len(), 4);
        assert!(q["options"][0]["label"].is_string() && q["options"][0]["text"].is_string());
    }

    let id = set["id"].as_str().unwrap();
    let got = app
        .get(&format!("/v1/practice-sets/{id}"), Some(&budi))
        .await;
    assert_eq!(got.status, StatusCode::OK);
    assert_eq!(got.body["questions"], set["questions"]);
    assert_no_answer_material(&got.body);
}

#[sqlx::test(migrations = "../../migrations")]
async fn generation_is_validated_before_calling_the_upstream(pool: PgPool) {
    let (app, _fake) = world(pool).await;
    let budi = app.student("budi").await;
    let body =
        json!({ "subjectId": "kimia", "topic": "", "practiceType": "essay", "totalQuestions": 7 });
    let res = app.post("/v1/practice-sets", body, Some(&budi)).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    let mut fields: Vec<&str> = res.body["error"]["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap())
        .collect();
    fields.sort_unstable();
    assert_eq!(
        fields,
        vec!["/practiceType", "/subjectId", "/topic", "/totalQuestions"]
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_upstream_error_during_generation_ends_the_stream_with_an_error_event(pool: PgPool) {
    let (app, _fake) = world(pool.clone()).await;
    let budi = app.student("budi").await;
    let mut body = request("mcq_normal", 5);
    body["topic"] = json!("__error__");
    let res = app.post("/v1/practice-sets", body, Some(&budi)).await;
    assert_eq!(res.status, StatusCode::OK);
    let all = events(&res);
    let (name, data) = all.last().unwrap();
    assert_eq!(name, "error");
    assert_eq!(data["error"]["code"], "upstream_failed");
    assert!(data["error"]["requestId"].is_string());
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM practice_sets")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "no set is stored when generation fails");
}

#[sqlx::test(migrations = "../../migrations")]
async fn submitting_grades_reveals_the_review_and_awards_stars(pool: PgPool) {
    let (app, _fake) = world(pool.clone()).await;
    let budi = app.student("budi").await;
    let set = generate(&app, &budi, "mcq_normal", 5).await;
    let id = set["id"].as_str().unwrap().to_owned();
    let key = key(&pool, &id).await;

    // Four right, the first one wrong.
    let answers: Vec<Value> = set["questions"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, q)| {
            let qid = q["id"].as_str().unwrap();
            let right = &key.iter().find(|(k, _)| k == qid).unwrap().1;
            let selected = if i == 0 {
                let wrong = q["options"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|o| o["label"].as_str().unwrap().to_owned())
                    .find(|l| !right.contains(l))
                    .unwrap();
                vec![wrong]
            } else {
                right.clone()
            };
            json!({ "questionId": qid, "selected": selected })
        })
        .collect();

    let res = app
        .post(
            &format!("/v1/practice-sets/{id}/submit"),
            json!({ "answers": answers }),
            Some(&budi),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    assert_eq!(res.body["correctCount"], 4);
    assert_eq!(res.body["totalCount"], 5);
    assert_eq!(res.body["starsEarned"], 20, "5 stars per correct answer");
    let items = res.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 5);
    assert_eq!(items[0]["isCorrect"], false);
    assert!(items[1..].iter().all(|i| i["isCorrect"] == true));
    for item in items {
        let qid = item["questionId"].as_str().unwrap();
        let right = &key.iter().find(|(k, _)| k == qid).unwrap().1;
        assert_eq!(item["correctAnswer"], json!(right));
        assert!(item["explanation"].as_str().unwrap().len() > 0);
        assert!(item["selected"].is_array());
    }

    let got = app
        .get(&format!("/v1/practice-sets/{id}"), Some(&budi))
        .await;
    assert_eq!(got.body["status"], "submitted");
    assert_eq!(got.body["result"]["correctCount"], 4);

    let stars: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(amount), 0)::bigint FROM star_ledger")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stars, 20);

    let mastery = app
        .get("/v1/topic-mastery?subjectId=ipa", Some(&budi))
        .await;
    assert_eq!(mastery.status, StatusCode::OK);
    let row = &mastery.body["data"][0];
    assert_eq!(row["topic"], "Pemuaian");
    assert_eq!(row["attempts"], 5);
    assert_eq!(row["correct"], 4);
    assert_eq!(row["masteryPercent"], 80);

    let again = app
        .post(
            &format!("/v1/practice-sets/{id}/submit"),
            json!({ "answers": answers }),
            Some(&budi),
        )
        .await;
    assert_eq!(again.status, StatusCode::CONFLICT, "a set is graded once");
    assert_eq!(again.body["error"]["code"], "conflict");
    let stars: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(amount), 0)::bigint FROM star_ledger")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stars, 20, "no second award");
}

#[sqlx::test(migrations = "../../migrations")]
async fn multiple_correct_needs_every_right_option(pool: PgPool) {
    let (app, _fake) = world(pool.clone()).await;
    let budi = app.student("budi").await;
    let set = generate(&app, &budi, "mcq_multiple_correct", 5).await;
    let id = set["id"].as_str().unwrap().to_owned();
    let key = key(&pool, &id).await;
    let (multi_id, multi_key) = key.iter().find(|(_, a)| a.len() >= 2).unwrap().clone();
    let answers: Vec<Value> = key
        .iter()
        .map(|(qid, right)| {
            // For one multi-answer question, pick only one of its right options.
            let selected = if *qid == multi_id {
                vec![multi_key[0].clone()]
            } else {
                right.clone()
            };
            json!({ "questionId": qid, "selected": selected })
        })
        .collect();
    let res = app
        .post(
            &format!("/v1/practice-sets/{id}/submit"),
            json!({ "answers": answers }),
            Some(&budi),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    assert_eq!(res.body["correctCount"], 4);
    let item = res.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["questionId"] == multi_id.as_str())
        .unwrap();
    assert_eq!(item["isCorrect"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_submission_must_answer_each_question_once_with_real_options(pool: PgPool) {
    let (app, _fake) = world(pool.clone()).await;
    let budi = app.student("budi").await;
    let set = generate(&app, &budi, "mcq_normal", 5).await;
    let id = set["id"].as_str().unwrap().to_owned();
    let qids: Vec<String> = set["questions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| q["id"].as_str().unwrap().to_owned())
        .collect();
    let all_a = |ids: &[String]| -> Vec<Value> {
        ids.iter()
            .map(|q| json!({ "questionId": q, "selected": ["A"] }))
            .collect()
    };
    let mut duplicated = all_a(&qids);
    duplicated[4] = json!({ "questionId": qids[0], "selected": ["A"] });
    let mut unknown_label = all_a(&qids);
    unknown_label[0]["selected"] = json!(["Z"]);
    let mut two_on_single = all_a(&qids);
    two_on_single[0]["selected"] = json!(["A", "B"]);
    for answers in [all_a(&qids[..4]), duplicated, unknown_label, two_on_single] {
        let res = app
            .post(
                &format!("/v1/practice-sets/{id}/submit"),
                json!({ "answers": answers }),
                Some(&budi),
            )
            .await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{}", res.text);
        assert_eq!(res.body["error"]["code"], "invalid_request");
    }
    let status: String = sqlx::query_scalar("SELECT status FROM practice_sets")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "ready", "a rejected submission changes nothing");
}

#[sqlx::test(migrations = "../../migrations")]
async fn sets_are_listed_newest_first(pool: PgPool) {
    let (app, _fake) = world(pool).await;
    let budi = app.student("budi").await;
    let first = generate(&app, &budi, "mcq_normal", 5).await;
    let second = generate(&app, &budi, "mcq_story", 5).await;
    let list = app.get("/v1/practice-sets?limit=1", Some(&budi)).await;
    assert_eq!(list.status, StatusCode::OK);
    assert_eq!(list.body["data"][0]["id"], second["id"]);
    assert_eq!(list.body["data"][0]["status"], "ready");
    assert!(list.body["data"][0]["scoreCorrect"].is_null());
    let cursor = list.body["nextCursor"].as_str().unwrap();
    let next = app
        .get(
            &format!("/v1/practice-sets?limit=1&cursor={cursor}"),
            Some(&budi),
        )
        .await;
    assert_eq!(next.body["data"][0]["id"], first["id"]);
    assert!(next.body["nextCursor"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn another_students_set_is_not_found(pool: PgPool) {
    let (app, _fake) = world(pool.clone()).await;
    let budi = app.student("budi").await;
    let siti = app.student("siti").await;
    let set = generate(&app, &budi, "mcq_normal", 5).await;
    let id = set["id"].as_str().unwrap().to_owned();
    let path = format!("/v1/practice-sets/{id}");
    let answers: Vec<Value> = set["questions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| json!({ "questionId": q["id"], "selected": ["A"] }))
        .collect();

    // Positive control: the owner can read it.
    assert_eq!(app.get(&path, Some(&budi)).await.status, StatusCode::OK);

    let mut missing = app.get(MISSING, Some(&siti)).await.body;
    missing["error"]["requestId"] = Value::Null;
    assert_eq!(missing["error"]["code"], "not_found");
    for res in [
        app.get(&path, Some(&siti)).await,
        app.post(
            &format!("{path}/submit"),
            json!({ "answers": answers }),
            Some(&siti),
        )
        .await,
    ] {
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{}", res.text);
        let mut body = res.body;
        body["error"]["requestId"] = Value::Null;
        assert_eq!(body, missing);
    }
    assert_eq!(
        app.get("/v1/practice-sets", Some(&siti)).await.body["data"],
        json!([])
    );
    assert_eq!(
        app.get("/v1/topic-mastery", Some(&siti)).await.body["data"],
        json!([])
    );
    let status: String = sqlx::query_scalar("SELECT status FROM practice_sets")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        status, "ready",
        "the other student's submit changed nothing"
    );

    // Positive control: the owner can still submit it.
    let ok = app
        .post(
            &format!("{path}/submit"),
            json!({ "answers": answers }),
            Some(&budi),
        )
        .await;
    assert_eq!(ok.status, StatusCode::OK);
}
