//! The recorded responses of the real practice API (`fixtures/practice/`, see its README). The
//! fake replays the error ones as they are, and its tests hold everything it builds to their shape.

/// `event: error` after a `progress`: every generated question failed verification (502).
pub(crate) const GENERATION_FAILED_STREAM: &str =
    include_str!("../fixtures/practice/generate-error-generation-failed-stream.sse");

/// The same failure as a plain JSON body, for `stream: false`.
pub(crate) const GENERATION_FAILED_JSON: &str =
    include_str!("../fixtures/practice/generate-error-generation-failed.json");

/// The frames of a recorded SSE body, each with its blank-line terminator.
pub(crate) fn sse_frames(recorded: &str) -> Vec<bytes::Bytes> {
    recorded
        .split("\n\n")
        .filter(|block| !block.trim().is_empty())
        .map(|block| bytes::Bytes::from(format!("{block}\n\n")))
        .collect()
}

#[cfg(test)]
#[allow(clippy::panic, reason = "tests")]
pub(crate) fn json(name: &str) -> serde_json::Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/practice")
        .join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name}: {e}"))
}
