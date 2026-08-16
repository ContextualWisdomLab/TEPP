//! Fail-closed membership-cutoff errors.

use std::fmt;

/// A fail-closed membership-cutoff error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MembershipCutoffError {
    /// Membership evidence became available after the knowledge cutoff.
    AvailabilityExceedsCutoff,
    /// An observation or recovery slice was empty or length-mismatched.
    InvalidEligibilityPayload,
}

impl fmt::Display for MembershipCutoffError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::AvailabilityExceedsCutoff => {
                "membership availability exceeds the knowledge cutoff"
            }
            Self::InvalidEligibilityPayload => "invalid membership-cutoff payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for MembershipCutoffError {}

#[cfg(test)]
mod tests {
    use super::MembershipCutoffError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                MembershipCutoffError::AvailabilityExceedsCutoff,
                "membership availability exceeds the knowledge cutoff",
            ),
            (
                MembershipCutoffError::InvalidEligibilityPayload,
                "invalid membership-cutoff payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
