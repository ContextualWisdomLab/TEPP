//! Fail-closed checkpoint-authority errors.

use std::fmt;

/// A fail-closed checkpoint-authority error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CheckpointAuthorityError {
    /// A checkpoint was treated as the CPU `f64` estimator.
    CheckpointIsNotEstimator,
    /// Artifact identity was missing or empty.
    MissingIdentity,
    /// Model-run provenance was missing or empty.
    MissingProvenance,
    /// Content digest was missing or empty.
    MissingDigest,
    /// Content digest was not canonical lowercase hex `SHA-256`.
    InvalidDigest,
    /// A recovery slice was empty or length-mismatched.
    InvalidAuthorityPayload,
}

impl fmt::Display for CheckpointAuthorityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::CheckpointIsNotEstimator => "a model checkpoint is not the cpu f64 estimator",
            Self::MissingIdentity => "checkpoint artifact is missing identity",
            Self::MissingProvenance => "checkpoint artifact is missing model-run provenance",
            Self::MissingDigest => "checkpoint artifact is missing content digest",
            Self::InvalidDigest => "checkpoint artifact digest is not canonical sha-256",
            Self::InvalidAuthorityPayload => "invalid checkpoint-authority payload",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CheckpointAuthorityError {}

#[cfg(test)]
mod tests {
    use super::CheckpointAuthorityError;

    #[test]
    fn error_messages_are_stable() {
        for (error, message) in [
            (
                CheckpointAuthorityError::CheckpointIsNotEstimator,
                "a model checkpoint is not the cpu f64 estimator",
            ),
            (
                CheckpointAuthorityError::MissingIdentity,
                "checkpoint artifact is missing identity",
            ),
            (
                CheckpointAuthorityError::MissingProvenance,
                "checkpoint artifact is missing model-run provenance",
            ),
            (
                CheckpointAuthorityError::MissingDigest,
                "checkpoint artifact is missing content digest",
            ),
            (
                CheckpointAuthorityError::InvalidDigest,
                "checkpoint artifact digest is not canonical sha-256",
            ),
            (
                CheckpointAuthorityError::InvalidAuthorityPayload,
                "invalid checkpoint-authority payload",
            ),
        ] {
            assert_eq!(error.to_string(), message);
        }
    }
}
