//! `GET /v1/progress`: the Progres page for the signed-in student. Rows are written straight into
//! the database at chosen instants, so the Jakarta week boundaries and every count are exact.
//! Isolation: the endpoint takes no student id; another student's rows never reach a response,
//! the owner sees exactly their own numbers, and a student with no rows sees the empty state.

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]

mod support;

use std::sync::Arc;

use axum::http::StatusCode;
use belajar_ports::clock::Clock;
use belajar_server::upstream_stub::UpstreamStub;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::TestApp;
use uuid::Uuid;

/// Thursday 2026-11-19, 10:00 WIB. This Jakarta week is Mon 16 to Sun 22 November.
const NOW: &str = "2026-11-19T03:00:00Z";

struct FixedClock(DateTime<Utc>);

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

async fn app(pool: PgPool) -> TestApp {
    TestApp::with_parts(
        pool,
        Arc::new(UpstreamStub),
        Arc::new(FixedClock(NOW.parse().unwrap())),
    )
    .await
}

async fn student_id(pool: &PgPool, username: &str) -> Uuid {
    sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn at(text: &str) -> DateTime<Utc> {
    text.parse().unwrap()
}

/// A submitted practice set with `correct` of `total` right, as Latihan's submit writes it:
/// the set, its answers, the topic mastery upsert and the star ledger row (5 per correct answer).
async fn practice(
    pool: &PgPool,
    student: Uuid,
    subject: &str,
    topic: &str,
    (correct, total): (i16, i16),
    when: &str,
) {
    let set_id = Uuid::now_v7();
    let when = at(when);
    sqlx::query(
        "INSERT INTO practice_sets (id, student_id, subject_id, topic, practice_type, \
             total_questions, status, practice, score_correct, submitted_at, created_at, \
             updated_at) \
         VALUES ($1, $2, $3, $4, 'mcq_normal', $5, 'submitted', '{}'::jsonb, $6, $7, $7, $7)",
    )
    .bind(set_id)
    .bind(student)
    .bind(subject)
    .bind(topic)
    .bind(total)
    .bind(correct)
    .bind(when)
    .execute(pool)
    .await
    .unwrap();
    for n in 0..total {
        sqlx::query(
            "INSERT INTO practice_answers (id, practice_set_id, student_id, question_id, \
                 selected, is_correct, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, '[\"A\"]'::jsonb, $5, $6, $6)",
        )
        .bind(Uuid::now_v7())
        .bind(set_id)
        .bind(student)
        .bind(format!("q{n}"))
        .bind(n < correct)
        .bind(when)
        .execute(pool)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO topic_mastery (id, student_id, subject_id, topic, attempts, correct, \
             created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7) \
         ON CONFLICT (student_id, subject_id, topic) DO UPDATE SET \
             attempts = topic_mastery.attempts + EXCLUDED.attempts, \
             correct = topic_mastery.correct + EXCLUDED.correct",
    )
    .bind(Uuid::now_v7())
    .bind(student)
    .bind(subject)
    .bind(topic)
    .bind(i32::from(total))
    .bind(i32::from(correct))
    .bind(when)
    .execute(pool)
    .await
    .unwrap();
    if correct > 0 {
        sqlx::query(
            "INSERT INTO star_ledger (id, student_id, amount, reason, ref_id, created_at, \
                 updated_at) VALUES ($1, $2, $3, 'practice', $4, $5, $5)",
        )
        .bind(Uuid::now_v7())
        .bind(student)
        .bind(i32::from(correct) * 5)
        .bind(set_id.to_string())
        .bind(when)
        .execute(pool)
        .await
        .unwrap();
    }
}

/// A Belajar lesson started at `started`, last touched then, completed at `completed` if given.
async fn lesson(
    pool: &PgPool,
    student: Uuid,
    subject: &str,
    chapter: &str,
    stars: i16,
    started: &str,
    completed: Option<&str>,
) {
    let conversation = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO conversations (id, student_id, mode, subject_id, chapter_id, title, \
             created_at, updated_at) VALUES ($1, $2, 'learn', $3, $4, $5, $6, $6)",
    )
    .bind(conversation)
    .bind(student)
    .bind(subject)
    .bind(chapter)
    .bind(format!("Judul {chapter}"))
    .bind(at(started))
    .execute(pool)
    .await
    .unwrap();
    let completed = completed.map(at);
    sqlx::query(
        "INSERT INTO lesson_progress (id, student_id, subject_id, chapter_id, conversation_id, \
             step, stars, completed_at, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, COALESCE($8, $9))",
    )
    .bind(Uuid::now_v7())
    .bind(student)
    .bind(subject)
    .bind(chapter)
    .bind(conversation)
    .bind(if completed.is_some() { "done" } else { "check" })
    .bind(stars)
    .bind(completed)
    .bind(at(started))
    .execute(pool)
    .await
    .unwrap();
}

