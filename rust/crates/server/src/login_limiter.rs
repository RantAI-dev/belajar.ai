//! The login limiter: 5 failed logins for one (lowercased) username within 15 minutes lock it
//! until 15 minutes after the first failure of the window. In memory only; one process is enough
//! for this app's traffic, and the acceptance test says an in-process lock suffices.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use belajar_ports::clock::Clock;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

const MAX_FAILURES: usize = 5;
const WINDOW: Duration = Duration::minutes(15);

pub struct LoginLimiter {
    clock: Arc<dyn Clock>,
    failures: Mutex<HashMap<String, VecDeque<DateTime<Utc>>>>,
}

impl LoginLimiter {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            failures: Mutex::new(HashMap::new()),
        }
    }

    /// Whole seconds until the username is unlocked, when it is locked. At least 1, so a caller
    /// never sees a `Retry-After: 0`.
    pub fn locked_remaining(&self, username: &str) -> Option<u64> {
        let now = self.clock.now();
        let mut failures = self.locked_failures(username, now)?;
        // The unlock is 15 minutes after the oldest failure still in the window.
        let first = failures.pop_front()?;
        let unlock = first + WINDOW;
        u64::try_from((unlock - now).num_seconds().max(1)).ok()
    }

    /// A failed login (unknown username or wrong password) joins the window. Refused attempts
    /// while locked never get this far, so a lock always drains.
    pub fn record_failure(&self, username: &str) {
        let now = self.clock.now();
        let mut map = self.failures.lock().unwrap_or_else(|p| p.into_inner());
        let failures = map.entry(username.to_owned()).or_default();
        prune(failures, now);
        failures.push_back(now);
    }

    /// A successful login clears the username's failures.
    pub fn clear(&self, username: &str) {
        let mut map = self.failures.lock().unwrap_or_else(|p| p.into_inner());
        map.remove(username);
    }

    fn locked_failures(
        &self,
        username: &str,
        now: DateTime<Utc>,
    ) -> Option<VecDeque<DateTime<Utc>>> {
        let mut map = self.failures.lock().unwrap_or_else(|p| p.into_inner());
        let failures = map.get_mut(username)?;
        prune(failures, now);
        (failures.len() >= MAX_FAILURES).then(|| failures.clone())
    }
}

fn prune(failures: &mut VecDeque<DateTime<Utc>>, now: DateTime<Utc>) {
    let cutoff = now - WINDOW;
    while failures.front().is_some_and(|ts| *ts <= cutoff) {
        failures.pop_front();
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "tests")]
mod tests {
    use std::sync::Arc;

    use belajar_ports::clock::Clock;
    use chrono::{DateTime, TimeZone, Utc};

    use super::*;

    /// A clock a test moves by hand.
    struct FixedClock(Mutex<DateTime<Utc>>);
    impl FixedClock {
        fn new(at: DateTime<Utc>) -> Arc<Self> {
            Arc::new(Self(Mutex::new(at)))
        }
        fn set(&self, at: DateTime<Utc>) {
            *self.0.lock().unwrap_or_else(|p| p.into_inner()) = at;
        }
    }
    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            *self.0.lock().unwrap_or_else(|p| p.into_inner())
        }
    }

    fn limiter(at: DateTime<Utc>) -> (LoginLimiter, Arc<FixedClock>) {
        let clock = FixedClock::new(at);
        (LoginLimiter::new(clock.clone()), clock)
    }

    #[test]
    fn five_failures_lock_until_fifteen_minutes_after_the_first() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let (limiter, _clock) = limiter(start);
        for _ in 0..4 {
            limiter.record_failure("budi");
            assert!(
                limiter.locked_remaining("budi").is_none(),
                "four is not locked"
            );
        }
        limiter.record_failure("budi");
        let remaining = limiter.locked_remaining("budi").expect("five locks");
        // Four failures at t=0, one more at t=0: the window starts at t=0, so unlock at t=15min.
        assert_eq!(remaining, 900);
        assert!(
            limiter.locked_remaining("siti").is_none(),
            "other usernames go free"
        );
    }

    #[test]
    fn the_window_rolls_and_the_lock_drains() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let (limiter, clock) = limiter(start);
        // Five failures spread over more than one window: only the last five 15-minute-window
        // failures count.
        for _ in 0..5 {
            limiter.record_failure("budi");
        }
        assert!(limiter.locked_remaining("budi").is_some());
        // Nothing new arrives; the failures age out one by one.
        clock.set(start + Duration::minutes(14));
        limiter.record_failure("budi"); // still locked inside the window
        assert!(limiter.locked_remaining("budi").is_some());
        clock.set(start + Duration::minutes(16));
        // The first failures fell out of the window, so the lock drained.
        assert!(limiter.locked_remaining("budi").is_none());
    }

    #[test]
    fn a_success_clears_the_history() {
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let (limiter, _clock) = limiter(start);
        for _ in 0..5 {
            limiter.record_failure("budi");
        }
        assert!(limiter.locked_remaining("budi").is_some());
        limiter.clear("budi");
        assert!(limiter.locked_remaining("budi").is_none());
    }

    #[test]
    fn a_lock_on_the_edge_reports_at_least_one_second() {
        // The first failure of the window sits 14:59.5 before the check: the raw remainder is
        // half a second, and `Retry-After` still says 1.
        let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let (limiter, clock) = limiter(start);
        for _ in 0..5 {
            limiter.record_failure("budi");
        }
        clock.set(start + Duration::seconds(899) + Duration::milliseconds(500));
        assert_eq!(limiter.locked_remaining("budi"), Some(1));
    }
}
