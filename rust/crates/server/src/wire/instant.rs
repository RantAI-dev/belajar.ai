//! Instants on the wire: RFC 3339 in UTC with exactly three decimals, `2026-11-11T08:30:00.000Z`,
//! in a field ending `At`. Every DTO uses this module; later slices reuse it.

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Deserializer, Serializer};

#[must_use]
pub fn to_wire(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// The wire form back to an instant. `None` when the text is not one we issued, so a cursor that
/// fails here becomes the caller's 400, never a 500.
#[must_use]
pub fn from_wire(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub fn serialize<S: Serializer>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&to_wire(dt))
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<Utc>, D::Error> {
    let text = String::deserialize(deserializer)?;
    DateTime::parse_from_rfc3339(&text)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(serde::de::Error::custom)
}

/// The same format for a nullable instant field (`completedAt`), so a DTO can say
/// `Option<DateTime<Utc>>` instead of splitting the field in two.
pub mod option {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(
        value: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(instant) => super::serialize(instant, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        match Option::<String>::deserialize(deserializer)? {
            None => Ok(None),
            Some(text) => DateTime::parse_from_rfc3339(&text)
                .map(|dt| Some(dt.with_timezone(&Utc)))
                .map_err(serde::de::Error::custom),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
mod tests {
    use chrono::TimeZone;
    use chrono::Timelike as _;

    use super::*;

    #[test]
    fn the_format_is_utc_with_exactly_three_decimals() {
        let dt = Utc
            .with_ymd_and_hms(2026, 11, 11, 8, 30, 0)
            .unwrap()
            .with_nanosecond(400_000_000)
            .unwrap();
        assert_eq!(to_wire(&dt), "2026-11-11T08:30:00.400Z");
    }

    #[test]
    fn the_format_truncates_farther_digits() {
        let dt = Utc
            .with_ymd_and_hms(2026, 1, 2, 3, 4, 5)
            .unwrap()
            .with_nanosecond(999_999_999)
            .unwrap();
        assert_eq!(to_wire(&dt), "2026-01-02T03:04:05.999Z");
    }

    #[test]
    fn a_nullable_instant_is_the_same_format_or_null() {
        #[derive(serde::Serialize)]
        struct Nullable {
            #[serde(with = "super::option")]
            at: Option<DateTime<Utc>>,
        }
        let dt = Utc.with_ymd_and_hms(2026, 11, 11, 8, 30, 0).unwrap();
        assert_eq!(
            serde_json::to_value(Nullable { at: Some(dt) }).unwrap(),
            serde_json::json!({ "at": "2026-11-11T08:30:00.000Z" })
        );
        assert_eq!(
            serde_json::to_value(Nullable { at: None }).unwrap(),
            serde_json::json!({ "at": null })
        );
    }
}
