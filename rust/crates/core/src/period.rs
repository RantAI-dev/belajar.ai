//! The time windows Progres counts in: the Jakarta week (Monday 00:00 to the next Monday 00:00
//! WIB), the week before it, and the last N Jakarta calendar days including today. Every window
//! is half-open, `[start, end)`, so an instant belongs to exactly one week.

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};

use crate::calendar::{jakarta_date, jakarta_day_start};

/// A half-open span of time, `start <= t < end`, plus the Jakarta dates it covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    /// The first Jakarta date in the window.
    pub first_day: NaiveDate,
    /// The last Jakarta date in the window (inclusive).
    pub last_day: NaiveDate,
}

impl Window {
    fn days(first_day: NaiveDate, day_count: u32) -> Self {
        let last_day = first_day + Duration::days(i64::from(day_count.max(1)) - 1);
        Self {
            start: jakarta_day_start(first_day),
            end: jakarta_day_start(last_day + Duration::days(1)),
            first_day,
            last_day,
        }
    }

    /// Whether `instant` falls inside the window.
    #[must_use]
    pub fn contains(&self, instant: DateTime<Utc>) -> bool {
        self.start <= instant && instant < self.end
    }

    /// Every Jakarta date in the window, oldest first.
    #[must_use]
    pub fn dates(&self) -> Vec<NaiveDate> {
        self.first_day
            .iter_days()
            .take_while(|day| *day <= self.last_day)
            .collect()
    }
}

/// The Jakarta week `now` falls in: Monday 00:00 WIB up to the next Monday 00:00 WIB.
#[must_use]
pub fn this_week(now: DateTime<Utc>) -> Window {
    let today = jakarta_date(now);
    let monday = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
    Window::days(monday, 7)
}

/// The Jakarta week before `this_week(now)`.
#[must_use]
pub fn last_week(now: DateTime<Utc>) -> Window {
    Window::days(this_week(now).first_day - Duration::days(7), 7)
}

/// The last `day_count` Jakarta calendar days, today included (`day_count` 0 is treated as 1).
#[must_use]
pub fn last_days(now: DateTime<Utc>, day_count: u32) -> Window {
    let today = jakarta_date(now);
    let first = today - Duration::days(i64::from(day_count.max(1)) - 1);
    Window::days(first, day_count)
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "tests")]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .expect("the test instants parse")
            .with_timezone(&Utc)
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    #[test]
    fn the_week_runs_monday_to_monday_in_jakarta() {
        // Thursday 2026-11-19, 10:00 WIB.
        let week = this_week(at("2026-11-19T03:00:00Z"));
        assert_eq!(week.first_day, date(2026, 11, 16), "Monday");
        assert_eq!(week.last_day, date(2026, 11, 22), "Sunday");
        assert_eq!(week.start, at("2026-11-15T17:00:00Z"), "Monday 00:00 WIB");
        assert_eq!(
            week.end,
            at("2026-11-22T17:00:00Z"),
            "next Monday 00:00 WIB"
        );
        assert_eq!(week.dates().len(), 7);
    }

    #[test]
    fn sunday_23_59_is_still_this_week_and_monday_00_00_starts_the_next() {
        // Sunday 2026-11-22 23:59 WIB = 16:59 UTC.
        let sunday_late = at("2026-11-22T16:59:00Z");
        assert_eq!(this_week(sunday_late).first_day, date(2026, 11, 16));
        assert!(this_week(sunday_late).contains(sunday_late));
        // Monday 2026-11-23 00:00 WIB = Sunday 17:00 UTC.
        let monday_midnight = at("2026-11-22T17:00:00Z");
        let next = this_week(monday_midnight);
        assert_eq!(next.first_day, date(2026, 11, 23));
        assert_eq!(next.start, monday_midnight);
        assert!(!this_week(sunday_late).contains(monday_midnight));
        assert_eq!(last_week(monday_midnight), this_week(sunday_late));
    }

    #[test]
    fn a_utc_sunday_evening_is_already_monday_in_jakarta() {
        // 2026-11-22 (a Sunday in UTC) 20:00Z is Monday 03:00 WIB: the new week.
        let instant = at("2026-11-22T20:00:00Z");
        assert_eq!(this_week(instant).first_day, date(2026, 11, 23));
        assert_eq!(last_week(instant).first_day, date(2026, 11, 16));
    }

    #[test]
    fn last_week_is_the_seven_days_before_this_week() {
        let now = at("2026-11-19T03:00:00Z");
        let last = last_week(now);
        assert_eq!(last.first_day, date(2026, 11, 9));
        assert_eq!(last.last_day, date(2026, 11, 15));
        assert_eq!(
            last.end,
            this_week(now).start,
            "the weeks touch, never overlap"
        );
    }

    #[test]
    fn last_n_days_include_today_in_jakarta() {
        // 2026-11-19T18:00Z is already 2026-11-20 01:00 WIB.
        let now = at("2026-11-19T18:00:00Z");
        let seven = last_days(now, 7);
        assert_eq!(seven.last_day, date(2026, 11, 20));
        assert_eq!(seven.first_day, date(2026, 11, 14));
        assert_eq!(seven.dates().len(), 7);
        assert!(seven.contains(now));
        assert_eq!(seven.end, at("2026-11-20T17:00:00Z"));
        let thirty = last_days(now, 30);
        assert_eq!(thirty.first_day, date(2026, 10, 22));
        assert_eq!(thirty.dates().len(), 30);
        assert_eq!(last_days(now, 0).dates(), vec![date(2026, 11, 20)]);
    }
}
