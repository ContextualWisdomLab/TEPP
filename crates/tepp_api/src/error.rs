//! Fail-closed API wire and contract errors.

use std::fmt;

/// A fail-closed service/API contract error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ApiError {
    /// Malformed JSON, unknown fields, empty required values, or bound violations.
    InvalidWirePayload,
    /// Payload used a contract version this crate does not support.
    UnsupportedContractVersion,
    /// Request exceeded configured size, depth, or cardinality limits.
    LimitExceeded,
    /// Tenant, purpose, or export authorization failed without leaking policy detail.
    AuthorizationDenied,
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidWirePayload => "invalid API wire payload",
            Self::UnsupportedContractVersion => "unsupported API contract version",
            Self::LimitExceeded => "API request exceeded configured limits",
            Self::AuthorizationDenied => "API authorization denied",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::ApiError;

    #[test]
    fn messages_are_stable_and_redacted() {
        assert_eq!(
            ApiError::InvalidWirePayload.to_string(),
            "invalid API wire payload"
        );
        assert_eq!(
            ApiError::UnsupportedContractVersion.to_string(),
            "unsupported API contract version"
        );
        assert_eq!(
            ApiError::LimitExceeded.to_string(),
            "API request exceeded configured limits"
        );
        assert_eq!(
            ApiError::AuthorizationDenied.to_string(),
            "API authorization denied"
        );
        assert!(!ApiError::AuthorizationDenied.to_string().contains("token"));
    }
}