/// One question asked in Tanya.
async fn ask(pool: &PgPool, student: Uuid, when: &str) {
    let conversation = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO conversations (id, student_id, mode, subject_id, title, created_at, \
             updated_at) VALUES ($1, $2, 'ask', 'ipa', 'Kenapa langit biru?', $3, $3)",
    )
    .bind(conversation)
    .bind(student)
    .bind(at(when))
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, student_id, role, content, created_at, \
             updated_at) VALUES ($1, $2, $3, 'user', 'Kenapa langit biru?', $4, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(conversation)
    .bind(student)
    .bind(at(when))
    .execute(pool)
    .await
    .unwrap();
}

/// Budi's week: two IPA topics and one Matematika topic, a finished and an unfinished chapter,
/// one question asked, and a perfect set at Sunday 23:59 WIB that belongs to last week.
async fn seed_budi(pool: &PgPool, budi: Uuid) {
    practice(
        pool,
        budi,
        "ipa",
        "Pemuaian",
        (2, 5),
        "2026-11-11T03:00:00Z",
    )
    .await;
    practice(
        pool,
        budi,
        "ipa",
        "Pemuaian",
        (4, 5),
        "2026-11-18T03:00:00Z",
    )
    .await;
    practice(pool, budi, "mtk", "Pecahan", (1, 5), "2026-11-17T03:00:00Z").await;
    // Sunday 15 November, 23:59 WIB: last week, by one minute.
    practice(pool, budi, "ipa", "Gaya", (5, 5), "2026-11-15T16:59:00Z").await;
    lesson(
        pool,
        budi,
        "ipa",
        "bab-1",
        3,
        "2026-11-18T04:00:00Z",
        Some("2026-11-18T05:00:00Z"),
    )
    .await;
    lesson(pool, budi, "ipa", "bab-2", 0, "2026-11-19T02:00:00Z", None).await;
    ask(pool, budi, "2026-11-19T01:00:00Z").await;
}

/// Siti's rows: a different subject, a different topic, another week's day.
async fn seed_siti(pool: &PgPool, siti: Uuid) {
    practice(pool, siti, "ips", "Peta", (1, 5), "2026-11-16T03:00:00Z").await;
    practice(
        pool,
        siti,
        "ipa",
        "Pemuaian",
        (5, 5),
        "2026-11-16T04:00:00Z",
    )
    .await;
    lesson(
        pool,
        siti,
        "mtk",
        "bab-9",
        2,
        "2026-11-16T05:00:00Z",
        Some("2026-11-16T06:00:00Z"),
    )
    .await;
    ask(pool, siti, "2026-11-12T01:00:00Z").await;
}

