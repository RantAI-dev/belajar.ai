//! The slice's own HTTP tests, next to the acceptance spec: the wire shapes, the field errors,
//! per-student isolation with a missing-id control, and the mission rules the acceptance file
//! leaves out (ask and practice counts, a reward per Jakarta day).

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]

mod support;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::http::StatusCode;
use belajar_core::grade::Grade;
use belajar_ports::clock::Clock;
use belajar_ports::upstream::{
    Assistant, Chapter, ChatRequest, ChatStream, Mapel, PageImage, PracticeAnswer, PracticeRequest,
    PracticeSet, PracticeStream, QuestionScore, Upstream, UpstreamError,
};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::TestApp;
use uuid::Uuid;

/// The upstream with exactly the two lists this slice reads; the streaming calls are other
/// slices' and stay unconfigured.
struct ChapterStub;

#[async_trait]
impl Upstream for ChapterStub {
    async fn mapel(&self, _grade: Grade) -> Result<Vec<Mapel>, UpstreamError> {
        Ok(vec![Mapel {
            name: "IPA".into(),
            knowledge_base_id: "kb-ipa".into(),
            document_count: 12,
        }])
    }

    async fn chapters(&self, _grade: Grade, mapel: &str) -> Result<Vec<Chapter>, UpstreamError> {
        if mapel != "IPA" {
            return Ok(Vec::new());
        }
        Ok((1..=5_u32)
            .map(|number| Chapter {
                id: format!("bab-{number}"),
                number,
                book_number: number,
                title: format!("Bab {number}: IPA"),
                page: Some(number * 10),
                document_id: "dok-ipa".into(),
                document_title: "IPA Kelas VII".into(),
                topics: Vec::new(),
            })
            .collect())
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatStream, UpstreamError> {
        Err(UpstreamError::NotConfigured {
            grade: request.grade.get(),
            assistant: Assistant::Learn,
        })
    }

    async fn generate_practice(
        &self,
        request: PracticeRequest,
    ) -> Result<PracticeStream, UpstreamError> {
        Err(UpstreamError::NotConfigured {
            grade: request.grade.get(),
            assistant: Assistant::Practice,
        })
    }

    async fn grade_practice(
        &self,
        grade: Grade,
        _set: &PracticeSet,
        _answers: &[PracticeAnswer],
    ) -> Result<Vec<QuestionScore>, UpstreamError> {
        Err(UpstreamError::NotConfigured {
            grade: grade.get(),
            assistant: Assistant::Practice,
        })
    }

    async fn page(&self, _page_url: &str) -> Result<PageImage, UpstreamError> {
        Err(UpstreamError::Unavailable("not configured".into()))
    }
}

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

async fn app(pool: PgPool, clock: Arc<TestClock>) -> TestApp {
    TestApp::with_parts(pool, Arc::new(ChapterStub), clock).await
}

async fn student_id(pool: &PgPool, username: &str) -> Uuid {
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// An ask-mode conversation with one user message at `at`.
async fn ask_message(pool: &PgPool, student: Uuid, at: &str) {
    let conversation = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO conversations (id, student_id, mode, title, created_at, updated_at) \
         VALUES ($1, $2, 'ask', 'Tanya', $3, $3)",
    )
    .bind(conversation)
    .bind(student)
    .bind(at.parse::<DateTime<Utc>>().unwrap())
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, student_id, role, content, created_at, \
         updated_at) VALUES ($1, $2, $3, 'user', 'berapa suhu air mendidih?', $4, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(conversation)
    .bind(student)
    .bind(at.parse::<DateTime<Utc>>().unwrap())
    .execute(pool)
    .await
    .unwrap();
}

/// A submitted practice set at `at`.
async fn submitted_practice(pool: &PgPool, student: Uuid, at: &str) {
    sqlx::query(
        "INSERT INTO practice_sets (id, student_id, subject_id, topic, practice_type, \
         total_questions, status, practice, score_correct, submitted_at, created_at, updated_at) \
         VALUES ($1, $2, 'ipa', 'Pemuaian', 'mcq_normal', 5, 'submitted', '{}'::jsonb, 4, $3, $3, $3)",
    )
    .bind(Uuid::now_v7())
    .bind(student)
    .bind(at.parse::<DateTime<Utc>>().unwrap())
    .execute(pool)
    .await
    .unwrap();
}

async fn start_lesson(app: &TestApp, cookie: &str) -> Value {
    let res = app
        .post(
            "/v1/lessons",
            json!({ "subjectId": "ipa", "chapterId": "bab-1" }),
            Some(cookie),
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.text);
    res.body
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_lesson_moves_through_the_steps_and_never_patches_to_done(pool: PgPool) {
    let at = "2026-11-19T03:00:00Z";
    let app = app(pool, TestClock::at(at)).await;
    let budi = app.student("budi").await;
    let lesson = start_lesson(&app, &budi).await;

    // The wire shape, in camelCase, with the shared instant format.
    assert_eq!(lesson["subjectId"], "ipa");
    assert_eq!(lesson["chapterId"], "bab-1");
    assert_eq!(lesson["chapterTitle"], "Bab 1: IPA");
    assert_eq!(lesson["step"], "explain");
    assert_eq!(lesson["stars"], 0);
    assert!(lesson["completedAt"].is_null());
    assert_eq!(
        lesson["updatedAt"].as_str().map(str::to_owned),
        Some("2026-11-19T03:00:00.000Z".into()),
        "updatedAt is the clock's instant in the shared format: {}",
        lesson
    );
    assert!(
        lesson["id"]
            .as_str()
            .is_some_and(|id| Uuid::parse_str(id).is_ok())
    );

    for step in ["check", "quiz"] {
        let id = lesson["id"].as_str().unwrap();
        let res = app
            .patch(
                &format!("/v1/lessons/{id}"),
                json!({ "step": step }),
                Some(&budi),
            )
            .await;
        assert_eq!(res.status, StatusCode::OK, "{}", res.text);
        assert_eq!(res.body["step"], step);
        assert_eq!(res.body["id"], lesson["id"]);
    }
    let got = app
        .get(
            &format!("/v1/lessons/{}", lesson["id"].as_str().unwrap()),
            Some(&budi),
        )
        .await;
    assert_eq!(got.status, StatusCode::OK);
    assert_eq!(got.body["step"], "quiz");

    let id = lesson["id"].as_str().unwrap();
    let done = app
        .patch(
            &format!("/v1/lessons/{id}"),
            json!({ "step": "done" }),
            Some(&budi),
        )
        .await;
    assert_eq!(done.status, StatusCode::BAD_REQUEST, "{}", done.text);
    assert_eq!(done.body["error"]["code"], "invalid_request");
    assert_eq!(done.body["error"]["errors"][0]["field"], "/step");
    assert_eq!(done.body["error"]["errors"][0]["code"], "invalid_format");

    // A path segment that is not a uuid can never exist: 404 with the standard body.
    let malformed = app.get("/v1/lessons/not-a-uuid", Some(&budi)).await;
    assert_eq!(malformed.status, StatusCode::NOT_FOUND);
    assert_eq!(malformed.body["error"]["code"], "not_found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn bad_subjects_chapters_and_quiz_ranges_are_field_errors(pool: PgPool) {
    let app = app(pool, TestClock::at("2026-11-19T03:00:00Z")).await;
    let budi = app.student("budi").await;

    for (body, field) in [
        (
            json!({ "subjectId": "kimia", "chapterId": "bab-1" }),
            "/subjectId",
        ),
        (
            json!({ "subjectId": "ipa", "chapterId": "bab-99" }),
            "/chapterId",
        ),
    ] {
        let res = app.post("/v1/lessons", body, Some(&budi)).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{}", res.text);
        assert_eq!(res.body["error"]["code"], "invalid_request");
        assert_eq!(res.body["error"]["errors"][0]["field"], field);
        assert_eq!(res.body["error"]["errors"][0]["code"], "invalid_format");
    }

    let lesson = start_lesson(&app, &budi).await;
    let id = lesson["id"].as_str().unwrap();
    for (correct, total, field) in [
        (6, 5, "/quizCorrect"),
        (0, 0, "/quizTotal"),
        (1, 21, "/quizTotal"),
    ] {
        let res = app
            .post(
                &format!("/v1/lessons/{id}/complete"),
                json!({ "quizCorrect": correct, "quizTotal": total }),
                Some(&budi),
            )
            .await;
        assert_eq!(
            res.status,
            StatusCode::BAD_REQUEST,
            "{correct}/{total}: {}",
            res.text
        );
        assert_eq!(res.body["error"]["errors"][0]["field"], field);
        assert_eq!(res.body["error"]["errors"][0]["code"], "out_of_range");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_lesson_is_invisible_to_another_student_like_a_missing_one(pool: PgPool) {
    let app = app(pool, TestClock::at("2026-11-19T03:00:00Z")).await;
    let budi = app.student("budi").await;
    let siti = app.student("siti").await;
    let lesson = start_lesson(&app, &budi).await;
    let id = lesson["id"].as_str().unwrap();
    let path = format!("/v1/lessons/{id}");

    // Positive control: the owner reaches it.
    assert_eq!(app.get(&path, Some(&budi)).await.status, StatusCode::OK);

    let missing = app
        .get(
            "/v1/lessons/0193b2c4-7a1e-7c3f-9d2a-5e8f1b6c4d90",
            Some(&siti),
        )
        .await
        .body;
    let missing = {
        let mut body = missing;
        body["error"]["requestId"] = Value::Null;
        body
    };
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
        assert_eq!(
            body, missing,
            "another student's lesson is indistinguishable from none"
        );
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn ask_and_practice_missions_count_today_and_the_learn_chat_does_not(pool: PgPool) {
    let at = "2026-11-19T03:00:00Z"; // 10:00 WIB on the 19th.
    let app = app(pool.clone(), TestClock::at(at)).await;
    let budi = app.student("budi").await;
    let id = student_id(&pool, "budi").await;

    ask_message(&pool, id, at).await;
    submitted_practice(&pool, id, at).await;
    // A user message in a learn-mode conversation is activity, but not the ask mission.
    sqlx::query(
        "INSERT INTO conversations (id, student_id, mode, title, created_at, updated_at) \
         VALUES ($1, $2, 'learn', 'Belajar', $3, $3)",
    )
    .bind(Uuid::now_v7())
    .bind(id)
    .bind(at.parse::<DateTime<Utc>>().unwrap())
    .execute(&pool)
    .await
    .unwrap();

    let home = app.get("/v1/home", Some(&budi)).await;
    assert_eq!(home.status, StatusCode::OK);
    let missions = home.body["missions"].as_array().unwrap();
    assert_eq!(missions[0]["kind"], "ask");
    assert_eq!(missions[0]["progress"], 1);
    assert_eq!(missions[0]["isCompleted"], true);
    assert_eq!(missions[1]["kind"], "lesson");
    assert_eq!(missions[1]["progress"], 0);
    assert_eq!(missions[2]["kind"], "practice");
    assert_eq!(missions[2]["progress"], 1);
    assert_eq!(home.body["stars"], 20, "two mission rewards of 10 each");
    assert_eq!(
        home.body["streakDays"], 1,
        "the learn conversation is activity today"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_mission_reward_is_given_once_per_jakarta_day(pool: PgPool) {
    let clock = TestClock::at("2026-11-19T03:00:00Z"); // 10:00 WIB on the 19th.
    let app = app(pool.clone(), clock.clone()).await;
    let budi = app.student("budi").await;
    let id = student_id(&pool, "budi").await;

    ask_message(&pool, id, "2026-11-19T03:00:00Z").await;
    assert_eq!(app.get("/v1/home", Some(&budi)).await.body["stars"], 10);
    assert_eq!(
        app.get("/v1/home", Some(&budi)).await.body["stars"],
        10,
        "the day's reward is given once"
    );

    // 17:30 UTC is already the next Jakarta day: no ask activity today, no new reward.
    clock.set("2026-11-19T17:30:00Z");
    let home = app.get("/v1/home", Some(&budi)).await;
    let ask = &home.body["missions"][0];
    assert_eq!(
        ask["progress"], 0,
        "the 19th's message is not on the 20th in Jakarta"
    );
    assert_eq!(home.body["stars"], 10, "no reward without today's activity");

    // Today's activity earns today's reward, on top of yesterday's.
    ask_message(&pool, id, "2026-11-19T17:30:00Z").await;
    assert_eq!(app.get("/v1/home", Some(&budi)).await.body["stars"], 20);
}
