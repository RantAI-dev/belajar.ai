//! The seams between the server and the outside world: the upstream learning API, the clock and
//! (added per slice, one file each) the student-scoped stores. Implementations live in
//! `belajar-upstream`, `belajar-store-pg` and the test doubles; the server only sees these traits.

pub mod accounts;
pub mod clock;
pub mod conversations;
pub mod home;
pub mod lessons;
pub mod practice;
pub mod store;
pub mod upstream;

pub use accounts::{AccountStore, LoginRecord, SessionRecord, StudentAccount};
pub use clock::{Clock, SystemClock};
pub use conversations::{
    Conversation, ConversationMode, ConversationStore, Message, MessageRole, NewMessage, Page,
    PageCursor,
};
pub use home::{ContinueItem, HomeData, HomeStore, MissionCounts, MissionKind};
pub use lessons::{Lesson, LessonStep, LessonStore};
pub use practice::{
    GradedAnswer, NewPracticeSet, PracticeSetStatus, PracticeSetSummary, PracticeStore,
    StoredPracticeAnswer, StoredPracticeSet, SubmitRecord, TopicMasteryEntry,
};
pub use store::StoreError;
pub use upstream::Upstream;
