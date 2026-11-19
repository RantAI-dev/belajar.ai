//! The list cursor on the wire: base64url (no padding) of `<epoch microseconds>|<uuid>`, opaque
//! to the client and safe unencoded in a query string. Microsecond precision keeps the keyset
//! comparison exact for rows written within the same millisecond.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use belajar_ports::conversations::PageCursor;

/// The wire form of a keyset cursor.
#[must_use]
pub fn encode(cursor: PageCursor) -> String {
    let plain = format!("{}|{}", cursor.at.timestamp_micros(), cursor.id);
    URL_SAFE_NO_PAD.encode(plain)
}

/// `None` when the text is not a cursor we ever issued: the caller answers 400, not 500.
#[must_use]
pub fn decode(text: &str) -> Option<PageCursor> {
    let plain = URL_SAFE_NO_PAD.decode(text).ok()?;
    let plain = String::from_utf8(plain).ok()?;
    let (at, id) = plain.split_once('|')?;
    let micros = at.parse::<i64>().ok()?;
    Some(PageCursor {
        at: chrono::DateTime::from_timestamp_micros(micros)?,
        id: uuid::Uuid::parse_str(id).ok()?,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use chrono::{TimeZone, Timelike, Utc};

    use super::*;

    #[test]
    fn the_cursor_round_trips_with_sub_millisecond_precision() {
        // The database keeps microseconds, so that is the precision a cursor must survive.
        let at = Utc
            .timestamp_opt(1_763_500_000, 0)
            .unwrap()
            .with_nanosecond(123_456_000)
            .unwrap();
        let cursor = PageCursor {
            at,
            id: "0193b2c4-7a1e-7c3f-9d2a-5e8f1b6c4d90".parse().unwrap(),
        };
        let wire = encode(cursor);
        assert!(!wire.contains(['+', '/', '=']));
        assert_eq!(
            decode(&wire),
            Some(cursor),
            "not a single microsecond is lost"
        );
    }

    #[test]
    fn anything_we_never_issued_decodes_to_none() {
        for text in [
            "",
            "!!!",
            "123|not-a-uuid",
            "aGk=",
            "only-one-part",
            "falsch",
        ] {
            assert_eq!(decode(text), None, "{text} is not a cursor");
        }
    }
}
