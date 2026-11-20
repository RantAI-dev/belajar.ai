//! Creates (idempotently) the demo student: username `demo`, password `belajar123`, nickname
//! `Rani`, grade 7. Applies migrations first, so it works on a fresh database. Results go to
//! stdout; errors to stderr, exit 1.

use belajar_core::grade::Grade;
use belajar_ports::accounts::AccountStore;
use belajar_store_pg::Db;
use std::process::ExitCode;

const USERNAME: &str = "demo";
const PASSWORD: &str = "belajar123";
const NICKNAME: &str = "Rani";
const GRADE: u8 = 7;

#[tokio::main]
async fn main() -> ExitCode {
    let url = std::env::var("DATABASE_URL").unwrap_or_default();
    if url.is_empty() {
        eprintln!("seed-demo: DATABASE_URL is required and is not set.");
        return ExitCode::FAILURE;
    }
    let db = match Db::connect(&url).await {
        Ok(db) => db,
        Err(err) => {
            eprintln!("seed-demo: cannot connect to the database: {err}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(err) = db.migrate().await {
        eprintln!("seed-demo: migrations failed: {err}");
        return ExitCode::FAILURE;
    }

    // Store usernames are always lowercased, and `demo` already is.
    let existing = match db.find_login_by_username(USERNAME).await {
        Ok(existing) => existing,
        Err(err) => {
            eprintln!("seed-demo: cannot look up the demo student: {err}");
            return ExitCode::FAILURE;
        }
    };
    let Ok(grade) = Grade::new(GRADE) else {
        eprintln!("seed-demo: grade {GRADE} is out of range.");
        return ExitCode::FAILURE;
    };

    match existing {
        Some(record) => {
            let hash = match belajar_server::passwords::hash(PASSWORD.to_owned()).await {
                Ok(hash) => hash,
                Err(err) => {
                    eprintln!("seed-demo: cannot hash the password: {err:?}");
                    return ExitCode::FAILURE;
                }
            };
            if let Err(err) = db.set_password_hash(record.user_id, &hash).await {
                eprintln!("seed-demo: cannot set the password: {err}");
                return ExitCode::FAILURE;
            }
            if let Err(err) = db
                .update_profile(record.user_id, Some(NICKNAME), Some(grade), None)
                .await
            {
                eprintln!("seed-demo: cannot update the profile: {err}");
                return ExitCode::FAILURE;
            }
            println!("seed-demo: updated the demo student (username {USERNAME}, grade {GRADE}).");
        }
        None => {
            let hash = match belajar_server::passwords::hash(PASSWORD.to_owned()).await {
                Ok(hash) => hash,
                Err(err) => {
                    eprintln!("seed-demo: cannot hash the password: {err:?}");
                    return ExitCode::FAILURE;
                }
            };
            if let Err(err) = db.create_student(USERNAME, &hash, NICKNAME, grade).await {
                eprintln!("seed-demo: cannot create the demo student: {err}");
                return ExitCode::FAILURE;
            }
            println!("seed-demo: created the demo student (username {USERNAME}, grade {GRADE}).");
        }
    }
    ExitCode::SUCCESS
}
