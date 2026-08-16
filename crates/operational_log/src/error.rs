//! Fail-closed operational-log errors.

use std::fmt;

/// A fail-closed operational-log error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OperationalLogError {
    /// Raw source text was supplied to the operational log.
    SourceTextNotLoggable,
    /// Source identity was supplied to the operational log.
    SourceIdentityNotLoggable,
    /// Blanket PII masking was treated as a log grant.
    BlanketMaskIsNotAuthorization,
    /// Log slices were empty or length-mismatched.
    InvalidLogPayload,
}

impl fmt::Display for OperationalLogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SourceTextNotLoggable => "source text cannot appear in an operational log",
            Self::SourceIdentityNotLoggable => {
                "source identity cannot appear in an operational log"
            }
            Self::BlanketMaskIsNotAuthorization => {
                "blanket PII masking is not operational-log authorization"
            }
            Self::InvalidLogPayload => "invalid operational-log payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for OperationalLogError {}

#[cfg(test)]
mod tests {
    use super::OperationalLogError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                OperationalLogError::SourceTextNotLoggable,
                "source text cannot appear in an operational log",
            ),
            (
                OperationalLogError::SourceIdentityNotLoggable,
                "source identity cannot appear in an operational log",
            ),
            (
                OperationalLogError::BlanketMaskIsNotAuthorization,
                "blanket PII masking is not operational-log authorization",
            ),
            (
                OperationalLogError::InvalidLogPayload,
                "invalid operational-log payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
