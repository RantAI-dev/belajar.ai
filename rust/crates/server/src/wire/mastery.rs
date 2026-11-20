//! Mastery on the wire: the level and the weekly trend, both decided by `belajar_core::mastery`
//! and only named here. Latihan and Progres share these, so one topic has one label everywhere.

use belajar_core::mastery::{MasteryLevel, Trend};
use serde::Serialize;
use utoipa::ToSchema;

/// `not_started` (Belum mulai), `learning` (Sedang belajar), `almost` (Hampir) or `mastered`
/// (Dikuasai: at least 80% over at least 5 answered questions).
// Variant order is core's order, so `Ord` sorts from not started to mastered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MasteryLevelDto {
    NotStarted,
    Learning,
    Almost,
    Mastered,
}

impl MasteryLevelDto {
    /// Started and not yet mastered: what "Latih lagi" offers.
    #[must_use]
    pub const fn needs_practice(self) -> bool {
        matches!(self, Self::Learning | Self::Almost)
    }
}

impl From<MasteryLevel> for MasteryLevelDto {
    fn from(level: MasteryLevel) -> Self {
        match level {
            MasteryLevel::NotStarted => Self::NotStarted,
            MasteryLevel::Learning => Self::Learning,
            MasteryLevel::Almost => Self::Almost,
            MasteryLevel::Mastered => Self::Mastered,
        }
    }
}

/// This week's accuracy against last week's: `up`, `down`, `same`, or `no_data` when either week
/// has no answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrendDto {
    Up,
    Down,
    Same,
    NoData,
}

impl From<Trend> for TrendDto {
    fn from(trend: Trend) -> Self {
        match trend {
            Trend::Up => Self::Up,
            Trend::Down => Self::Down,
            Trend::Same => Self::Same,
            Trend::NoData => Self::NoData,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
mod tests {
    use belajar_core::mastery::{MasteryLevel, Trend};

    use super::{MasteryLevelDto, TrendDto};

    #[test]
    fn the_wire_names_match_the_core_names() {
        for level in [
            MasteryLevel::NotStarted,
            MasteryLevel::Learning,
            MasteryLevel::Almost,
            MasteryLevel::Mastered,
        ] {
            let wire = serde_json::to_value(MasteryLevelDto::from(level)).unwrap();
            assert_eq!(wire, level.as_str());
        }
        for trend in [Trend::Up, Trend::Down, Trend::Same, Trend::NoData] {
            let wire = serde_json::to_value(TrendDto::from(trend)).unwrap();
            assert_eq!(wire, trend.as_str());
        }
    }
}
