//! The field-error vocabulary every request shares: one entry per bad field, with the fixed codes
//! `too_short`, `too_long`, `invalid_format`, `out_of_range`.

use crate::api_error::FieldError;

pub(crate) fn too_short(field: &str, min: usize) -> FieldError {
    FieldError {
        field: field.into(),
        code: "too_short",
        message: format!("Must be at least {min} characters."),
    }
}

pub(crate) fn too_long(field: &str, max: usize) -> FieldError {
    FieldError {
        field: field.into(),
        code: "too_long",
        message: format!("Must be at most {max} characters."),
    }
}

/// `what` completes "Must be …", for example "one of our subject ids".
pub(crate) fn invalid_format(field: &str, what: &str) -> FieldError {
    FieldError {
        field: field.into(),
        code: "invalid_format",
        message: format!("Must be {what}."),
    }
}

pub(crate) fn out_of_range(field: &str, min: u8, max: u8) -> FieldError {
    FieldError {
        field: field.into(),
        code: "out_of_range",
        message: format!("Must be between {min} and {max}."),
    }
}

/// A count outside its bounds, with a message that reads like one: exact, bounded or "at least".
pub(crate) fn out_of_count(field: &str, min: usize, max: Option<usize>) -> FieldError {
    FieldError {
        field: field.into(),
        code: "out_of_range",
        message: match max {
            Some(max) if max == min => format!("Must be exactly {min}."),
            Some(max) => format!("Must be between {min} and {max}."),
            None => format!("Must be at least {min}."),
        },
    }
}

/// `too_short` when `len < min`, `too_long` when `len > max`, nothing in between.
pub(crate) fn check_length(
    errors: &mut Vec<FieldError>,
    field: &str,
    len: usize,
    min: usize,
    max: usize,
) {
    if len < min {
        errors.push(too_short(field, min));
    } else if len > max {
        errors.push(too_long(field, max));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "tests")]
#[allow(clippy::indexing_slicing, reason = "tests")]
mod tests {
    use super::*;

    #[test]
    fn count_messages_read_like_counts() {
        assert_eq!(out_of_count("/n", 1, Some(1)).message, "Must be exactly 1.");
        assert_eq!(
            out_of_count("/n", 1, Some(3)).message,
            "Must be between 1 and 3."
        );
        assert_eq!(
            out_of_count("limit", 1, None).message,
            "Must be at least 1."
        );
        assert_eq!(out_of_count("/n", 1, None).code, "out_of_range");
    }

    #[test]
    fn length_is_checked_after_trimming_by_the_caller() {
        let mut errors = Vec::new();
        check_length(&mut errors, "/topic", 0, 1, 100);
        assert_eq!(errors[0].code, "too_short");
        let mut errors = Vec::new();
        check_length(&mut errors, "/topic", 101, 1, 100);
        assert_eq!(errors[0].code, "too_long");
        let mut errors = Vec::new();
        check_length(&mut errors, "/topic", 100, 1, 100);
        assert!(errors.is_empty());
    }
}
