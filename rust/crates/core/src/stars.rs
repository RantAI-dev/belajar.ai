//! A quiz result as stars. Pure arithmetic, so a better or worse retake is the same rule
//! everywhere: three for a perfect score, two for at least 60% correct, one for finishing.

/// Stars for one quiz result. Callers validate `0 <= correct <= total` and `total >= 1`; the
/// arithmetic itself stays safe for any pair of values.
#[must_use]
pub fn quiz_stars(correct: u32, total: u32) -> u8 {
    // u64 so no pair of inputs can overflow.
    if u64::from(correct) >= u64::from(total) {
        3
    } else if u64::from(correct) * 10 >= u64::from(total) * 6 {
        2
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::quiz_stars;

    #[test]
    fn a_perfect_score_is_three_stars() {
        for (correct, total) in [(5, 5), (1, 1), (20, 20)] {
            assert_eq!(quiz_stars(correct, total), 3, "{correct}/{total}");
        }
    }

    #[test]
    fn sixty_percent_or_better_is_two_stars() {
        for (correct, total) in [(3, 5), (12, 20), (6, 10), (2, 3)] {
            assert_eq!(quiz_stars(correct, total), 2, "{correct}/{total}");
        }
    }

    #[test]
    fn anything_less_is_one_star_even_zero_correct() {
        for (correct, total) in [(2, 5), (11, 20), (0, 5), (0, 1), (1, 2)] {
            assert_eq!(quiz_stars(correct, total), 1, "{correct}/{total}");
        }
    }
}
