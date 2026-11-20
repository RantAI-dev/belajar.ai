//! Topic mastery: the percentage a student sees and the level it earns. One rule for every
//! screen (Latihan's "Penguasaanmu" and Progres), so a topic never has two different labels.

/// Below this many answered questions a topic cannot be `Mastered`, however good the score:
/// one lucky 4/4 set is not mastery yet.
pub const MIN_ANSWERS_FOR_MASTERED: u32 = 5;

/// How well a student knows a topic, from never practised to mastered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MasteryLevel {
    /// "Belum mulai": no answered question yet.
    NotStarted,
    /// "Sedang belajar".
    Learning,
    /// "Hampir".
    Almost,
    /// "Dikuasai".
    Mastered,
}

impl MasteryLevel {
    /// The wire value (lower `snake_case`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::Learning => "learning",
            Self::Almost => "almost",
            Self::Mastered => "mastered",
        }
    }
}

/// Round(100 × correct / attempts), half up, in integers; 0 when nothing was answered.
#[must_use]
pub fn mastery_percent(correct: u32, attempts: u32) -> u32 {
    if attempts == 0 {
        return 0;
    }
    // u64 so no pair of inputs can overflow; `correct` above `attempts` is capped at 100%.
    let correct = u64::from(correct.min(attempts));
    let attempts = u64::from(attempts);
    let rounded = (200 * correct + attempts) / (2 * attempts);
    u32::try_from(rounded).unwrap_or(100)
}

/// The level for a topic's answers. Thresholds, on the rounded percent the student sees (so the
/// label never disagrees with the number next to it):
/// no answers → NotStarted; ≥ 80% with at least `MIN_ANSWERS_FOR_MASTERED` answers → Mastered;
/// ≥ 60% (or ≥ 80% on fewer answers) → Almost; anything else → Learning.
#[must_use]
pub fn mastery_level(correct: u32, attempts: u32) -> MasteryLevel {
    if attempts == 0 {
        return MasteryLevel::NotStarted;
    }
    let percent = mastery_percent(correct, attempts);
    if percent >= 80 && attempts >= MIN_ANSWERS_FOR_MASTERED {
        MasteryLevel::Mastered
    } else if percent >= 60 {
        MasteryLevel::Almost
    } else {
        MasteryLevel::Learning
    }
}

/// This week's accuracy on a topic compared with last week's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    /// "Naik".
    Up,
    /// "Turun".
    Down,
    /// "Tetap".
    Same,
    /// One of the two weeks has no answers, so there is nothing to compare.
    NoData,
}

impl Trend {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Same => "same",
            Self::NoData => "no_data",
        }
    }
}

/// A change smaller than this many percentage points is "tetap": one question more or less in
/// a ten-question set should not flip the arrow.
pub const TREND_DEAD_BAND_POINTS: u32 = 5;

/// `(correct, attempts)` for this week and last week. Either week without answers is `NoData`.
#[must_use]
pub fn trend(this_week: (u32, u32), last_week: (u32, u32)) -> Trend {
    let (this_correct, this_attempts) = this_week;
    let (last_correct, last_attempts) = last_week;
    if this_attempts == 0 || last_attempts == 0 {
        return Trend::NoData;
    }
    let now = mastery_percent(this_correct, this_attempts);
    let before = mastery_percent(last_correct, last_attempts);
    if now >= before + TREND_DEAD_BAND_POINTS {
        Trend::Up
    } else if before >= now + TREND_DEAD_BAND_POINTS {
        Trend::Down
    } else {
        Trend::Same
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_rounds_half_up_and_survives_zero_attempts() {
        assert_eq!(mastery_percent(4, 5), 80);
        assert_eq!(mastery_percent(1, 3), 33);
        assert_eq!(mastery_percent(2, 3), 67);
        assert_eq!(mastery_percent(5, 5), 100);
        assert_eq!(mastery_percent(0, 7), 0);
        assert_eq!(mastery_percent(0, 0), 0);
        assert_eq!(mastery_percent(1, 8), 13, "12.5 rounds up");
        assert_eq!(mastery_percent(9, 5), 100, "never above 100");
        assert_eq!(mastery_percent(u32::MAX, u32::MAX), 100);
    }

    #[test]
    fn nothing_answered_is_not_started() {
        assert_eq!(mastery_level(0, 0), MasteryLevel::NotStarted);
    }

    #[test]
    fn eighty_percent_over_enough_answers_is_mastered() {
        assert_eq!(MIN_ANSWERS_FOR_MASTERED, 5);
        assert_eq!(mastery_level(4, 5), MasteryLevel::Mastered);
        assert_eq!(mastery_level(5, 5), MasteryLevel::Mastered);
        assert_eq!(mastery_level(8, 10), MasteryLevel::Mastered);
        assert_eq!(mastery_level(80, 100), MasteryLevel::Mastered);
    }

    #[test]
    fn too_few_answers_cannot_be_mastered_however_good() {
        assert_eq!(mastery_level(4, 4), MasteryLevel::Almost);
        assert_eq!(mastery_level(1, 1), MasteryLevel::Almost);
        assert_eq!(mastery_level(3, 3), MasteryLevel::Almost);
    }

    #[test]
    fn sixty_to_seventy_nine_percent_is_almost() {
        assert_eq!(mastery_level(3, 5), MasteryLevel::Almost);
        assert_eq!(mastery_level(6, 10), MasteryLevel::Almost);
        assert_eq!(mastery_level(7, 10), MasteryLevel::Almost);
        // 79.4% rounds to 79: just below the line.
        assert_eq!(mastery_level(27, 34), MasteryLevel::Almost);
    }

    #[test]
    fn below_sixty_percent_is_learning_even_zero() {
        assert_eq!(mastery_level(2, 5), MasteryLevel::Learning);
        assert_eq!(mastery_level(0, 5), MasteryLevel::Learning);
        assert_eq!(mastery_level(59, 100), MasteryLevel::Learning);
        assert_eq!(mastery_level(0, 1), MasteryLevel::Learning);
    }

    #[test]
    fn levels_order_from_not_started_to_mastered() {
        assert!(MasteryLevel::NotStarted < MasteryLevel::Learning);
        assert!(MasteryLevel::Learning < MasteryLevel::Almost);
        assert!(MasteryLevel::Almost < MasteryLevel::Mastered);
        assert_eq!(MasteryLevel::Mastered.as_str(), "mastered");
        assert_eq!(MasteryLevel::NotStarted.as_str(), "not_started");
    }

    #[test]
    fn trend_needs_both_weeks() {
        assert_eq!(trend((3, 5), (0, 0)), Trend::NoData);
        assert_eq!(trend((0, 0), (3, 5)), Trend::NoData);
        assert_eq!(trend((0, 0), (0, 0)), Trend::NoData);
    }

    #[test]
    fn trend_compares_this_week_with_last_week() {
        assert_eq!(trend((4, 5), (2, 5)), Trend::Up);
        assert_eq!(trend((2, 5), (4, 5)), Trend::Down);
        assert_eq!(trend((4, 5), (8, 10)), Trend::Same);
        // Within the dead band: 64% vs 60% is "tetap", 65% vs 60% is "naik".
        assert_eq!(trend((16, 25), (3, 5)), Trend::Same);
        assert_eq!(trend((13, 20), (3, 5)), Trend::Up);
        assert_eq!(trend((3, 5), (13, 20)), Trend::Down);
        assert_eq!(Trend::NoData.as_str(), "no_data");
    }
}
