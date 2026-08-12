//! Fail-closed validation metric errors.

use std::fmt;

/// A fail-closed validation-domain error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ValidationError {
    /// Empty, unequal-length, or non-finite input vectors.
    InvalidInput,
    /// Acceptance thresholds or Monte Carlo settings were inconsistent.
    InvalidConfiguration,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidInput => "invalid validation input",
            Self::InvalidConfiguration => "invalid validation configuration",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::ValidationError;

    #[test]
    fn messages_are_stable() {
        assert_eq!(
            ValidationError::InvalidInput.to_string(),
            "invalid validation input"
        );
        assert_eq!(
            ValidationError::InvalidConfiguration.to_string(),
            "invalid validation configuration"
        );
    }
}
