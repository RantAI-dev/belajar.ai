//! The student's calendar day. "Today" everywhere in the app (missions, streak) is a Jakarta
//! calendar date: fixed UTC+07:00, no DST, so the rule is one shift, not a time zone database.

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

/// Jakarta is seven hours ahead of UTC, always: no DST since 1964.
const OFFSET_HOURS: i64 = 7;

/// The Jakarta calendar date an instant falls on. `2026-11-18T17:30:00Z` is `2026-11-19`: it is
/// already the 19th, half past midnight WIB.
#[must_use]
pub fn jakarta_date(instant: DateTime<Utc>) -> NaiveDate {
    (instant + Duration::hours(OFFSET_HOURS)).date_naive()
}

/// Midnight in Jakarta of `date`, as a UTC instant: the first instant of that calendar day.
/// `jakarta_day_start(jakarta_date(t)) <= t` always holds.
#[must_use]
pub fn jakarta_day_start(date: NaiveDate) -> DateTime<Utc> {
    // `NaiveTime::MIN` is 00:00:00, so the pair is always a valid datetime.
    let midnight = NaiveDateTime::new(date, NaiveTime::MIN) - Duration::hours(OFFSET_HOURS);
    Utc.from_utc_datetime(&midnight)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::expect_used, reason = "tests")]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("the test instants parse")
            .with_timezone(&Utc)
    }

    #[test]
    fn the_jakarta_day_turns_at_five_pm_utc() {
        // 17:00 UTC is midnight WIB: the day boundary an hour before Jakarta's midnight.
        assert_eq!(
            jakarta_date(at("2026-11-18T16:59:59Z")),
            NaiveDate::from_ymd_opt(2026, 11, 18).expect("valid date")
        );
        assert_eq!(
            jakarta_date(at("2026-11-18T17:00:00Z")),
            NaiveDate::from_ymd_opt(2026, 11, 19).expect("valid date")
        );
    }

    #[test]
    fn a_morning_utc_instant_is_the_same_jakarta_day() {
        // 03:00 UTC is 10:00 WIB, still the same date.
        assert_eq!(
            jakarta_date(at("2026-11-19T03:00:00Z")),
            NaiveDate::from_ymd_opt(2026, 11, 19).expect("valid date")
        );
    }

    #[test]
    fn a_day_starts_at_jakarta_midnight_and_covers_its_own_date() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 19).expect("valid date");
        let start = jakarta_day_start(date);
        assert_eq!(start, at("2026-11-18T17:00:00Z"));
        for (instant, expected) in [
            (start, true),
            (start - chrono::Duration::milliseconds(1), false),
            (
                start + chrono::Duration::hours(24) - chrono::Duration::milliseconds(1),
                true,
            ),
            (start + chrono::Duration::hours(24), false),
        ] {
            assert_eq!(jakarta_date(instant) == date, expected, "at {instant}");
        }
    }
}
