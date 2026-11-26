//! `GET /documents/{id}`: a cited book, whole. belajar-server opens it at the `#page=` of the
//! link it was given. The first request downloads the book; later ones read the local copy.

use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::{AppState, error_response};

pub(crate) async fn original(
    State(state): State<AppState>,
    Path(document_id): Path<String>,
) -> Response {
    let (path, content_type) = match state.kb.original(&document_id).await {
        Ok(found) => found,
        Err(error) => {
            tracing::warn!(%error, document_id, "a book could not be fetched");
            return error_response(StatusCode::NOT_FOUND, "no such document", "not_found");
        }
    };
    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(error) => {
            tracing::error!(%error, "a cached book could not be opened");
            return error_response(StatusCode::NOT_FOUND, "no such document", "not_found");
        }
    };
    let body = axum::body::Body::from_stream(tokio_util_stream(file));
    let mut response = (StatusCode::OK, body).into_response();
    let content_type = HeaderValue::from_str(&content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, content_type);
    response
}

/// A file as a stream of chunks, without pulling in tokio-util for one call.
fn tokio_util_stream(
    file: tokio::fs::File,
) -> impl futures::Stream<Item = Result<bytes::Bytes, std::io::Error>> {
    futures::stream::unfold(file, |mut file| async move {
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0u8; 256 * 1024];
        match file.read(&mut buf).await {
            Ok(0) => None,
            Ok(n) => {
                buf.truncate(n);
                Some((Ok(bytes::Bytes::from(buf)), file))
            }
            Err(e) => Some((Err(e), file)),
        }
    })
}
