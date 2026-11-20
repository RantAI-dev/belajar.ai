//! The fake's API: mapel, chat, practice generation and the book page images (grading is
//! `grade.rs`).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::data::{self, Answer};
use crate::{
    AppState, bank, data_frame, error_response, event_frame, recorded, sse_body, sse_response,
};

pub(crate) async fn mapel() -> Json<Value> {
    Json(data::mapel_json())
}

// ===== chat =========================================================================================================

#[derive(Deserialize)]
pub(crate) struct ChatIn {
    #[serde(default)]
    messages: Vec<MsgIn>,
    mapel: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct MsgIn {
    role: String,
    #[serde(default)]
    content: String,
}

pub(crate) async fn chat(State(state): State<AppState>, Json(request): Json<ChatIn>) -> Response {
    let question = request
        .messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| message.content.as_str())
        .unwrap_or("materi umum");
    let answer = data::answer_for(question, request.mapel.as_deref());
    let frames = match answer {
        Answer::Grounded { text, sources } => chat_frames(
            &text,
            data::sources_json(&sources, &text, &state.base_url),
            json!(true),
            json!(data::grounded_score()),
        ),
        Answer::Ungrounded { text } => chat_frames(&text, json!([]), json!(false), Value::Null),
    };
    // The answer itself moves faster than the progress steps: step_delay / 4 per chunk.
    let chunk_delay = state.config.step_delay / 4;
    sse_response(sse_body(frames, chunk_delay, state.config.ping_every))
}

fn chat_frames(text: &str, sources: Value, grounded: Value, retrieval_score: Value) -> Vec<Bytes> {
    // Split on spaces only: newlines stay inside a chunk, so fenced blocks survive the stream.
    let words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
    let mut frames = Vec::new();
    for (i, piece) in words.chunks(3).enumerate() {
        // Like a real token stream, a chunk carries its own leading space: clients concatenate.
        let content = if i == 0 {
            piece.join(" ")
        } else {
            format!(" {}", piece.join(" "))
        };
        frames.push(data_frame(&json!({
            "id": "fake-chat-1",
            "object": "chat.completion.chunk",
            "choices": [{ "index": 0, "delta": { "content": content }, "finish_reason": null }],
        })));
    }
    frames.push(data_frame(&json!({
        "id": "fake-chat-1",
        "object": "chat.completion.chunk",
        "choices": [{ "index": 0, "delta": {}, "finish_reason": "stop" }],
        "sources": sources,
        "grounded": grounded,
        "retrieval_score": retrieval_score,
    })));
    frames.push(Bytes::from_static(b"data: [DONE]\n\n"));
    frames
}

// ===== practice generation ==========================================================================================

#[derive(Deserialize)]
pub(crate) struct GenerateIn {
    topic: Option<String>,
    #[serde(default)]
    topics: Option<Vec<String>>,
    #[serde(default)]
    practice_type: Option<String>,
    #[serde(default)]
    total_questions: Option<u8>,
    #[serde(default)]
    stream: bool,
}

pub(crate) async fn generate_practice(
    State(state): State<AppState>,
    Json(request): Json<GenerateIn>,
) -> Response {
    let mut topics = request.topics.clone().unwrap_or_default();
    if topics.is_empty()
        && let Some(topic) = request.topic.clone()
    {
        topics.push(topic);
    }
    if topics.is_empty() {
        topics.push("Pemuaian".into());
    }
    // The server validates 5, 10 or 15 before calling; the fake just stays within the bank's size.
    let total = request.total_questions.unwrap_or(5).clamp(1, 50) as usize;
    let practice_type = request.practice_type.as_deref().unwrap_or("mcq_normal");

    // The reserved topic "__error__" replays the real upstream's recorded failure, so a test
    // watches the error path end to end.
    if topics.iter().any(|topic| topic == "__error__") {
        if !request.stream {
            let body: Value = serde_json::from_str(recorded::GENERATION_FAILED_JSON)
                .unwrap_or_else(|_| json!({ "error": { "message": "generation failed" } }));
            return (StatusCode::BAD_GATEWAY, Json(body)).into_response();
        }
        return sse_response(sse_body(
            recorded::sse_frames(recorded::GENERATION_FAILED_STREAM),
            state.config.step_delay,
            state.config.ping_every,
        ));
    }

    // The reserved topic "__short__" delivers 2 of 5, as the real API did for the recorded
    // `generate-mcq_multiple_correct-json` when its checks rejected the rest.
    let delivered = if topics.iter().any(|topic| topic == "__short__") {
        (total * 2 / 5).max(1)
    } else {
        total
    };
    let set = bank::set_json(practice_type, total, delivered, &topics);
    if !request.stream {
        return Json(set).into_response();
    }
    let mut frames = vec![event_frame(
        "progress",
        &json!({ "stage": "retrieval", "delivered": 0, "adapter_calls": 0, "total": total, "elapsed_s": 0 }),
    )];
    for step in 1..=delivered {
        frames.push(event_frame(
            "progress",
            &json!({
                "stage": "generating",
                "delivered": step,
                "total": total,
                "adapter_calls": step + 1,
                "elapsed_s": 0.2 + step as f64 * 0.3,
            }),
        ));
    }
    let done = total as f64;
    frames.push(event_frame(
        "progress",
        &json!({ "stage": "checking", "delivered": delivered, "total": total, "adapter_calls": total + 2, "elapsed_s": 0.3 + done * 0.3 }),
    ));
    frames.push(event_frame(
        "progress",
        &json!({ "stage": "labelling", "delivered": delivered, "adapter_calls": total + 2, "total": total, "elapsed_s": 0.4 + done * 0.3 }),
    ));
    frames.push(event_frame("result", &set));
    sse_response(sse_body(
        frames,
        state.config.step_delay,
        state.config.ping_every,
    ))
}

