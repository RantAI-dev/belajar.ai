//! One place that turns an `UpstreamError` into our error. The upstream's own text goes to the
//! log under the request id; the body carries only the registry's message.

use axum::response::Response;
use belajar_ports::upstream::UpstreamError;

use crate::api_error::ApiError;
use crate::error_codes::ErrorCode;

/// Logs the cause once and returns the error a client sees.
pub fn api_error(err: &UpstreamError, request_id: &str) -> ApiError {
    let code = match err {
        UpstreamError::NotConfigured { .. } | UpstreamError::Unavailable(_) => {
            ErrorCode::Unavailable
        }
        UpstreamError::Timeout => ErrorCode::Timeout,
        UpstreamError::Failed { .. } | UpstreamError::Protocol(_) => ErrorCode::UpstreamFailed,
    };
    tracing::error!(request_id, code = code.as_str(), error = %err, "upstream call failed");
    let error = ApiError::new(code);
    if code == ErrorCode::Unavailable {
        error.retry_after(30)
    } else {
        error
    }
}

pub fn into_response(err: &UpstreamError, request_id: &str) -> Response {
    api_error(err, request_id).into_response(request_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use belajar_ports::upstream::Assistant;

    #[test]
    fn each_upstream_failure_has_one_code_and_no_upstream_text() {
        let cases = [
            (
                UpstreamError::NotConfigured {
                    grade: 8,
                    assistant: Assistant::Ask,
                },
                "unavailable",
            ),
            (UpstreamError::Unavailable("down".into()), "unavailable"),
            (UpstreamError::Timeout, "timeout"),
            (
                UpstreamError::Failed {
                    status: 400,
                    detail: "secret detail".into(),
                },
                "upstream_failed",
            ),
            (
                UpstreamError::Protocol("bad json".into()),
                "upstream_failed",
            ),
        ];
        for (err, code) in cases {
            let body = api_error(&err, "req-1").body("req-1");
            assert_eq!(body.error.code, code);
            assert_eq!(body.error.request_id, "req-1");
            assert!(!body.error.message.contains("secret") && !body.error.message.contains("json"));
        }
    }
}
