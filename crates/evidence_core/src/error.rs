//! Errors returned by immutable evidence-domain validation.

use std::fmt;

/// A fail-closed validation error for evidence identifiers and records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EvidenceError {
    /// The supplied identifier was malformed or was not an RFC 9562 UUIDv7.
    InvalidEvidenceId,
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEvidenceId => formatter.write_str("invalid evidence identifier"),
        }
    }
}

impl std::error::Error for EvidenceError {}
