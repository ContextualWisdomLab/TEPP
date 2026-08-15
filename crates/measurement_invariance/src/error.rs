//! Fail-closed measurement-invariance errors.

use std::fmt;

/// A fail-closed measurement-invariance error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum InvarianceError {
    /// A weaker invariance status was treated as shared metric meaning.
    InvarianceTooWeakForSharedMetricMeaning,
    /// An unknown invariance-status wire name was supplied.
    UnknownInvarianceLevel,
    /// Loading slices were empty, identity-mismatched, length-mismatched, or non-finite.
    InvalidLoadingPayload,
}

impl fmt::Display for InvarianceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvarianceTooWeakForSharedMetricMeaning => {
                "invariance is too weak for shared metric meaning"
            }
            Self::UnknownInvarianceLevel => "unknown invariance level",
            Self::InvalidLoadingPayload => "invalid invariance loading payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for InvarianceError {}

#[cfg(test)]
mod tests {
    use super::InvarianceError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                InvarianceError::InvarianceTooWeakForSharedMetricMeaning,
                "invariance is too weak for shared metric meaning",
            ),
            (
                InvarianceError::UnknownInvarianceLevel,
                "unknown invariance level",
            ),
            (
                InvarianceError::InvalidLoadingPayload,
                "invalid invariance loading payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
