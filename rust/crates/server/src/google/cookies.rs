//! The two short-lived cookies of a Google sign-in. Each holds only a random value whose SHA-256
//! keys a server-side row, so the browser never sees state, nonce, verifier or any token.

/// Binds a sign-in attempt to the browser that started it.
pub const FLOW_COOKIE: &str = "belajar_google_flow";
/// Binds a verified Google account with no student yet to the browser picking the grade.
pub const SIGNUP_COOKIE: &str = "belajar_google_signup";
/// Ten minutes: the attempt and the pending signup expire with their cookie.
pub const MAX_AGE_SECONDS: i64 = 600;
/// Sent only to the Google routes. `SameSite=Lax` still sends it on Google's top-level redirect
/// back to the callback, which `Strict` would not.
const PATH: &str = "/v1/auth/google";

#[must_use]
pub fn set(name: &str, value: &str, secure: bool) -> String {
    format!(
        "{name}={value}; Path={PATH}; Max-Age={MAX_AGE_SECONDS}; HttpOnly; SameSite=Lax{}",
        if secure { "; Secure" } else { "" }
    )
}

#[must_use]
pub fn cleared(name: &str, secure: bool) -> String {
    format!(
        "{name}=; Path={PATH}; Max-Age=0; HttpOnly; SameSite=Lax{}",
        if secure { "; Secure" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flow_cookie_is_http_only_scoped_and_short_lived() {
        let cookie = set(FLOW_COOKIE, "abc", true);
        assert!(cookie.starts_with("belajar_google_flow=abc;"));
        for part in [
            "HttpOnly",
            "SameSite=Lax",
            "Path=/v1/auth/google",
            "Max-Age=600",
            "Secure",
        ] {
            assert!(cookie.contains(part), "{part} in {cookie}");
        }
        assert!(!set(FLOW_COOKIE, "abc", false).contains("Secure"));
        assert!(cleared(SIGNUP_COOKIE, false).contains("Max-Age=0"));
    }
}
