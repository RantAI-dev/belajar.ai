//! Acceptance tests for Belajar (subjects, chapters, guided lessons with stars) and Beranda (home:
//! stars, streak, daily missions, continue learning). Written by the lead before the slice
//! started; the worker must make them pass without editing this file.
#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::http::StatusCode;
use belajar_fake_upstream::{FakeConfig, FakeServer};
use belajar_ports::Clock;
use belajar_upstream::{HttpUpstream, UpstreamConfig, UpstreamKeys};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::TestApp;

const KEYS: &str = "7.ask=fake-7-ask,7.learn=fake-7-learn,7.practice=fake-7-practice";

struct TestClock(Mutex<DateTime<Utc>>);

impl TestClock {
    fn at(instant: &str) -> Arc<Self> {
        Arc::new(Self(Mutex::new(instant.parse().unwrap())))
    }
    fn set(&self, instant: &str) {
        *self.0.lock().unwrap() = instant.parse().unwrap();
    }
}

impl Clock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().unwrap()
    }
}

async fn world(pool: PgPool, clock: Arc<TestClock>) -> (TestApp, FakeServer) {
    let fake = belajar_fake_upstream::spawn(FakeConfig::default())
        .await
        .unwrap();
    let upstream = HttpUpstream::new(UpstreamConfig {
        base_url: fake.base_url.clone(),
        keys: UpstreamKeys::parse(KEYS).unwrap(),
        timeout: Duration::from_secs(10),
    })
    .unwrap();
    let app = TestApp::with_parts(pool, Arc::new(upstream), clock).await;
    (app, fake)
}

async fn start(app: &TestApp, cookie: &str, chapter: &str) -> Value {
    let res = app
        .post(
            "/v1/lessons",
            json!({ "subjectId": "ipa", "chapterId": chapter }),
            Some(cookie),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text);
    res.body
}

async fn complete(app: &TestApp, cookie: &str, lesson: &str, correct: u64, total: u64) -> Value {
    let res = app
        .post(
            &format!("/v1/lessons/{lesson}/complete"),
            json!({ "quizCorrect": correct, "quizTotal": total }),
            Some(cookie),
        )
        .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    res.body
}

