#![allow(clippy::unwrap_used, reason = "tests")]
#![allow(clippy::expect_used, reason = "tests")]

use belajar_core::grade::Grade;
use belajar_core::theme::ThemePreference;
use belajar_ports::accounts::AccountStore;
use belajar_ports::store::StoreError;
use belajar_store_pg::Db;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

async fn db(pool: &PgPool) -> Db {
    Db::from_pool(pool.clone())
}

#[sqlx::test(migrations = "../../migrations")]
async fn creating_a_student_is_case_insensitively_unique(pool: PgPool) {
    let db = db(&pool).await;
    let first = db
        .create_student("budi", "hash", "Budi", Grade::new(7).unwrap())
        .await
        .unwrap();
    assert_eq!(first.username, "budi");
    assert_eq!(first.grade.get(), 7);

    let again = db
        .create_student("BUDI", "hash", "Budi", Grade::new(7).unwrap())
        .await;
    assert!(
        matches!(again, Err(StoreError::Conflict)),
        "any case collides"
    );

    let fetched = db.get_student(first.id).await.unwrap();
    assert_eq!(fetched, first);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_missing_student_is_not_found_and_updates_keep_the_rest(pool: PgPool) {
    let db = db(&pool).await;
    assert!(matches!(
        db.get_student(Uuid::now_v7()).await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        db.password_hash(Uuid::now_v7()).await,
        Err(StoreError::NotFound)
    ));

    let student = db
        .create_student("siti", "hash", "Siti", Grade::new(3).unwrap())
        .await
        .unwrap();

    let updated = db
        .update_profile(student.id, None, Some(Grade::new(4).unwrap()), None)
        .await
        .unwrap();
    assert_eq!(updated.nickname, "Siti", "nickname untouched");
    assert_eq!(updated.grade.get(), 4);

    let updated = db
        .update_profile(student.id, Some("Sitii"), None, None)
        .await
        .unwrap();
    assert_eq!(updated.nickname, "Sitii");
    assert_eq!(updated.grade.get(), 4, "grade untouched");

    db.set_password_hash(student.id, "hash2").await.unwrap();
    assert_eq!(db.password_hash(student.id).await.unwrap(), "hash2");
    assert_eq!(
        db.find_login_by_username("siti")
            .await
            .unwrap()
            .unwrap()
            .password_hash,
        "hash2"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn sessions_expire_and_the_current_one_survives_a_password_change(pool: PgPool) {
    let db = db(&pool).await;
    let student = db
        .create_student("rani", "hash", "Rani", Grade::new(8).unwrap())
        .await
        .unwrap();
    let now = Utc::now();

    let expires = now + Duration::try_days(30).unwrap();
    let a = db
        .create_session(student.id, &[1, 2, 3], expires)
        .await
        .unwrap();
    db.create_session(student.id, &[4, 5, 6], expires)
        .await
        .unwrap();
    db.create_session(
        student.id,
        &[7, 8, 9],
        now - Duration::try_hours(1).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(
        db.find_session(&[1, 2, 3], now).await.unwrap().unwrap(),
        belajar_ports::accounts::SessionRecord {
            session_id: a,
            student_id: student.id
        }
    );
    assert!(
        db.find_session(&[7, 8, 9], now).await.unwrap().is_none(),
        "already expired"
    );
    assert!(db.find_session(&[9, 9], now).await.unwrap().is_none());

    // A password change keeps only the current session.
    db.delete_other_sessions(student.id, a).await.unwrap();
    assert!(db.find_session(&[1, 2, 3], now).await.unwrap().is_some());
    assert!(db.find_session(&[4, 5, 6], now).await.unwrap().is_none());

    db.delete_session(student.id, a).await.unwrap();
    assert!(db.find_session(&[1, 2, 3], now).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_theme_starts_at_system_and_changes_only_for_its_own_student(pool: PgPool) {
    let db = db(&pool).await;
    let budi = db
        .create_student("budi", "hash", "Budi", Grade::new(5).unwrap())
        .await
        .unwrap();
    let siti = db
        .create_student("siti", "hash", "Siti", Grade::new(5).unwrap())
        .await
        .unwrap();
    assert_eq!(budi.theme, ThemePreference::System);
    assert_eq!(
        db.get_student(siti.id).await.unwrap().theme,
        ThemePreference::System
    );

    let updated = db
        .update_profile(budi.id, None, None, Some(ThemePreference::Dark))
        .await
        .unwrap();
    assert_eq!(updated.theme, ThemePreference::Dark, "owner: changed");
    assert_eq!(updated.nickname, "Budi", "nickname untouched");
    assert_eq!(
        db.get_student(siti.id).await.unwrap().theme,
        ThemePreference::System,
        "another student's theme is untouched"
    );

    let kept = db
        .update_profile(budi.id, Some("Budi K"), None, None)
        .await
        .unwrap();
    assert_eq!(kept.theme, ThemePreference::Dark, "None keeps the theme");
}
