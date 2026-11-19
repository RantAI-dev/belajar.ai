//! The SSE plumbing shared by the two streaming calls.
//!
//! eventsource-stream swallows comment lines, as the SSE spec says a `: ping` is not an event, so a
//! ping can only keep the timeout alive if we see its bytes. [`ActivityStream`] therefore counts
//! every chunk the upstream sends while we wait for the next event, and the driver pushes its
//! deadline back whenever that counter moves — a comment, and more forgivingly the bytes of a
//! partially delivered event, both count as life.

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::Duration;

use belajar_ports::upstream::UpstreamError;
use eventsource_stream::{EventStreamError, Eventsource};
use futures::stream::{Stream, StreamExt};

/// What one parsed SSE event turns into.
pub(crate) enum SseStep<T> {
    /// Yield these items; keep reading the stream.
    Emit(Vec<T>),
    /// Yield these items, then the stream ends without an error.
    Finish(Vec<T>),
    /// Yield this error, then the stream ends.
    Fail(UpstreamError),
}

/// Drives one upstream SSE response: parses events, translates them, ends the stream after its
/// final item or its first error, and answers `Timeout` when nothing arrives in time.
pub(crate) fn spawn_sse<T, F>(
    response: reqwest::Response,
    timeout: Duration,
    truncated: UpstreamError,
    mut translate: F,
) -> tokio::sync::mpsc::Receiver<Result<T, UpstreamError>>
where
    T: Send + 'static,
    F: FnMut(eventsource_stream::Event) -> SseStep<T> + Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    tokio::spawn(async move {
        let activity = Arc::new(AtomicU64::new(0));
        let events = ActivityStream::new(response.bytes_stream(), activity.clone()).eventsource();
        tokio::pin!(events);

        let mut seen_activity = 0u64;
        let mut deadline = tokio::time::Instant::now() + timeout;
        loop {
            // `events` is pinned once; each iteration re-projects it so the future we race
            // against the deadline borrows a stable place and not a statement temporary.
            let mut events_pin = events.as_mut();
            let sleep = tokio::time::sleep_until(deadline);
            tokio::select! {
                item = events_pin.next() => {
                    match item {
                        None => {
                            // The stream cut off without its final event; that is not a clean end.
                            let _ = tx.send(Err(truncated)).await;
                            return;
                        }
                        Some(Err(error)) => {
                            let _ = tx.send(Err(stream_error(error))).await;
                            return;
                        }
                        Some(Ok(event)) => {
                            deadline = tokio::time::Instant::now() + timeout;
                            match translate(event) {
                                SseStep::Emit(items) => {
                                    for item in items {
                                        if tx.send(Ok(item)).await.is_err() {
                                            return; // the consumer is gone; stop reading
                                        }
                                    }
                                }
                                SseStep::Finish(items) => {
                                    for item in items {
                                        if tx.send(Ok(item)).await.is_err() {
                                            return;
                                        }
                                    }
                                    return;
                                }
                                SseStep::Fail(error) => {
                                    let _ = tx.send(Err(error)).await;
                                    return;
                                }
                            }
                        }
                    }
                }
                _ = sleep => {
                    let current = activity.load(Ordering::Relaxed);
                    if current != seen_activity {
                        // Only bytes arrived (a ping, or half an event); that is still life.
                        seen_activity = current;
                        deadline = tokio::time::Instant::now() + timeout;
                        continue;
                    }
                    let _ = tx.send(Err(UpstreamError::Timeout)).await;
                    return;
                }
            }
        }
    });
    rx
}

fn stream_error(error: EventStreamError<reqwest::Error>) -> UpstreamError {
    match error {
        EventStreamError::Utf8(_) => UpstreamError::Protocol(
            "the upstream sent an event stream that is not valid UTF-8".into(),
        ),
        EventStreamError::Parser(_) => {
            UpstreamError::Protocol("the upstream sent a malformed event stream".into())
        }
        EventStreamError::Transport(error) => {
            if error.is_timeout() {
                UpstreamError::Timeout
            } else {
                UpstreamError::Unavailable(format!("the upstream stream failed: {error}"))
            }
        }
    }
}

/// A byte-stream wrapper that counts each chunk it hands to the SSE parser.
struct ActivityStream<S> {
    inner: S,
    activity: Arc<AtomicU64>,
}

impl<S> ActivityStream<S> {
    fn new(inner: S, activity: Arc<AtomicU64>) -> Self {
        Self { inner, activity }
    }
}

impl<S, B, E> Stream for ActivityStream<S>
where
    S: Stream<Item = Result<B, E>> + Unpin,
{
    type Item = Result<B, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match Pin::new(&mut this.inner).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                this.activity.fetch_add(1, Ordering::Relaxed);
                Poll::Ready(Some(item))
            }
            other => other,
        }
    }
}
