//! Fail-closed derived-sensitivity errors.

use std::fmt;

/// A fail-closed derived-sensitivity error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DerivedSensitivityError {
    /// Derivation was treated as declassification to public.
    DerivationIsNotDeclassification,
    /// Blanket PII masking was treated as a declassification grant.
    BlanketMaskIsNotAuthorization,
    /// Sensitivity slices were empty or length-mismatched.
    InvalidSensitivityPayload,
}

impl fmt::Display for DerivedSensitivityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::DerivationIsNotDeclassification => "derivation is not declassification to public",
            Self::BlanketMaskIsNotAuthorization => {
                "blanket PII masking is not declassification authorization"
            }
            Self::InvalidSensitivityPayload => "invalid derived-sensitivity payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for DerivedSensitivityError {}

#[cfg(test)]
mod tests {
    use super::DerivedSensitivityError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                DerivedSensitivityError::DerivationIsNotDeclassification,
                "derivation is not declassification to public",
            ),
            (
                DerivedSensitivityError::BlanketMaskIsNotAuthorization,
                "blanket PII masking is not declassification authorization",
            ),
            (
                DerivedSensitivityError::InvalidSensitivityPayload,
                "invalid derived-sensitivity payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
