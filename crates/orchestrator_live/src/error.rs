//! Fail-closed orchestrator live-listener errors.

use std::fmt;

/// A fail-closed interpretation-listener contract error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OrchestratorLiveError {
    /// Malformed JSON, unknown fields, empty required values, or bound violations.
    InvalidWirePayload,
    /// Payload used a contract version this crate does not support.
    UnsupportedContractVersion,
    /// Request exceeded configured size, depth, or cardinality limits.
    LimitExceeded,
    /// Bind, header, or purpose authorization failed without leaking policy detail.
    AuthorizationDenied,
    /// LLM or orchestrator output was offered as scientific or statistical authority.
    ScientificAuthorityRefused,
}

impl fmt::Display for OrchestratorLiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidWirePayload => "invalid orchestrator wire payload",
            Self::UnsupportedContractVersion => "unsupported orchestrator contract version",
            Self::LimitExceeded => "orchestrator request exceeded configured limits",
            Self::AuthorizationDenied => "orchestrator authorization denied",
            Self::ScientificAuthorityRefused => "orchestrator output is not scientific authority",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for OrchestratorLiveError {}

#[cfg(test)]
mod tests {
    use super::OrchestratorLiveError;

    #[test]
    fn messages_are_stable_and_redacted() {
        assert_eq!(
            OrchestratorLiveError::InvalidWirePayload.to_string(),
            "invalid orchestrator wire payload"
        );
        assert_eq!(
            OrchestratorLiveError::UnsupportedContractVersion.to_string(),
            "unsupported orchestrator contract version"
        );
        assert_eq!(
            OrchestratorLiveError::LimitExceeded.to_string(),
            "orchestrator request exceeded configured limits"
        );
        assert_eq!(
            OrchestratorLiveError::AuthorizationDenied.to_string(),
            "orchestrator authorization denied"
        );
        assert_eq!(
            OrchestratorLiveError::ScientificAuthorityRefused.to_string(),
            "orchestrator output is not scientific authority"
        );
        assert!(
            !OrchestratorLiveError::AuthorizationDenied
                .to_string()
                .contains("token")
        );
    }
}
