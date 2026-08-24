//! Fail-closed payload-bound errors.

use std::fmt;

/// A fail-closed payload-bound error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PayloadBoundError {
    /// A maximum size or depth of zero was configured.
    InvalidBound,
    /// Identity was missing or empty.
    MissingIdentity,
    /// Provenance was missing or empty.
    MissingProvenance,
    /// The payload exceeded the configured byte bound.
    PayloadTooLarge,
    /// The payload exceeded the configured nesting-depth bound.
    PayloadTooDeep,
    /// A recovery slice was empty or length-mismatched.
    InvalidPayloadDecision,
}

impl fmt::Display for PayloadBoundError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidBound => "payload bound must be positive",
            Self::MissingIdentity => "untrusted payload is missing identity",
            Self::MissingProvenance => "untrusted payload is missing provenance",
            Self::PayloadTooLarge => "untrusted payload exceeds the byte bound",
            Self::PayloadTooDeep => "untrusted payload exceeds the depth bound",
            Self::InvalidPayloadDecision => "invalid payload-bound decision payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PayloadBoundError {}

#[cfg(test)]
mod tests {
    use super::PayloadBoundError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                PayloadBoundError::InvalidBound,
                "payload bound must be positive",
            ),
            (
                PayloadBoundError::MissingIdentity,
                "untrusted payload is missing identity",
            ),
            (
                PayloadBoundError::MissingProvenance,
                "untrusted payload is missing provenance",
            ),
            (
                PayloadBoundError::PayloadTooLarge,
                "untrusted payload exceeds the byte bound",
            ),
            (
                PayloadBoundError::PayloadTooDeep,
                "untrusted payload exceeds the depth bound",
            ),
            (
                PayloadBoundError::InvalidPayloadDecision,
                "invalid payload-bound decision payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
