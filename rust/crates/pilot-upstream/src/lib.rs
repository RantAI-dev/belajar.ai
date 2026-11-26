//! A small upstream for belajar-server built from parts: a knowledge base (rantai-kb) for the
//! books, a reranker, and an OpenAI-compatible model. It speaks the same wire protocol as the
//! upstream learning API (see `belajar-upstream`'s `wire` module and `belajar-fake-upstream`), so
//! belajar-server needs no change to use it: point `BELAJAR_UPSTREAM_URL` here.

pub mod catalog;
mod chat;
pub mod config;
pub mod kb;
pub mod llm;
mod pages;
mod practice;
pub mod prompt;
pub mod rag;
pub mod rerank;

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use serde_json::json;
use tokio::sync::{RwLock, mpsc};

use crate::catalog::Catalog;
use crate::config::Config;
use crate::kb::{Chunk, Kb};
use crate::llm::Llm;
use crate::rag::Passage;
use crate::rerank::Reranker;

pub struct App {
    pub config: Config,
    pub kb: Kb,
    pub llm: Llm,
    pub reranker: Option<Reranker>,
    pub catalog: RwLock<Catalog>,
}

pub type AppState = Arc<App>;

/// The grade the request's key belongs to.
#[derive(Clone, Copy)]
pub(crate) struct Grade(pub u8);

impl App {
    pub fn new(config: Config) -> Result<Self, String> {
        let kb = Kb::new(
            &config.kb_url,
            &config.kb_key,
            &config.cache_dir,
            config.timeout,
        )
        .map_err(|e| e.to_string())?;
        let llm = Llm::new(&config.llm_url, &config.llm_model, config.timeout)?;
        let reranker = config
            .rerank_url
            .as_deref()
            .map(|url| Reranker::new(url, config.timeout))
            .transpose()?;
        Ok(Self {
            config,
            kb,
            llm,
            reranker,
            catalog: RwLock::new(Catalog::default()),
        })
    }

    /// Reads the list of bases; kept for the life of the process, re-read when it was empty.
    pub async fn catalog(&self) -> Result<Catalog, String> {
        {
            let catalog = self.catalog.read().await;
            if !catalog.is_empty() {
                return Ok(catalog.clone());
            }
        }
        let bases = self.kb.bases().await.map_err(|e| e.to_string())?;
        let catalog = Catalog::new(bases);
        *self.catalog.write().await = catalog.clone();
        Ok(catalog)
    }

    /// Search the grade's (or the mapel's) books, rerank, and keep what fits the model.
    pub(crate) async fn retrieve(
        &self,
        grade: u8,
        mapel: Option<&str>,
        query: &str,
    ) -> Result<Vec<Passage>, String> {
        let bases = self.catalog().await?.bases_for(grade, mapel);
        if bases.is_empty() {
            return Ok(Vec::new());
        }
        let chunks = self
            .kb
            .search(query, &bases, rag::SEARCH_CHUNKS)
            .await
            .map_err(|e| e.to_string())?;
        let scores = self.scores(query, &chunks).await;
        Ok(rag::select(chunks, &scores))
    }

    /// The reranker's scores, or the knowledge base's own when the reranker is not there.
    async fn scores(&self, query: &str, chunks: &[Chunk]) -> Vec<f64> {
        let fallback = || {
            chunks
                .iter()
                .map(|c| {
                    c.retrieval
                        .as_ref()
                        .and_then(|r| r.rerank_score)
                        .unwrap_or(0.0)
                })
                .collect::<Vec<f64>>()
        };
        let Some(reranker) = &self.reranker else {
            return fallback();
        };
        let documents: Vec<String> = chunks.iter().map(|c| rag::clean(&c.content)).collect();
        match reranker.scores(query, &documents).await {
            Ok(scores) => scores,
            Err(error) => {
                tracing::warn!(%error, "the reranker failed; using the knowledge base's order");
                fallback()
            }
        }
    }

