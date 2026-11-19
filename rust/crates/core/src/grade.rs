use std::fmt;

/// A school grade, 1 to 12 (SD 1–6, SMP 7–9, SMA/SMK 10–12). Raw input becomes a `Grade` once,
/// at the edge, so the rest of the code never handles an out-of-range number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Grade(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradeError {
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Sd,
    Smp,
    Sma,
}

impl Grade {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 12;

    pub const fn new(value: u8) -> Result<Self, GradeError> {
        if value >= Self::MIN && value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(GradeError::OutOfRange)
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    #[must_use]
    pub const fn level(self) -> Level {
        match self.0 {
            1..=6 => Level::Sd,
            7..=9 => Level::Smp,
            _ => Level::Sma,
        }
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_every_grade_from_one_to_twelve() {
        for value in Grade::MIN..=Grade::MAX {
            assert_eq!(Grade::new(value).map(Grade::get), Ok(value));
        }
    }

    #[test]
    fn rejects_zero_and_thirteen() {
        assert_eq!(Grade::new(0), Err(GradeError::OutOfRange));
        assert_eq!(Grade::new(13), Err(GradeError::OutOfRange));
    }

    #[test]
    fn maps_each_grade_to_its_school_level() {
        let level = |v| Grade::new(v).map(Grade::level);
        assert_eq!(level(6), Ok(Level::Sd));
        assert_eq!(level(7), Ok(Level::Smp));
        assert_eq!(level(9), Ok(Level::Smp));
        assert_eq!(level(10), Ok(Level::Sma));
    }
}
