//! The student's conversations and their messages, as the server sees them. No HTTP, no SQL:
//! those live in `belajar-server` and `belajar-store-pg`. Every method is student-scoped: a row
//! of another student is `NotFound`, never a different error.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::store::StoreError;
use crate::upstream::Source;

/// `ask` is Tanya, `learn` is a Belajar chapter session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationMode {
    Ask,
    Learn,
}

impl ConversationMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Learn => "learn",
        }
    }

    /// `None` is any value the column cannot hold, so the caller decides what to answer.
    #[must_use]
    pub fn from_stored(value: &str) -> Option<Self> {
        match value {
            "ask" => Some(Self::Ask),
            "learn" => Some(Self::Learn),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
}

impl MessageRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    pub id: Uuid,
    pub student_id: Uuid,
    pub mode: ConversationMode,
    /// Our subject id ("ipa", "mtk", ...), or `None` for "all subjects".
    pub subject_id: Option<String>,
    pub chapter_id: Option<String>,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub student_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    /// The upstream's cited book passages, `page_url` included; the server never sends those
    /// server-side fields to the browser.
    pub sources: Vec<Source>,
    /// `None` on user messages.
    pub is_grounded: Option<bool>,
    pub created_at: DateTime<Utc>,
}

/// A new message to store. `content` arrives already validated and trimmed; `sources` is empty
/// for user messages.
#[derive(Debug)]
pub struct NewMessage<'a> {
    pub conversation_id: Uuid,
    pub role: MessageRole,
    pub content: &'a str,
    pub sources: &'a [Source],
    pub is_grounded: Option<bool>,
    /// The conversation's title becomes this when this message is its first; `None` leaves the
    /// title alone. A title set by a rename is never overwritten, because a renamed conversation
    /// already has messages.
    pub title_if_first: Option<&'a str>,
}

/// A keyset cursor: the page continues strictly after this `(at, id)` pair. The wire form is
/// opaque; only the server encodes and decodes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageCursor {
    /// `updated_at` for conversations, `created_at` for messages.
    pub at: DateTime<Utc>,
    pub id: Uuid,
}

/// One page of a list: the rows and the cursor the next page needs, or `None` on the last page.
pub struct Page<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<PageCursor>,
}

#[async_trait]
pub trait ConversationStore: Send + Sync {
    /// Creates the conversation. `title` arrives already chosen; `now` is the clock's now.
    async fn create_conversation(
        &self,
        student_id: Uuid,
        mode: ConversationMode,
        subject_id: Option<&str>,
        chapter_id: Option<&str>,
        title: &str,
        now: DateTime<Utc>,
    ) -> Result<Conversation, StoreError>;

    /// One conversation of this student, or `NotFound` (the same answer for another student's
    /// conversation and for a missing one).
    async fn get_conversation(
        &self,
        student_id: Uuid,
        conversation_id: Uuid,
    ) -> Result<Conversation, StoreError>;

    /// Sets the title. `updated_at` is left alone: it tracks messages, not edits.
    async fn rename_conversation(
        &self,
        student_id: Uuid,
        conversation_id: Uuid,
        title: &str,
    ) -> Result<Conversation, StoreError>;

    /// The student's conversations, newest `updated_at` first, tie-broken by id descending.
    /// `mode` of `None` lists both; `after` continues after a cursor; `limit` is already
    /// validated (1..=50).
    async fn list_conversations(
        &self,
        student_id: Uuid,
        mode: Option<ConversationMode>,
        after: Option<PageCursor>,
        limit: u32,
    ) -> Result<Page<Conversation>, StoreError>;

    /// Stores one message. Also bumps the conversation's `updated_at` to `now`, and its title
    /// when `title_if_first` is set and this is the conversation's first message.
    async fn add_message(
        &self,
        student_id: Uuid,
        message: NewMessage<'_>,
        now: DateTime<Utc>,
    ) -> Result<Message, StoreError>;

    /// The conversation's messages, oldest first; the cursor continues to newer messages.
    /// A missing or foreign conversation is `NotFound`.
    async fn list_messages(
        &self,
        student_id: Uuid,
        conversation_id: Uuid,
        after: Option<PageCursor>,
        limit: u32,
    ) -> Result<Page<Message>, StoreError>;

    /// The last `limit` messages, oldest first, for the memory window of the next question.
    /// The new question is not among them.
    async fn last_messages(
        &self,
        student_id: Uuid,
        conversation_id: Uuid,
        limit: u32,
    ) -> Result<Vec<Message>, StoreError>;

    /// One message of this student, with its sources, or `NotFound`.
    async fn get_message(&self, student_id: Uuid, message_id: Uuid) -> Result<Message, StoreError>;
}
