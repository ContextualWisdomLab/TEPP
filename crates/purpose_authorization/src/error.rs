//! Fail-closed purpose-authorization errors.

use std::fmt;

/// A fail-closed purpose-authorization error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PurposeAuthorizationError {
    /// A grant was used for a purpose it does not authorize.
    CrossPurposeUse,
    /// Blanket PII masking was offered as a substitute for authorization.
    BlanketMaskIsNotAuthorization,
    /// An unknown purpose wire name was supplied.
    UnknownPurpose,
    /// Purpose slices were empty or length-mismatched.
    InvalidPurposePayload,
}

impl fmt::Display for PurposeAuthorizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::CrossPurposeUse => "authorization grant used for a different purpose",
            Self::BlanketMaskIsNotAuthorization => "blanket mask is not authorization",
            Self::UnknownPurpose => "unknown processing purpose",
            Self::InvalidPurposePayload => "invalid purpose payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PurposeAuthorizationError {}

#[cfg(test)]
mod tests {
    use super::PurposeAuthorizationError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                PurposeAuthorizationError::CrossPurposeUse,
                "authorization grant used for a different purpose",
            ),
            (
                PurposeAuthorizationError::BlanketMaskIsNotAuthorization,
                "blanket mask is not authorization",
            ),
            (
                PurposeAuthorizationError::UnknownPurpose,
                "unknown processing purpose",
            ),
            (
                PurposeAuthorizationError::InvalidPurposePayload,
                "invalid purpose payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
