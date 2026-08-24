//! Fail-closed membership-target errors.

use std::fmt;

/// A fail-closed membership-target error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MembershipTargetError {
    /// A typed target was collapsed into another kind.
    TargetKindCollapsed,
    /// A recovery slice was empty or length-mismatched.
    InvalidTargetPayload,
}

impl fmt::Display for MembershipTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TargetKindCollapsed => "membership target kind cannot collapse into another kind",
            Self::InvalidTargetPayload => "invalid membership-target payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for MembershipTargetError {}

#[cfg(test)]
mod tests {
    use super::MembershipTargetError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                MembershipTargetError::TargetKindCollapsed,
                "membership target kind cannot collapse into another kind",
            ),
            (
                MembershipTargetError::InvalidTargetPayload,
                "invalid membership-target payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
