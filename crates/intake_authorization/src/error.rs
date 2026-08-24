//! Fail-closed intake-authorization errors.

use std::fmt;

/// A fail-closed intake-authorization error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IntakeAuthorizationError {
    /// Intake was attempted without a purpose-bound grant.
    MissingGrant,
    /// Size, identity, or provenance bounds were treated as authorization.
    BoundsAreNotAuthorization,
    /// A recovery slice was empty or length-mismatched.
    InvalidIntakePayload,
}

impl fmt::Display for IntakeAuthorizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingGrant => "untrusted intake requires a purpose-bound grant",
            Self::BoundsAreNotAuthorization => {
                "identity, provenance, size, and depth bounds are not authorization"
            }
            Self::InvalidIntakePayload => "invalid intake-authorization payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for IntakeAuthorizationError {}

#[cfg(test)]
mod tests {
    use super::IntakeAuthorizationError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                IntakeAuthorizationError::MissingGrant,
                "untrusted intake requires a purpose-bound grant",
            ),
            (
                IntakeAuthorizationError::BoundsAreNotAuthorization,
                "identity, provenance, size, and depth bounds are not authorization",
            ),
            (
                IntakeAuthorizationError::InvalidIntakePayload,
                "invalid intake-authorization payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
