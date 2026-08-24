//! Fail-closed provider-receipt errors.

use std::fmt;

/// A fail-closed provider-receipt error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProviderReceiptError {
    /// Raw source text was supplied to a provider receipt.
    SourceTextNotDisclosable,
    /// Source identity was supplied to a provider receipt.
    SourceIdentityNotDisclosable,
    /// Blanket PII masking was treated as a disclosure grant.
    BlanketMaskIsNotAuthorization,
    /// A receipt or recovery slice was empty or length-mismatched.
    InvalidReceiptPayload,
}

impl fmt::Display for ProviderReceiptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::SourceTextNotDisclosable => "source text cannot appear in a provider receipt",
            Self::SourceIdentityNotDisclosable => {
                "source identity cannot appear in a provider receipt"
            }
            Self::BlanketMaskIsNotAuthorization => {
                "blanket PII masking is not provider-disclosure authorization"
            }
            Self::InvalidReceiptPayload => "invalid provider-receipt payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ProviderReceiptError {}

#[cfg(test)]
mod tests {
    use super::ProviderReceiptError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                ProviderReceiptError::SourceTextNotDisclosable,
                "source text cannot appear in a provider receipt",
            ),
            (
                ProviderReceiptError::SourceIdentityNotDisclosable,
                "source identity cannot appear in a provider receipt",
            ),
            (
                ProviderReceiptError::BlanketMaskIsNotAuthorization,
                "blanket PII masking is not provider-disclosure authorization",
            ),
            (
                ProviderReceiptError::InvalidReceiptPayload,
                "invalid provider-receipt payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
