//! `POST /api/v1/chat/completions`: one question, answered from the books, streamed.

use axum::Extension;
use axum::Json;
use axum::extract::State;
use axum::response::Response;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::rag::{self, Passage};
use crate::{AppState, Grade, data_frame, error_frame, prompt, sse_channel};

/// Below this reranker score the best passage does not answer the question: say so instead of
/// letting the model stretch it.
pub const GROUNDED_MIN: f64 = 0.2;
/// Passages under this score are dropped even when a better one grounds the answer.
const PASSAGE_MIN: f64 = 0.05;
/// About 400 words: a school answer, not an essay.
const ANSWER_TOKENS: u32 = 700;

#[derive(Deserialize)]
pub(crate) struct ChatIn {
    #[serde(default)]
    messages: Vec<MessageIn>,
    #[serde(default)]
    mapel: Option<String>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct MessageIn {
    role: String,
    #[serde(default)]
    content: String,
}

/// belajar's system profile, the earlier turns and the new question, apart.
fn split(messages: &[MessageIn]) -> (Option<String>, Vec<(String, String)>, String) {
    let profile = messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone());
    let turns: Vec<&MessageIn> = messages.iter().filter(|m| m.role != "system").collect();
    let (question, earlier) = match turns.split_last() {
        Some((last, earlier)) if last.role == "user" => (last.content.clone(), earlier.to_vec()),
        _ => (String::new(), turns),
    };
    let history = earlier
        .into_iter()
        .map(|m| (m.role.clone(), m.content.clone()))
        .collect();
    (profile, history, question)
}

fn chunk_frame(content: &str) -> bytes::Bytes {
    data_frame(&json!({
        "id": "pilot-chat",
        "object": "chat.completion.chunk",
        "choices": [{ "index": 0, "delta": { "content": content }, "finish_reason": null }],
    }))
}

fn final_frame(sources: Value, grounded: bool, score: Option<f64>) -> bytes::Bytes {
    data_frame(&json!({
        "id": "pilot-chat",
        "object": "chat.completion.chunk",
        "choices": [{ "index": 0, "delta": {}, "finish_reason": "stop" }],
        "sources": sources,
        "grounded": grounded,
        "retrieval_score": score,
    }))
}

/// The sources as the upstream spells them, `cited` set from the answer's own `[n]`.
pub(crate) fn sources_json(state: &AppState, passages: &[Passage], answer: &str) -> Value {
    let cited = rag::cited_numbers(answer);
    Value::Array(
        passages
            .iter()
            .map(|p| {
                json!({
                    "title": p.chunk.document_title,
                    "section": p.chunk.section,
                    "documentId": p.chunk.document_id,
                    "page": p.pdf_page(),
                    "number": p.number,
                    "cited": cited.contains(&p.number),
                    "page_url": state.page_url(&p.chunk.document_id, p.pdf_page()),
                })
            })
            .collect(),
    )
}

pub(crate) async fn chat(
    State(state): State<AppState>,
    Extension(Grade(grade)): Extension<Grade>,
    Json(request): Json<ChatIn>,
) -> Response {
    let (tx, response) = sse_channel();
    tokio::spawn(async move {
        let (profile, history, question) = split(&request.messages);
        let previous = history
            .iter()
            .rev()
            .find(|(role, _)| role == "user")
            .map(|(_, c)| c.as_str());
        let query = rag::search_query(&question, previous);
        let started = std::time::Instant::now();
        let passages = match state
            .retrieve(grade, request.mapel.as_deref(), &query)
            .await
        {
            Ok(passages) => passages,
            Err(error) => {
                tracing::error!(%error, "retrieval failed");
                let _ = tx
                    .send(error_frame("the books could not be searched"))
                    .await;
                return;
            }
        };
        let best = passages.first().map(|p| p.score);
        let passages: Vec<Passage> = passages
            .into_iter()
            .filter(|p| p.score >= PASSAGE_MIN)
            .collect();
        tracing::info!(
            grade,
            mapel = request.mapel.as_deref().unwrap_or("-"),
            passages = passages.len(),
            best = best.unwrap_or(f64::NAN),
            retrieval_ms = started.elapsed().as_millis() as u64,
            "retrieved"
        );
        if best.is_none_or(|b| b < GROUNDED_MIN) || passages.is_empty() {
            let _ = tx.send(chunk_frame(prompt::NOT_IN_BOOK)).await;
            let _ = tx.send(final_frame(json!([]), false, best)).await;
            let _ = tx
                .send(bytes::Bytes::from_static(b"data: [DONE]\n\n"))
                .await;
            return;
        }
        let history = prompt::recent_history(&history);
        let messages = prompt::answer_messages(profile.as_deref(), &history, &question, &passages);
        let mut stream = match state.llm.stream(&messages, ANSWER_TOKENS).await {
            Ok(stream) => Box::pin(stream),
            Err(error) => {
                tracing::error!(%error, "the model did not start");
                let _ = tx.send(error_frame("the model is unavailable")).await;
                return;
            }
        };
        let mut answer = String::new();
        while let Some(piece) = stream.next().await {
            match piece {
                Ok(text) => {
                    answer.push_str(&text);
                    if tx.send(chunk_frame(&text)).await.is_err() {
                        return; // belajar hung up; stop spending the model's time
                    }
                }
                Err(error) => {
                    tracing::error!(%error, "the model stream broke");
                    let _ = tx.send(error_frame("the answer was interrupted")).await;
                    return;
                }
            }
        }
        tracing::info!(
            total_ms = started.elapsed().as_millis() as u64,
            chars = answer.len(),
            "answered"
        );
        let sources = sources_json(&state, &passages, &answer);
        let _ = tx.send(final_frame(sources, true, best)).await;
        let _ = tx
            .send(bytes::Bytes::from_static(b"data: [DONE]\n\n"))
            .await;
    });
    response
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, reason = "tests")]

    use super::*;

    fn m(role: &str, content: &str) -> MessageIn {
        MessageIn {
            role: role.into(),
            content: content.into(),
        }
    }

    #[test]
    fn the_profile_history_and_question_are_split() {
        let (profile, history, question) = split(&[
            m("system", "profil"),
            m("user", "apa itu kalor?"),
            m("assistant", "kalor adalah ..."),
            m("user", "contohnya?"),
        ]);
        assert_eq!(profile.as_deref(), Some("profil"));
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].0, "user");
        assert_eq!(question, "contohnya?");
    }
}
