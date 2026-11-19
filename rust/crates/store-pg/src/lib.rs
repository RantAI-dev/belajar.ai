//! Postgres behind the store traits in `belajar-ports`. Every query on a student's rows filters
//! by that student's id; queries are runtime-checked (`query_as::<_, T>(...)`), never macros.

mod accounts;
mod conversations;
mod home;
mod lessons;
mod practice;

use sqlx::postgres::PgPool;

/// One Postgres pool behind every store trait. `Clone` is cheap: the pool is shared.
#[derive(Debug, Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    /// Connects to `url` (a few connections up front; the pool grows on demand).
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        Ok(Self {
            pool: PgPool::connect(url).await?,
        })
    }

    /// Wraps a pool someone else owns (a `#[sqlx::test]` database, a larger app).
    #[must_use]
    pub const fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Applies every migration in `rust/migrations`. The server and the seed tool call this at
    /// startup, so a deployment never needs a manual migration step.
    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("../../migrations").run(&self.pool).await
    }

    /// One round trip, for `GET /ready`.
    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
    }
}
