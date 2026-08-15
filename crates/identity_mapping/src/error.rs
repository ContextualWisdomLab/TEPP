//! Fail-closed identity-mapping errors.

use std::fmt;

/// A fail-closed identity-mapping error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IdentityMappingError {
    /// An analytical purpose tried to export source identities.
    UnauthorizedReidentification,
    /// Blanket PII masking was treated as re-identification authorization.
    BlanketMaskIsNotAuthorization,
    /// Mapping slices were empty or length-mismatched.
    InvalidMappingPayload,
}

impl fmt::Display for IdentityMappingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnauthorizedReidentification => {
                "analytical purpose cannot export source identities"
            }
            Self::BlanketMaskIsNotAuthorization => {
                "blanket PII masking is not re-identification authorization"
            }
            Self::InvalidMappingPayload => "invalid identity-mapping payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for IdentityMappingError {}

#[cfg(test)]
mod tests {
    use super::IdentityMappingError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                IdentityMappingError::UnauthorizedReidentification,
                "analytical purpose cannot export source identities",
            ),
            (
                IdentityMappingError::BlanketMaskIsNotAuthorization,
                "blanket PII masking is not re-identification authorization",
            ),
            (
                IdentityMappingError::InvalidMappingPayload,
                "invalid identity-mapping payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
