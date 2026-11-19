//! One state for every route. Later slices add one `Arc<dyn XStore>` field here and one field in
//! `Deps`, and nothing else changes: `app()` is the only place that constructs it.

use std::sync::Arc;

use belajar_ports::accounts::AccountStore;
use belajar_ports::clock::Clock;
use belajar_ports::home::HomeStore;
use belajar_ports::lessons::LessonStore;
use belajar_ports::upstream::Upstream;
use belajar_store_pg::Db;

use crate::config::Config;
use crate::login_limiter::LoginLimiter;

/// What `app()` needs from the outside world. `Db` is one pool behind every store trait, so a later
/// slice wraps `deps.db.clone()` in the `Arc<dyn XStore>` it needs.
pub struct Deps {
    pub db: Db,
    pub upstream: Arc<dyn Upstream>,
    pub clock: Arc<dyn Clock>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub accounts: Arc<dyn AccountStore>,
    pub lessons: Arc<dyn LessonStore>,
    pub home: Arc<dyn HomeStore>,
    pub upstream: Arc<dyn Upstream>,
    pub clock: Arc<dyn Clock>,
    pub config: Config,
    pub limiter: Arc<LoginLimiter>,
}

impl AppState {
    /// The one constructor: `app()` calls it, tests call it.
    pub fn new(config: Config, deps: Deps) -> Self {
        // Three stores over one pool: `Arc::new` needs an owned `Db` per trait object.
        let accounts: Arc<dyn AccountStore> = Arc::new(deps.db.clone());
        let lessons: Arc<dyn LessonStore> = Arc::new(deps.db.clone());
        let home: Arc<dyn HomeStore> = Arc::new(deps.db.clone());
        Self {
            db: deps.db,
            accounts,
            lessons,
            home,
            upstream: deps.upstream,
            clock: deps.clock.clone(),
            config,
            limiter: Arc::new(LoginLimiter::new(deps.clock)),
        }
    }
}
