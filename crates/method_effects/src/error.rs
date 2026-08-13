//! Fail-closed method-effect errors.

use std::fmt;

/// A fail-closed method-effect error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MethodEffectsError {
    /// A method source was used as an inferential topic weight.
    MethodSourceIsNotInferentialWeight,
    /// An unknown method-source wire name was supplied.
    UnknownMethodSource,
    /// Source-label slices were empty or length-mismatched.
    InvalidSourcePayload,
}

impl fmt::Display for MethodEffectsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MethodSourceIsNotInferentialWeight => {
                "method source is not an inferential weight"
            }
            Self::UnknownMethodSource => "unknown method source",
            Self::InvalidSourcePayload => "invalid method-source payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for MethodEffectsError {}

#[cfg(test)]
mod tests {
    use super::MethodEffectsError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                MethodEffectsError::MethodSourceIsNotInferentialWeight,
                "method source is not an inferential weight",
            ),
            (
                MethodEffectsError::UnknownMethodSource,
                "unknown method source",
            ),
            (
                MethodEffectsError::InvalidSourcePayload,
                "invalid method-source payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
