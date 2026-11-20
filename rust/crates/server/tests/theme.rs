//! The theme preference on the student profile: its default, `PATCH /v1/me { theme }`, the field
//! error for a value outside the three, and isolation (a student changes only their own).

#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]
#![allow(clippy::indexing_slicing, reason = "tests")]

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::TestApp;

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_student_follows_the_device_theme(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let res = app
        .post(
            "/v1/auth/register",
            json!({ "username": "budi", "password": "rahasia-123", "nickname": "Budi", "grade": 4 }),
            None,
        )
        .await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_eq!(res.body["theme"], "system");
    let cookie = res.cookie().unwrap();
    assert_eq!(
        app.get("/v1/me", Some(&cookie)).await.body["theme"],
        "system"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_theme_is_saved_and_comes_back_on_me_and_on_login(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let cookie = app.student("budi").await;
    for theme in ["dark", "light", "system", "dark"] {
        let res = app
            .patch("/v1/me", json!({ "theme": theme }), Some(&cookie))
            .await;
        assert_eq!(res.status, StatusCode::OK, "{theme}");
        assert_eq!(res.body["theme"], theme);
        assert_eq!(res.body["nickname"], "budi", "nickname untouched");
    }
    assert_eq!(app.get("/v1/me", Some(&cookie)).await.body["theme"], "dark");

    let login = app
        .post(
            "/v1/auth/login",
            json!({ "username": "budi", "password": "rahasia-123" }),
            None,
        )
        .await;
    assert_eq!(login.status, StatusCode::OK);
    assert_eq!(
        login.body["theme"], "dark",
        "a new device gets the saved theme"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_theme_outside_the_three_is_a_field_error(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let cookie = app.student("budi").await;
    for bad in [json!("blue"), json!("Dark"), json!("")] {
        let res = app
            .patch("/v1/me", json!({ "theme": bad }), Some(&cookie))
            .await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(res.body["error"]["code"], "invalid_request");
        assert_eq!(res.body["error"]["errors"][0]["field"], "/theme");
        assert_eq!(res.body["error"]["errors"][0]["code"], "invalid_format");
    }
    let res = app
        .patch("/v1/me", json!({ "theme": 3 }), Some(&cookie))
        .await;
    assert_eq!(
        res.status,
        StatusCode::BAD_REQUEST,
        "a number is not a theme"
    );
    assert_eq!(res.body["error"]["code"], "invalid_request");
    assert_eq!(
        app.get("/v1/me", Some(&cookie)).await.body["theme"],
        "system"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_student_changes_only_their_own_theme(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let budi = app.student("budi").await;
    let siti = app.student("siti").await;

    // Positive control: the owner's change is visible to the owner.
    let res = app
        .patch("/v1/me", json!({ "theme": "dark" }), Some(&budi))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(app.get("/v1/me", Some(&budi)).await.body["theme"], "dark");

    // Isolation: the other student's profile is untouched, and their change leaves budi's.
    assert_eq!(app.get("/v1/me", Some(&siti)).await.body["theme"], "system");
    let res = app
        .patch("/v1/me", json!({ "theme": "light" }), Some(&siti))
        .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(app.get("/v1/me", Some(&siti)).await.body["theme"], "light");
    assert_eq!(app.get("/v1/me", Some(&budi)).await.body["theme"], "dark");
}

#[sqlx::test(migrations = "../../migrations")]
async fn changing_the_theme_needs_a_session(pool: PgPool) {
    let app = TestApp::new(pool).await;
    let res = app.patch("/v1/me", json!({ "theme": "dark" }), None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"]["code"], "unauthenticated");
}
