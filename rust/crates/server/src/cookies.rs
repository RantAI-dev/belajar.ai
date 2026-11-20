//! The session cookie, built in one place: name, attributes and both directions of parsing.

use axum::http::{HeaderMap, header};

pub const COOKIE_NAME: &str = "belajar_session";
/// 30 days.
pub const SESSION_MAX_AGE_SECONDS: u64 = 2_592_000;

#[must_use]
pub fn session_cookie(token: &str, secure: bool) -> String {
    format!(
        "{COOKIE_NAME}={token}; Path=/; Max-Age={SESSION_MAX_AGE_SECONDS}; HttpOnly; SameSite=Lax{}",
        if secure { "; Secure" } else { "" }
    )
}

#[must_use]
pub fn cleared_cookie(secure: bool) -> String {
    format!(
        "{COOKIE_NAME}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{}",
        if secure { "; Secure" } else { "" }
    )
}

/// The `belajar_session` token from a request's `Cookie` headers, if it is a non-empty one.
#[must_use]
pub fn token_from_headers(headers: &HeaderMap) -> Option<String> {
    value_from_headers(headers, COOKIE_NAME)
}

/// The value of the cookie `name` from a request's `Cookie` headers, if it is a non-empty one.
#[must_use]
pub fn value_from_headers(headers: &HeaderMap, name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    for value in headers.get_all(header::COOKIE) {
        let Some(raw) = value.to_str().ok() else {
            continue;
        };
        for pair in raw.split(';') {
            if let Some(token) = pair.trim().strip_prefix(&prefix)
                && !token.is_empty()
            {
                return Some(token.to_owned());
            }
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    fn headers(cookie: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.append(header::COOKIE, HeaderValue::from_str(cookie).unwrap());
        map
    }

    #[test]
    fn a_session_cookie_carries_every_attribute() {
        let cookie = session_cookie("abc123", false);
        assert!(cookie.starts_with("belajar_session=abc123"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("Max-Age=2592000"));
        assert!(!cookie.contains("Secure"), "off when configured off");
        assert!(session_cookie("t", true).contains("; Secure"));
    }

    #[test]
    fn a_cleared_cookie_empties_the_token_and_the_age() {
        let cookie = cleared_cookie(true);
        assert!(cookie.starts_with("belajar_session="));
        assert!(cookie.contains("Max-Age=0"));
        assert!(cookie.contains("Secure"));
    }

    #[test]
    fn the_token_comes_back_from_any_position_in_the_cookie_header() {
        assert_eq!(
            token_from_headers(&headers("belajar_session=tok")),
            Some("tok".into())
        );
        assert_eq!(
            token_from_headers(&headers("other=x; belajar_session=tok; more=y")),
            Some("tok".into())
        );
        assert_eq!(token_from_headers(&headers("belajar_session=")), None);
        assert_eq!(token_from_headers(&headers("other=x")), None);
        assert_eq!(token_from_headers(&HeaderMap::new()), None);
    }
}
