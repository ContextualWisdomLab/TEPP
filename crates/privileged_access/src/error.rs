//! Fail-closed privileged-access errors.

use std::fmt;

/// A fail-closed privileged-access error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PrivilegedAccessError {
    /// Source identity was supplied to the privileged-access audit log.
    SourceIdentityNotAuditable,
    /// Blanket PII masking was treated as an audit grant.
    BlanketMaskIsNotAuthorization,
    /// Audit slices were empty or length-mismatched.
    InvalidAuditPayload,
}

impl fmt::Display for PrivilegedAccessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SourceIdentityNotAuditable => {
                "source identity cannot appear in a privileged-access audit"
            }
            Self::BlanketMaskIsNotAuthorization => {
                "blanket PII masking is not privileged-access authorization"
            }
            Self::InvalidAuditPayload => "invalid privileged-access audit payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PrivilegedAccessError {}

#[cfg(test)]
mod tests {
    use super::PrivilegedAccessError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                PrivilegedAccessError::SourceIdentityNotAuditable,
                "source identity cannot appear in a privileged-access audit",
            ),
            (
                PrivilegedAccessError::BlanketMaskIsNotAuthorization,
                "blanket PII masking is not privileged-access authorization",
            ),
            (
                PrivilegedAccessError::InvalidAuditPayload,
                "invalid privileged-access audit payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