#[sqlx::test(migrations = "../../migrations")]
async fn subjects_come_from_the_upstream_in_a_fixed_order_with_our_ids(pool: PgPool) {
    let (app, _fake) = world(pool, TestClock::at("2026-11-19T03:00:00Z")).await;
    let budi = app.student("budi").await;
    let res = app.get("/v1/subjects", Some(&budi)).await;
    assert_eq!(res.status, StatusCode::OK);
    let ids: Vec<&str> = res.body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        vec!["ipa", "mtk", "bing", "bind", "ips", "inf", "pp", "pai"]
    );
    let ipa = &res.body["data"][0];
    assert_eq!(ipa["name"], "IPA");
    assert_eq!(ipa["mapel"], "IPA");
    assert!(ipa["documentCount"].as_u64().unwrap() > 0);
    assert!(res.body["nextCursor"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn chapters_show_the_students_own_progress(pool: PgPool) {
    let (app, _fake) = world(pool, TestClock::at("2026-11-19T03:00:00Z")).await;
    let budi = app.student("budi").await;
    let before = app.get("/v1/subjects/ipa/chapters", Some(&budi)).await;
    assert_eq!(before.status, StatusCode::OK);
    let chapters = before.body["data"].as_array().unwrap();
    assert!(chapters.len() >= 3);
    assert_eq!(chapters[0]["id"], "bab-1");
    assert_eq!(chapters[0]["number"], 1);
    assert!(chapters[0]["title"].is_string());
    assert!(
        chapters
            .iter()
            .all(|c| c["stars"] == 0 && c["step"].is_null() && c["isCompleted"] == false)
    );

    let lesson = start(&app, &budi, "bab-1").await;
    complete(&app, &budi, lesson["id"].as_str().unwrap(), 5, 5).await;
    let after = app.get("/v1/subjects/ipa/chapters", Some(&budi)).await;
    let first = &after.body["data"][0];
    assert_eq!(first["stars"], 3);
    assert_eq!(first["step"], "done");
    assert_eq!(first["isCompleted"], true);
    assert_eq!(first["lessonId"], lesson["id"]);

    let unknown = app.get("/v1/subjects/kimia/chapters", Some(&budi)).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(unknown.body["error"]["code"], "not_found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_lesson_opens_a_learn_conversation_and_resumes_on_the_second_start(pool: PgPool) {
    let (app, _fake) = world(pool.clone(), TestClock::at("2026-11-19T03:00:00Z")).await;
    let budi = app.student("budi").await;
    let lesson = start(&app, &budi, "bab-1").await;
    assert_eq!(lesson["subjectId"], "ipa");
    assert_eq!(lesson["chapterId"], "bab-1");
    assert_eq!(lesson["step"], "explain");
    assert_eq!(lesson["stars"], 0);
    assert!(lesson["completedAt"].is_null());
    assert!(!lesson["chapterTitle"].as_str().unwrap().is_empty());

    let (mode, chapter, title): (String, Option<String>, String) =
        sqlx::query_as("SELECT mode, chapter_id, title FROM conversations WHERE id = $1::uuid")
            .bind(lesson["conversationId"].as_str().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(mode, "learn");
    assert_eq!(chapter.as_deref(), Some("bab-1"));
    assert_eq!(title, lesson["chapterTitle"].as_str().unwrap());

    let again = app
        .post(
            "/v1/lessons",
            json!({ "subjectId": "ipa", "chapterId": "bab-1" }),
            Some(&budi),
        )
        .await;
    assert_eq!(
        again.status,
        StatusCode::OK,
        "an existing lesson is resumed, not duplicated"
    );
    assert_eq!(again.body["id"], lesson["id"]);
    assert_eq!(again.body["conversationId"], lesson["conversationId"]);

    let id = lesson["id"].as_str().unwrap();
    let moved = app
        .patch(
            &format!("/v1/lessons/{id}"),
            json!({ "step": "quiz" }),
            Some(&budi),
        )
        .await;
    assert_eq!(moved.status, StatusCode::OK);
    assert_eq!(moved.body["step"], "quiz");
    let done = app
        .patch(
            &format!("/v1/lessons/{id}"),
            json!({ "step": "done" }),
            Some(&budi),
        )
        .await;
    assert_eq!(
        done.status,
        StatusCode::BAD_REQUEST,
        "only completing a lesson finishes it"
    );

    for body in [
        json!({ "subjectId": "kimia", "chapterId": "bab-1" }),
        json!({ "subjectId": "ipa", "chapterId": "bab-99" }),
    ] {
        let bad = app.post("/v1/lessons", body, Some(&budi)).await;
        assert_eq!(bad.status, StatusCode::BAD_REQUEST);
        assert_eq!(bad.body["error"]["code"], "invalid_request");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn completing_keeps_the_best_stars_and_awards_only_new_ones(pool: PgPool) {
    let (app, _fake) = world(pool.clone(), TestClock::at("2026-11-19T03:00:00Z")).await;
    let budi = app.student("budi").await;
    let id = start(&app, &budi, "bab-2").await["id"]
        .as_str()
        .unwrap()
        .to_owned();

    // 2 of 5 = 40%: one star.
    let first = complete(&app, &budi, &id, 2, 5).await;
    assert_eq!(first["lesson"]["stars"], 1);
    assert_eq!(first["lesson"]["step"], "done");
    assert!(first["lesson"]["completedAt"].is_string());
    assert_eq!(first["starsEarned"], 5);
    // 3 of 5 = 60%: two stars, one new level.
    let second = complete(&app, &budi, &id, 3, 5).await;
    assert_eq!(second["lesson"]["stars"], 2);
    assert_eq!(second["starsEarned"], 5);
    // All correct: three stars.
    let third = complete(&app, &budi, &id, 5, 5).await;
    assert_eq!(third["lesson"]["stars"], 3);
    assert_eq!(third["starsEarned"], 5);
    // A worse result never lowers the best, and earns nothing.
    let fourth = complete(&app, &budi, &id, 1, 5).await;
    assert_eq!(fourth["lesson"]["stars"], 3);
    assert_eq!(fourth["starsEarned"], 0);

    let ledger: i64 =
        sqlx::query_scalar("SELECT COALESCE(SUM(amount), 0)::bigint FROM star_ledger")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(ledger, 15);

    for (correct, total) in [(6, 5), (0, 0), (1, 21)] {
        let bad = app
            .post(
                &format!("/v1/lessons/{id}/complete"),
                json!({ "quizCorrect": correct, "quizTotal": total }),
                Some(&budi),
            )
            .await;
        assert_eq!(bad.status, StatusCode::BAD_REQUEST, "{correct}/{total}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn home_shows_stars_missions_and_what_to_continue(pool: PgPool) {
    let (app, _fake) = world(pool, TestClock::at("2026-11-19T03:00:00Z")).await;
    let budi = app.student("budi").await;
    let empty = app.get("/v1/home", Some(&budi)).await;
    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(empty.body["nickname"], "budi");
    assert_eq!(empty.body["grade"], 7);
    assert_eq!(empty.body["stars"], 0);
    assert_eq!(empty.body["streakDays"], 0);
    let kinds: Vec<&str> = empty.body["missions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, vec!["ask", "lesson", "practice"]);
    for m in empty.body["missions"].as_array().unwrap() {
        assert_eq!(m["progress"], 0);
        assert_eq!(m["target"], 1);
        assert_eq!(m["isCompleted"], false);
        assert_eq!(m["rewardStars"], 10);
        assert!(!m["title"].as_str().unwrap().is_empty());
    }
    assert_eq!(empty.body["continueLearning"], json!([]));

    let unfinished = start(&app, &budi, "bab-3").await;
    let finished = start(&app, &budi, "bab-1").await;
    complete(&app, &budi, finished["id"].as_str().unwrap(), 5, 5).await;

    let home = app.get("/v1/home", Some(&budi)).await;
    let lesson_mission = &home.body["missions"][1];
    assert_eq!(lesson_mission["kind"], "lesson");
    assert_eq!(lesson_mission["progress"], 1);
    assert_eq!(lesson_mission["isCompleted"], true);
    assert_eq!(
        home.body["stars"],
        15 + 10,
        "3 lesson stars x5, plus the mission reward"
    );
    assert_eq!(home.body["streakDays"], 1);
    let cont = home.body["continueLearning"].as_array().unwrap();
    assert_eq!(
        cont.len(),
        1,
        "finished lessons are not offered to continue"
    );
    assert_eq!(cont[0]["lessonId"], unfinished["id"]);
    assert_eq!(cont[0]["subjectId"], "ipa");
    assert_eq!(cont[0]["step"], "explain");

    let again = app.get("/v1/home", Some(&budi)).await;
    assert_eq!(again.body["stars"], 25, "a mission reward is given once");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_streak_counts_jakarta_days_and_survives_until_a_day_is_missed(pool: PgPool) {
    let clock = TestClock::at("2026-11-17T03:00:00Z"); // 10:00 WIB on the 17th
    let (app, _fake) = world(pool, clock.clone()).await;
    let budi = app.student("budi").await;
    start(&app, &budi, "bab-1").await;
    clock.set("2026-11-18T03:00:00Z");
    start(&app, &budi, "bab-2").await;
    assert_eq!(app.get("/v1/home", Some(&budi)).await.body["streakDays"], 2);

    // 17:30 UTC on the 18th is 00:30 WIB on the 19th: a new day in Jakarta.
    clock.set("2026-11-18T17:30:00Z");
    assert_eq!(
        app.get("/v1/home", Some(&budi)).await.body["streakDays"],
        2,
        "yesterday still counts"
    );
    start(&app, &budi, "bab-3").await;
    assert_eq!(app.get("/v1/home", Some(&budi)).await.body["streakDays"], 3);

    // The 20th passes with nothing done.
    clock.set("2026-11-21T03:00:00Z");
    assert_eq!(app.get("/v1/home", Some(&budi)).await.body["streakDays"], 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn another_students_lesson_and_home_are_their_own(pool: PgPool) {
    let (app, _fake) = world(pool.clone(), TestClock::at("2026-11-19T03:00:00Z")).await;
    let budi = app.student("budi").await;
    let siti = app.student("siti").await;
    let lesson = start(&app, &budi, "bab-1").await;
    let id = lesson["id"].as_str().unwrap();
    let path = format!("/v1/lessons/{id}");

    // Positive control: the owner reaches it.
    assert_eq!(app.get(&path, Some(&budi)).await.status, StatusCode::OK);

    let mut missing = app
        .get(
            "/v1/lessons/0193b2c4-7a1e-7c3f-9d2a-5e8f1b6c4d90",
            Some(&siti),
        )
        .await
        .body;
    missing["error"]["requestId"] = Value::Null;
    assert_eq!(missing["error"]["code"], "not_found");
    for res in [
        app.get(&path, Some(&siti)).await,
        app.patch(&path, json!({ "step": "quiz" }), Some(&siti))
            .await,
        app.post(
            &format!("{path}/complete"),
            json!({ "quizCorrect": 5, "quizTotal": 5 }),
            Some(&siti),
        )
        .await,
    ] {
        assert_eq!(res.status, StatusCode::NOT_FOUND, "{}", res.text);
        let mut body = res.body;
        body["error"]["requestId"] = Value::Null;
        assert_eq!(body, missing);
    }
    let step: String = sqlx::query_scalar("SELECT step FROM lesson_progress")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(step, "explain", "the other student's calls changed nothing");

    complete(&app, &budi, id, 5, 5).await;
    let home = app.get("/v1/home", Some(&siti)).await;
    assert_eq!(home.body["stars"], 0);
    assert_eq!(home.body["streakDays"], 0);
    assert_eq!(home.body["continueLearning"], json!([]));
    let chapters = app.get("/v1/subjects/ipa/chapters", Some(&siti)).await;
    assert_eq!(chapters.body["data"][0]["stars"], 0);
    assert!(
        app.get("/v1/home", Some(&budi)).await.body["stars"]
            .as_u64()
            .unwrap()
            > 0
    );
}