// ===== book pages ==================================================================================================

/// A pre-signed page: `documentId` and `page` as the upstream names them, no key needed.
pub(crate) async fn page_image(Path((document_id, page)): Path<(String, String)>) -> Response {
    let Some(number) = page
        .strip_suffix(".svg")
        .and_then(|page| page.parse::<u32>().ok())
    else {
        return error_response(StatusCode::NOT_FOUND, "no such page", "not_found");
    };
    let svg = render_page(&document_id, number);
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        svg,
    )
        .into_response()
}

/// A simple book page: a title line, grey text lines, two lines highlighted yellow, the page
/// number. `documentId` is interpolated into XML, so escape it even though we only ever see our
/// own ids.
fn render_page(document_id: &str, page: u32) -> String {
    let title = escape(document_id);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="360" height="480" viewBox="0 0 360 480">
<rect width="360" height="480" fill="#fffdf3"/>
<text x="24" y="42" font-family="Georgia, serif" font-size="16" fill="#222">{title}</text>
<text x="24" y="70" font-family="Georgia, serif" font-size="11" fill="#9a9a9a">Pada bab ini kita mempelajari pengertian, contoh, dan penerapannya.</text>
<text x="24" y="86" font-family="Georgia, serif" font-size="11" fill="#9a9a9a">Materi disusun agar mudah dipahami oleh siswa kelas tujuh.</text>
<text x="24" y="102" font-family="Georgia, serif" font-size="11" fill="#9a9a9a">Setiap subbab diawali dengan pertanyaan pemantik sederhana.</text>
<rect x="20" y="118" width="320" height="15" fill="#ffe97a" opacity="0.85"/>
<text x="24" y="130" font-family="Georgia, serif" font-size="11" fill="#333">Definisi utama dicetak tebal pada paragraf pertama bab ini.</text>
<rect x="20" y="137" width="320" height="15" fill="#ffe97a" opacity="0.85"/>
<text x="24" y="149" font-family="Georgia, serif" font-size="11" fill="#333">Bagian ini sering muncul dalam soal ulangan akhir semester.</text>
<text x="24" y="172" font-family="Georgia, serif" font-size="11" fill="#9a9a9a">Ringkasan dan latihan berada di halaman-halaman terakhir.</text>
<text x="24" y="188" font-family="Georgia, serif" font-size="11" fill="#9a9a9a">Kunci jawaban tersedia pada buku pegangan guru.</text>
<text x="180" y="462" text-anchor="middle" font-family="Georgia, serif" font-size="11" fill="#777">Halaman {page}</text>
</svg>"##
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The error body the auth guard and the handlers share: the upstream's own error shape.
pub(crate) fn error_json(status: StatusCode, message: &str, code: &str) -> Response {
    error_response(status, message, code)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]

    use super::*;

    #[test]
    fn the_page_svg_escapes_the_document_id() {
        let svg = render_page("kb-<ipa>&7", 12);
        assert!(svg.contains("kb-&lt;ipa&gt;&amp;7"));
        assert!(svg.contains("Halaman 12"));
    }

    #[test]
    fn chat_answers_arrive_in_word_chunks() {
        let frames = chat_frames(
            "Rel diberi celah supaya bisa memuai dengan aman.",
            json!([]),
            json!(true),
            Value::Null,
        );
        // 8 words over chunks of 3: 3 delta chunks, one final chunk, then [DONE].
        assert_eq!(frames.len(), 5);
        let last = String::from_utf8(frames.get(3).unwrap().to_vec()).unwrap();
        assert!(last.contains("\"finish_reason\":\"stop\""));
        assert!(last.contains("\"grounded\":true"));
        assert!(String::from_utf8_lossy(frames.get(4).unwrap()).contains("[DONE]"));
    }

    #[test]
    fn chat_chunks_concatenate_back_to_the_answer() {
        let text = "Rel diberi celah supaya bisa memuai dengan aman.";
        let joined: String = chat_frames(text, json!([]), json!(true), Value::Null)
            .iter()
            .filter_map(|frame| {
                let line = String::from_utf8(frame.to_vec()).unwrap();
                let json: Value = serde_json::from_str(line.strip_prefix("data: ")?.trim()).ok()?;
                json.pointer("/choices/0/delta/content")?
                    .as_str()
                    .map(str::to_owned)
            })
            .collect();
        assert_eq!(joined, text);
    }
}
