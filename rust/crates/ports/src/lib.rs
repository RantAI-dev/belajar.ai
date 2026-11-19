//! The seams between the server and the outside world: the upstream learning API, the clock and
//! (added per slice, one file each) the student-scoped stores. Implementations live in
//! `belajar-upstream`, `belajar-store-pg` and the test doubles; the server only sees these traits.

pub mod accounts;
pub mod clock;
pub mod store;
pub mod upstream;

pub use accounts::{AccountStore, LoginRecord, SessionRecord, StudentAccount};
pub use clock::{Clock, SystemClock};
pub use store::StoreError;
pub use upstream::Upstream;