    /// Where belajar-server fetches a cited page: the whole book, opened at the page.
    pub(crate) fn page_url(&self, document_id: &str, page: Option<u32>) -> String {
        let fragment = page.map(|p| format!("#page={p}")).unwrap_or_default();
        format!(
            "{}/documents/{document_id}{fragment}",
            self.config.public_url
        )
    }
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/api/v1/mapel", get(mapel))
        .route("/api/v1/chat/completions", post(chat::chat))
        .route("/api/v1/practice/generate", post(practice::generate))
        .route("/api/v1/practice/grade", post(practice::grade))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_key,
        ))
        .with_state(state.clone());
    // Book files: no key, like the upstream's pre-signed page links. Bind to loopback.
    let files = Router::new()
        .route("/documents/{document_id}", get(pages::original))
        .with_state(state.clone());
    Router::new()
        .route("/health", get(health))
        .merge(api)
        .merge(files)
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "ok": true }))
}

async fn require_key(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let grade = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|key| state.config.client_keys.get(key.trim()).copied());
    let Some(grade) = grade else {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "invalid api key",
            "invalid_api_key",
        );
    };
    request.extensions_mut().insert(Grade(grade));
    next.run(request).await
}

async fn mapel(State(state): State<AppState>, request: Request) -> Response {
    let Some(Grade(grade)) = request.extensions().get::<Grade>().copied() else {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "invalid api key",
            "invalid_api_key",
        );
    };
    match state.catalog().await {
        Ok(catalog) => Json(
            catalog
                .mapel(grade)
                .into_iter()
                .map(|m| {
                    json!({
                        "mapel": m.name,
                        "knowledge_base_id": m.knowledge_base_ids.first().cloned().unwrap_or_default(),
                        "document_count": m.document_count,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(error) => {
            tracing::error!(%error, "cannot list the knowledge bases");
            error_response(StatusCode::BAD_GATEWAY, "the knowledge base is unavailable", "kb_unavailable")
        }
    }
}

/// `{"error": ...}` with a status, like the upstream's own error responses.
pub(crate) fn error_response(status: StatusCode, message: &str, code: &str) -> Response {
    (
        status,
        Json(json!({ "error": { "message": message, "type": "pilot_upstream", "code": code } })),
    )
        .into_response()
}

/// An SSE stream fed by a channel: the headers leave at once, frames as they are made, and a
/// `: ping` every few seconds while the slow parts (search, model) work.
pub(crate) fn sse_channel() -> (mpsc::Sender<Bytes>, Response) {
    let (tx, rx) = mpsc::channel::<Bytes>(64);
    let (out_tx, out_rx) = mpsc::channel::<Result<Bytes, Infallible>>(64);
    tokio::spawn(async move {
        let mut rx = rx;
        let mut ping = tokio::time::interval(Duration::from_secs(5));
        ping.tick().await;
        loop {
            tokio::select! {
                frame = rx.recv() => match frame {
                    Some(frame) => {
                        if out_tx.send(Ok(frame)).await.is_err() { return; }
                    }
                    None => return,
                },
                _ = ping.tick() => {
                    if out_tx.send(Ok(Bytes::from_static(b": ping\n\n"))).await.is_err() { return; }
                }
            }
        }
    });
    let body = axum::body::Body::from_stream(tokio_stream::wrappers::ReceiverStream::new(out_rx));
    let mut response = Response::new(body);
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    (tx, response)
}

pub(crate) fn data_frame(payload: &serde_json::Value) -> Bytes {
    Bytes::from(format!("data: {payload}\n\n"))
}

pub(crate) fn event_frame(name: &str, payload: &serde_json::Value) -> Bytes {
    Bytes::from(format!("event: {name}\ndata: {payload}\n\n"))
}

/// The stream's `event: error`, which belajar reads as an upstream failure.
pub(crate) fn error_frame(message: &str) -> Bytes {
    event_frame(
        "error",
        &json!({ "error": { "message": message, "type": "pilot_upstream", "code": "pipeline_failed" }, "status": 502 }),
    )
}
