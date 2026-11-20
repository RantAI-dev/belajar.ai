//! The student's colour theme preference. `System` follows the device (prefers-color-scheme);
//! the web app resolves it, the server only stores the choice.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    /// The stored and wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// Exactly one of `system`, `light`, `dark`; anything else is `None`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preference_round_trips_through_its_spelling() {
        for theme in ThemePreference::ALL {
            assert_eq!(ThemePreference::parse(theme.as_str()), Some(theme));
        }
    }

    #[test]
    fn unknown_or_differently_cased_values_are_rejected() {
        for bad in ["", "Dark", "auto", "blue", " light"] {
            assert_eq!(ThemePreference::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn the_default_follows_the_device() {
        assert_eq!(ThemePreference::default(), ThemePreference::System);
    }
}
