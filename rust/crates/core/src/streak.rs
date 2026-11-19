//! The streak: consecutive active days, counted in Jakarta calendar days. The streak survives a
//! day that has not started yet — today missing keeps yesterday's run alive — and dies only when
//! a full day passes with nothing done.

use std::collections::BTreeSet;

use chrono::NaiveDate;

/// The number of consecutive active days ending today, or ending yesterday when today has no
/// activity yet (the streak is still alive). 0 when neither today nor yesterday is active.
#[must_use]
pub fn streak_days(active: &BTreeSet<NaiveDate>, today: NaiveDate) -> u32 {
    // Count from today when it is active; otherwise the streak may still be alive at yesterday.
    let start = if active.contains(&today) {
        today
    } else {
        match today.pred_opt() {
            Some(yesterday) if active.contains(&yesterday) => yesterday,
            _ => return 0,
        }
    };

    let mut count = 0_u32;
    let mut day = start;
    while active.contains(&day) {
        count += 1;
        // `pred_opt` is None at the smallest representable date: the loop simply ends there.
        match day.pred_opt() {
            Some(previous) => day = previous,
            None => break,
        }
    }
    count
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "tests")]
mod tests {
    use std::collections::BTreeSet;

    use chrono::NaiveDate;

    use super::streak_days;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("the test dates are valid")
    }

    fn days(specs: &[(i32, u32, u32)]) -> BTreeSet<NaiveDate> {
        specs.iter().map(|&(y, m, d)| date(y, m, d)).collect()
    }

    #[test]
    fn no_activity_is_no_streak() {
        assert_eq!(streak_days(&days(&[]), date(2026, 11, 19)), 0);
    }

    #[test]
    fn activity_only_today_counts_one() {
        assert_eq!(streak_days(&days(&[(2026, 11, 19)]), date(2026, 11, 19)), 1);
    }

    #[test]
    fn today_without_activity_keeps_yesterdays_streak_alive() {
        let active = days(&[(2026, 11, 17), (2026, 11, 18)]);
        assert_eq!(streak_days(&active, date(2026, 11, 19)), 2);
    }

    #[test]
    fn a_missed_day_kills_the_streak() {
        assert_eq!(
            streak_days(&days(&[(2026, 11, 17), (2026, 11, 18)]), date(2026, 11, 21)),
            0
        );
        assert_eq!(streak_days(&days(&[(2026, 11, 20)]), date(2026, 11, 21)), 1);
    }

    #[test]
    fn a_gap_inside_the_run_stops_the_count_at_the_gap() {
        // Active 17, 18 and 15: counting back from the 19th (yesterday alive) stops after two.
        let active = days(&[(2026, 11, 15), (2026, 11, 17), (2026, 11, 18)]);
        assert_eq!(streak_days(&active, date(2026, 11, 19)), 2);
    }

    #[test]
    fn activity_far_in_the_past_or_future_does_not_count() {
        assert_eq!(
            streak_days(&days(&[(2026, 11, 1), (2026, 10, 30)]), date(2026, 11, 19)),
            0
        );
    }
}
