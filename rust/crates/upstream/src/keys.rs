//! Per-grade upstream keys. A key is a secret: it is never Debug-printed (the hand-written `Debug`
//! shows only which grade/assistant pairs exist) and never echoed by a `KeysError`.

use std::collections::HashMap;
use std::fmt;

use belajar_core::grade::Grade;
use belajar_ports::upstream::Assistant;

/// One upstream key per grade and assistant, parsed from comma-separated
/// `<grade>.<assistant>=<key>` entries, e.g. `7.ask=…,7.learn=…,7.practice=…`.
/// The empty string is valid and means nothing is configured.
#[derive(Clone, Default)]
pub struct UpstreamKeys {
    entries: HashMap<(u8, Assistant), String>,
}

#[derive(Debug, thiserror::Error)]
pub enum KeysError {
    /// The entry is not `<grade>.<assistant>=<key>`.
    #[error("upstream key entry {index}: expected <grade>.<assistant>=<key>")]
    Malformed { index: usize },
    /// The grade side is not a number from 1 to 12.
    #[error("upstream key entry {index}: the grade must be a number from 1 to 12")]
    BadGrade { index: usize },
    /// The assistant side is not `ask`, `learn` or `practice`.
    #[error("upstream key entry {index}: the assistant must be ask, learn or practice")]
    UnknownAssistant { index: usize },
    /// The key side is present but empty.
    #[error("upstream key entry {index}: the key is empty")]
    EmptyKey { index: usize },
}

impl UpstreamKeys {
    /// Parses the configured key list. Surrounding whitespace around entries and around the key
    /// itself is ignored. A later duplicate of the same grade/assistant wins.
    pub fn parse(raw: &str) -> Result<Self, KeysError> {
        let mut entries = HashMap::new();
        for (index, entry) in raw.split(',').enumerate() {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let (left, key) = entry
                .split_once('=')
                .ok_or(KeysError::Malformed { index })?;
            let key = key.trim();
            if key.is_empty() {
                return Err(KeysError::EmptyKey { index });
            }
            let (grade, assistant) = left.split_once('.').ok_or(KeysError::Malformed { index })?;
            let grade: u8 = grade
                .trim()
                .parse()
                .map_err(|_| KeysError::BadGrade { index })?;
            if !(Grade::MIN..=Grade::MAX).contains(&grade) {
                return Err(KeysError::BadGrade { index });
            }
            let assistant = match assistant.trim() {
                "ask" => Assistant::Ask,
                "learn" => Assistant::Learn,
                "practice" => Assistant::Practice,
                _ => return Err(KeysError::UnknownAssistant { index }),
            };
            entries.insert((grade, assistant), key.to_owned());
        }
        Ok(Self { entries })
    }

    pub fn key_for(&self, grade: Grade, assistant: Assistant) -> Option<&str> {
        self.entries
            .get(&(grade.get(), assistant))
            .map(String::as_str)
    }
}

impl fmt::Debug for UpstreamKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Only the pairs, never the key material, so a config dump cannot leak a secret.
        let mut pairs: Vec<(u8, &'static str)> = self
            .entries
            .keys()
            .map(|(grade, assistant)| (*grade, assistant_name(*assistant)))
            .collect();
        pairs.sort_unstable();
        let mut set = f.debug_set();
        for (grade, assistant) in pairs {
            set.entry(&format_args!("{grade}.{assistant}"));
        }
        set.finish()
    }
}

fn assistant_name(assistant: Assistant) -> &'static str {
    match assistant {
        Assistant::Ask => "ask",
        Assistant::Learn => "learn",
        Assistant::Practice => "practice",
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests"
    )]

    use super::*;

    fn keys() -> UpstreamKeys {
        UpstreamKeys::parse("7.ask=fake-7-ask, 7.learn = fake-7-learn ,7.practice=fake-7-practice")
            .unwrap()
    }

    #[test]
    fn parses_all_three_assistants_ignoring_whitespace() {
        let keys = keys();
        let grade = Grade::new(7).unwrap();
        assert_eq!(keys.key_for(grade, Assistant::Ask), Some("fake-7-ask"));
        assert_eq!(keys.key_for(grade, Assistant::Learn), Some("fake-7-learn"));
        assert_eq!(
            keys.key_for(grade, Assistant::Practice),
            Some("fake-7-practice")
        );
        assert_eq!(keys.key_for(Grade::new(8).unwrap(), Assistant::Ask), None);
    }

    #[test]
    fn the_empty_string_is_valid_and_configures_nothing() {
        let keys = UpstreamKeys::parse("").unwrap();
        assert_eq!(keys.key_for(Grade::new(1).unwrap(), Assistant::Ask), None);
        assert_eq!(format!("{keys:?}"), "{}");
    }

    #[test]
    fn later_duplicates_win() {
        let keys = UpstreamKeys::parse("7.ask=first,7.ask=second").unwrap();
        assert_eq!(
            keys.key_for(Grade::new(7).unwrap(), Assistant::Ask),
            Some("second")
        );
    }

    #[test]
    fn rejects_malformed_and_out_of_range_entries() {
        // "7.ask=x.y" is absent on purpose: only the first `=` splits, and a key may contain
        // dots or further equals.
        for bad in ["7.ask", "13.ask=x", "7.teacher=x", "x.ask=y", "7.ask=", "7"] {
            assert!(
                UpstreamKeys::parse(bad).is_err(),
                "{bad} should be rejected"
            );
        }
    }

    #[test]
    fn error_messages_never_carry_the_key_material() {
        // A key that is part of a malformed entry must not surface in the error text.
        let error = UpstreamKeys::parse("13.ask=super-secret-key").unwrap_err();
        assert!(!error.to_string().contains("super-secret-key"));
    }

    #[test]
    fn debug_shows_pairs_but_never_the_keys() {
        let printed = format!("{:?}", keys());
        assert!(printed.contains("7.ask"), "pairs appear: {printed}");
        assert!(!printed.contains("fake-"), "no key material: {printed}");
    }
}