fn topics(body: &Value) -> Vec<String> {
    body["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["topics"].as_array().unwrap().iter())
        .map(|t| {
            format!(
                "{}/{}",
                t["subjectId"].as_str().unwrap(),
                t["topic"].as_str().unwrap()
            )
        })
        .collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn another_students_rows_never_reach_my_progress(pool: PgPool) {
    let app = app(pool.clone()).await;
    let budi_cookie = app.student("budi").await;
    let siti_cookie = app.student("siti").await;
    let kosong_cookie = app.student("kosong").await;
    let budi = student_id(&pool, "budi").await;
    let siti = student_id(&pool, "siti").await;
    seed_budi(&pool, budi).await;
    seed_siti(&pool, siti).await;

    // Positive control: the owner sees exactly their own numbers.
    let res = app.get("/v1/progress", Some(&budi_cookie)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    let body = &res.body;
    assert_eq!(
        topics(body),
        vec!["ipa/Pemuaian", "ipa/Gaya", "mtk/Pecahan"],
        "Budi's topics only, weakest first per subject"
    );
    assert_eq!(
        body["allTime"],
        json!({
            "stars": 60,
            "lessonsCompleted": 1,
            "questionsAnswered": 20,
            "correctAnswers": 12,
            "accuracyPercent": 60,
            "daysActive": 5,
        })
    );
    assert_eq!(
        body["thisWeek"],
        json!({
            "stars": 25,
            "lessonsCompleted": 1,
            "questionsAnswered": 10,
            "correctAnswers": 5,
            "accuracyPercent": 50,
            "daysActive": 3,
        })
    );
    assert_eq!(body["streakDays"], 3);
    let pemuaian = &body["subjects"][0]["topics"][0];
    assert_eq!(
        pemuaian["attempts"], 10,
        "Siti's 5/5 on the same topic is not Budi's"
    );
    assert_eq!(pemuaian["correct"], 6);
    let ids: Vec<&str> = body["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["subjectId"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec!["ipa", "mtk"], "no IPS: that is Siti's");
    let chapters: Vec<&str> = body["subjects"][0]["chapters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["chapterId"].as_str().unwrap())
        .collect();
    assert_eq!(chapters, vec!["bab-2", "bab-1"]);
    assert_eq!(
        body["subjects"][1]["chapters"],
        json!([]),
        "bab-9 is Siti's"
    );
    let sessions = body["recentSessions"].as_array().unwrap();
    assert_eq!(
        sessions.len(),
        7,
        "4 sets, 2 lessons, 1 question: all Budi's"
    );
    assert!(sessions.iter().all(|s| s["title"] != "Peta"));
    assert_eq!(body["activity"]["last30Days"]["questionsAsked"], 1);
    assert_eq!(body["activity"]["last30Days"]["questionsAnswered"], 20);

    // The other student sees only theirs.
    let res = app.get("/v1/progress", Some(&siti_cookie)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(topics(&res.body), vec!["ipa/Pemuaian", "ips/Peta"]);
    assert_eq!(res.body["allTime"]["questionsAnswered"], 10);
    assert_eq!(res.body["subjects"][0]["topics"][0]["attempts"], 5);

    // A student with no rows sees the empty state, whatever the others did.
    let res = app.get("/v1/progress", Some(&kosong_cookie)).await;
    assert_eq!(res.status, StatusCode::OK);
    let body = &res.body;
    assert_eq!(body["subjects"], json!([]));
    assert_eq!(body["practiceAgain"], json!([]));
    assert_eq!(body["continueLessons"], json!([]));
    assert_eq!(body["recentSessions"], json!([]));
    assert_eq!(body["streakDays"], 0);
    assert_eq!(body["allTime"]["questionsAnswered"], 0);
    assert_eq!(body["allTime"]["stars"], 0);
    assert_eq!(body["allTime"]["accuracyPercent"], Value::Null);
    assert_eq!(body["activity"]["last30Days"]["daysActive"], 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn weeks_trends_and_next_steps_follow_the_jakarta_calendar(pool: PgPool) {
    let app = app(pool.clone()).await;
    let cookie = app.student("budi").await;
    let budi = student_id(&pool, "budi").await;
    seed_budi(&pool, budi).await;

    let res = app.get("/v1/progress", Some(&cookie)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.text);
    let body = &res.body;
    assert_eq!(body["weekStartsOn"], "2026-11-16");
    assert_eq!(body["weekEndsOn"], "2026-11-22");

    let ipa = &body["subjects"][0];
    assert_eq!(ipa["subjectId"], "ipa");
    assert_eq!(ipa["name"], "IPA");
    // 2/5 + 4/5 + 5/5 = 11/15 = 73%.
    assert_eq!(ipa["masteryPercent"], 73);
    assert_eq!(ipa["level"], "almost");
    assert_eq!(ipa["lessonsStarted"], 2);
    assert_eq!(ipa["lessonsCompleted"], 1);

    let pemuaian = &ipa["topics"][0];
    assert_eq!(pemuaian["topic"], "Pemuaian");
    assert_eq!(pemuaian["masteryPercent"], 60);
    assert_eq!(pemuaian["level"], "almost");
    assert_eq!(pemuaian["thisWeekPercent"], 80);
    assert_eq!(pemuaian["lastWeekPercent"], 40);
    assert_eq!(pemuaian["trend"], "up");
    assert_eq!(pemuaian["lastPracticedAt"], "2026-11-18T03:00:00.000Z");

    // Sunday 23:59 WIB is last week: no answers this week, so no trend.
    let gaya = &ipa["topics"][1];
    assert_eq!(gaya["topic"], "Gaya");
    assert_eq!(gaya["level"], "mastered");
    assert_eq!(gaya["thisWeekPercent"], Value::Null);
    assert_eq!(gaya["lastWeekPercent"], 100);
    assert_eq!(gaya["trend"], "no_data");

    let pecahan = &body["subjects"][1]["topics"][0];
    assert_eq!(pecahan["level"], "learning");
    assert_eq!(pecahan["trend"], "no_data");

    let again: Vec<&str> = body["practiceAgain"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["topic"].as_str().unwrap())
        .collect();
    assert_eq!(
        again,
        vec!["Pecahan", "Pemuaian"],
        "weakest first, mastered left out"
    );
    let next = body["continueLessons"].as_array().unwrap();
    assert_eq!(next.len(), 1);
    assert_eq!(next[0]["chapterId"], "bab-2");
    assert_eq!(next[0]["isCompleted"], false);
    assert_eq!(next[0]["step"], "check");

    // The last 7 days end today (Thursday 19th) and start on Friday the 13th.
    let week = &body["activity"]["last7Days"];
    assert_eq!(week["startsOn"], "2026-11-13");
    assert_eq!(week["endsOn"], "2026-11-19");
    assert_eq!(week["days"].as_array().unwrap().len(), 7);
    assert_eq!(week["daysActive"], 4, "15th, 17th, 18th, 19th");
    assert_eq!(week["questionsAnswered"], 15);
    assert_eq!(week["lessonsCompleted"], 1);
    let sunday = &week["days"][2];
    assert_eq!(sunday["on"], "2026-11-15");
    assert_eq!(sunday["isActive"], true);
    assert_eq!(sunday["questionsAnswered"], 5);
    assert_eq!(sunday["correctAnswers"], 5);
    let month = &body["activity"]["last30Days"];
    assert_eq!(month["startsOn"], "2026-10-21");
    assert_eq!(month["daysActive"], 5);

    let first = &body["recentSessions"][0];
    assert_eq!(first["kind"], "lesson", "bab-2, touched at 09:00 WIB today");
    assert_eq!(first["occurredAt"], "2026-11-19T02:00:00.000Z");
    let practice = body["recentSessions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["kind"] == "practice")
        .unwrap();
    assert_eq!(practice["title"], "Pemuaian");
    assert_eq!(practice["correctCount"], 4);
    assert_eq!(practice["totalCount"], 5);
}

#[sqlx::test(migrations = "../../migrations")]
async fn progress_needs_a_session(pool: PgPool) {
    let app = app(pool).await;
    let res = app.get("/v1/progress", None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"]["code"], "unauthenticated");
}
