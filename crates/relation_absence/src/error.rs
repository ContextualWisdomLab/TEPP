//! Fail-closed relation-absence errors.

use std::fmt;

/// A fail-closed relation-absence error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RelationAbsenceError {
    /// An unobserved pair was treated as evidence of no relationship.
    AbsenceIsNotNegative,
    /// A recovery slice was empty or length-mismatched.
    InvalidObservationPayload,
}

impl fmt::Display for RelationAbsenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::AbsenceIsNotNegative => {
                "unobserved relation pairs are not evidence of no relationship"
            }
            Self::InvalidObservationPayload => "invalid relation-absence payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for RelationAbsenceError {}

#[cfg(test)]
mod tests {
    use super::RelationAbsenceError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                RelationAbsenceError::AbsenceIsNotNegative,
                "unobserved relation pairs are not evidence of no relationship",
            ),
            (
                RelationAbsenceError::InvalidObservationPayload,
                "invalid relation-absence payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
