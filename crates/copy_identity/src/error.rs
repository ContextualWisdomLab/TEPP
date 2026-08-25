//! Fail-closed copy-identity errors.

use std::fmt;

/// A fail-closed copy-identity error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CopyIdentityError {
    /// A template copy was treated as the source document identity.
    CopyIsNotSourceIdentity,
    /// A template copy was treated as a state transition.
    CopyIsNotTransition,
    /// A recovery slice was empty or length-mismatched.
    InvalidCopyPayload,
}

impl fmt::Display for CopyIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::CopyIsNotSourceIdentity => "a template copy is not the source document identity",
            Self::CopyIsNotTransition => "a template copy is not a state transition",
            Self::InvalidCopyPayload => "invalid copy-identity payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CopyIdentityError {}

#[cfg(test)]
mod tests {
    use super::CopyIdentityError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                CopyIdentityError::CopyIsNotSourceIdentity,
                "a template copy is not the source document identity",
            ),
            (
                CopyIdentityError::CopyIsNotTransition,
                "a template copy is not a state transition",
            ),
            (
                CopyIdentityError::InvalidCopyPayload,
                "invalid copy-identity payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
