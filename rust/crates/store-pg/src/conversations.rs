//! `ConversationStore` on Postgres. Every query on a student's rows filters by `student_id`, so
//! another student's conversation is indistinguishable from a missing one. All times come from
//! the caller's clock, never from the database's `now()`.

use async_trait::async_trait;
use belajar_ports::conversations::{
    Conversation, ConversationMode, ConversationStore, Message, MessageRole, NewMessage, Page,
    PageCursor,
};
use belajar_ports::store::StoreError;
use belajar_ports::upstream::Source;
use chrono::{DateTime, Utc};
use sqlx::types::Json;
use uuid::Uuid;

use crate::Db;

/// Every driver error in this slice is one the caller cannot cause: the text is for the log only.
fn internal(err: sqlx::Error) -> StoreError {
    StoreError::Internal(err.to_string())
}

fn map_option<T>(row: Option<T>) -> Result<T, StoreError> {
    row.ok_or(StoreError::NotFound)
}

#[derive(Debug, sqlx::FromRow)]
struct ConversationRow {
    id: Uuid,
    student_id: Uuid,
    mode: String,
    subject_id: Option<String>,
    chapter_id: Option<String>,
    title: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ConversationRow> for Conversation {
    type Error = StoreError;

    fn try_from(row: ConversationRow) -> Result<Self, Self::Error> {
        let mode = ConversationMode::from_stored(&row.mode).ok_or_else(|| {
            StoreError::Internal(format!("conversations.mode drifted: {}", row.mode))
        })?;
        Ok(Conversation {
            id: row.id,
            student_id: row.student_id,
            mode,
            subject_id: row.subject_id,
            chapter_id: row.chapter_id,
            title: row.title,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MessageRow {
    id: Uuid,
    conversation_id: Uuid,
    student_id: Uuid,
    role: String,
    content: String,
    /// The ports `Source` values as stored, `page_url` included (server-side only).
    sources: Json<Vec<Source>>,
    is_grounded: Option<bool>,
    created_at: DateTime<Utc>,
}

impl TryFrom<MessageRow> for Message {
    type Error = StoreError;

    fn try_from(row: MessageRow) -> Result<Self, Self::Error> {
        let role = match row.role.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            other => {
                return Err(StoreError::Internal(format!(
                    "messages.role drifted: {other}"
                )));
            }
        };
        Ok(Message {
            id: row.id,
            conversation_id: row.conversation_id,
            student_id: row.student_id,
            role,
            content: row.content,
            sources: row.sources.0,
            is_grounded: row.is_grounded,
            created_at: row.created_at,
        })
    }
}

const CONVERSATION_COLUMNS: &str = "id, student_id, mode, subject_id, chapter_id, title, \
     created_at, updated_at";
const MESSAGE_COLUMNS: &str =
    "id, conversation_id, student_id, role, content, sources, is_grounded, created_at";

/// `Some` when the conversation exists and belongs to the student; the caller turns `None` into
/// `NotFound`. Used before any query that would otherwise return an empty list for a foreign id.
async fn owned_conversation<'e, E>(
    executor: E,
    student_id: Uuid,
    conversation_id: Uuid,
) -> Result<Option<Uuid>, StoreError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM conversations WHERE id = $1 AND student_id = $2",
    )
    .bind(conversation_id)
    .bind(student_id)
    .fetch_optional(executor)
    .await
    .map_err(internal)?;
    Ok(id)
}

/// Cuts a `limit + 1` fetch to one page and reads the next cursor off its last row.
fn cut_page<T, F>(rows: Vec<T>, limit: u32, key: F) -> Page<T>
where
    F: Fn(&T) -> PageCursor,
{
    let has_more = rows.len() > limit as usize;
    let mut data = rows;
    data.truncate(limit as usize);
    let next_cursor = if has_more { data.last().map(key) } else { None };
    Page { data, next_cursor }
}

#[async_trait]
impl ConversationStore for Db {
    async fn create_conversation(
        &self,
        student_id: Uuid,
        mode: ConversationMode,
        subject_id: Option<&str>,
        chapter_id: Option<&str>,
        title: &str,
        now: DateTime<Utc>,
    ) -> Result<Conversation, StoreError> {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO conversations \
             (id, student_id, mode, subject_id, chapter_id, title, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(id)
        .bind(student_id)
        .bind(mode.as_str())
        .bind(subject_id)
        .bind(chapter_id)
        .bind(title)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(internal)?;
        Ok(Conversation {
            id,
            student_id,
            mode,
            subject_id: subject_id.map(str::to_owned),
            chapter_id: chapter_id.map(str::to_owned),
            title: title.to_owned(),
            created_at: now,
            updated_at: now,
        })
    }

    async fn get_conversation(
        &self,
        student_id: Uuid,
        conversation_id: Uuid,
    ) -> Result<Conversation, StoreError> {
        let row = sqlx::query_as::<_, ConversationRow>(&format!(
            "SELECT {CONVERSATION_COLUMNS} FROM conversations WHERE id = $1 AND student_id = $2"
        ))
        .bind(conversation_id)
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        map_option(row)?.try_into()
    }

    async fn rename_conversation(
        &self,
        student_id: Uuid,
        conversation_id: Uuid,
        title: &str,
    ) -> Result<Conversation, StoreError> {
        let result =
            sqlx::query("UPDATE conversations SET title = $3 WHERE id = $1 AND student_id = $2")
                .bind(conversation_id)
                .bind(student_id)
                .bind(title)
                .execute(&self.pool)
                .await
                .map_err(internal)?;
        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        self.get_conversation(student_id, conversation_id).await
    }

    async fn list_conversations(
        &self,
        student_id: Uuid,
        mode: Option<ConversationMode>,
        after: Option<PageCursor>,
        limit: u32,
    ) -> Result<Page<Conversation>, StoreError> {
        // One row beyond the limit tells the caller there is a next page without a second query.
        let rows = sqlx::query_as::<_, ConversationRow>(&format!(
            "SELECT {CONVERSATION_COLUMNS} FROM conversations \
             WHERE student_id = $1 \
             AND ($2::text IS NULL OR mode = $2) \
             AND ($3::timestamptz IS NULL OR (updated_at, id) < ($3::timestamptz, $4::uuid)) \
             ORDER BY updated_at DESC, id DESC \
             LIMIT $5"
        ))
        .bind(student_id)
        .bind(mode.map(ConversationMode::as_str))
        .bind(after.map(|cursor| cursor.at))
        .bind(after.map(|cursor| cursor.id))
        // The caller validated 1..=50, so the addition cannot overflow.
        .bind(i64::from(limit) + 1)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        let conversations = rows
            .into_iter()
            .map(Conversation::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(cut_page(conversations, limit, |c| PageCursor {
            at: c.updated_at,
            id: c.id,
        }))
    }

    async fn add_message(
        &self,
        student_id: Uuid,
        message: NewMessage<'_>,
        now: DateTime<Utc>,
    ) -> Result<Message, StoreError> {
        let id = Uuid::now_v7();
        let mut tx = self.pool.begin().await.map_err(internal)?;
        // The ownership check and the inserts share one transaction, so a message is never
        // attached to another student's conversation.
        if owned_conversation(tx.as_mut(), student_id, message.conversation_id)
            .await?
            .is_none()
        {
            return Err(StoreError::NotFound);
        }
        sqlx::query(
            "INSERT INTO messages \
             (id, conversation_id, student_id, role, content, sources, is_grounded, \
              created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
        )
        .bind(id)
        .bind(message.conversation_id)
        .bind(student_id)
        .bind(message.role.as_str())
        .bind(message.content)
        .bind(Json(message.sources))
        .bind(message.is_grounded)
        .bind(now)
        .execute(tx.as_mut())
        .await
        .map_err(internal)?;
        // The title moves only when this is the conversation's first message, checked at the
        // moment of the insert, so a rename is never overwritten by a concurrent question.
        sqlx::query(
            "UPDATE conversations \
             SET updated_at = $3, \
                 title = CASE WHEN $4::text IS NOT NULL AND NOT EXISTS ( \
                     SELECT 1 FROM messages WHERE conversation_id = $1 AND id <> $5 \
                 ) THEN $4 ELSE title END \
             WHERE id = $1 AND student_id = $2",
        )
        .bind(message.conversation_id)
        .bind(student_id)
        .bind(now)
        .bind(message.title_if_first)
        .bind(id)
        .execute(tx.as_mut())
        .await
        .map_err(internal)?;
        tx.commit().await.map_err(internal)?;
        Ok(Message {
            id,
            conversation_id: message.conversation_id,
            student_id,
            role: message.role,
            content: message.content.to_owned(),
            sources: message.sources.to_vec(),
            is_grounded: message.is_grounded,
            created_at: now,
        })
    }

    async fn list_messages(
        &self,
        student_id: Uuid,
        conversation_id: Uuid,
        after: Option<PageCursor>,
        limit: u32,
    ) -> Result<Page<Message>, StoreError> {
        if owned_conversation(&self.pool, student_id, conversation_id)
            .await?
            .is_none()
        {
            return Err(StoreError::NotFound);
        }
        let rows = sqlx::query_as::<_, MessageRow>(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages \
             WHERE student_id = $1 AND conversation_id = $2 \
             AND ($3::timestamptz IS NULL OR (created_at, id) > ($3::timestamptz, $4::uuid)) \
             ORDER BY created_at ASC, id ASC \
             LIMIT $5"
        ))
        .bind(student_id)
        .bind(conversation_id)
        .bind(after.map(|cursor| cursor.at))
        .bind(after.map(|cursor| cursor.id))
        .bind(i64::from(limit) + 1)
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        let messages = rows
            .into_iter()
            .map(Message::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(cut_page(messages, limit, |m| PageCursor {
            at: m.created_at,
            id: m.id,
        }))
    }

    async fn last_messages(
        &self,
        student_id: Uuid,
        conversation_id: Uuid,
        limit: u32,
    ) -> Result<Vec<Message>, StoreError> {
        if owned_conversation(&self.pool, student_id, conversation_id)
            .await?
            .is_none()
        {
            return Err(StoreError::NotFound);
        }
        let rows = sqlx::query_as::<_, MessageRow>(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages \
             WHERE student_id = $1 AND conversation_id = $2 \
             ORDER BY created_at DESC, id DESC \
             LIMIT $3"
        ))
        .bind(student_id)
        .bind(conversation_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(internal)?;
        let mut messages: Vec<Message> = Vec::with_capacity(rows.len());
        for row in rows.into_iter().rev() {
            messages.push(Message::try_from(row)?);
        }
        Ok(messages)
    }

    async fn get_message(&self, student_id: Uuid, message_id: Uuid) -> Result<Message, StoreError> {
        let row = sqlx::query_as::<_, MessageRow>(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages WHERE id = $1 AND student_id = $2"
        ))
        .bind(message_id)
        .bind(student_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(internal)?;
        map_option(row)?.try_into()
    }
}
