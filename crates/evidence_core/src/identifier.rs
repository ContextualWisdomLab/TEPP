//! Stable, sortable identifiers for immutable source evidence.

use crate::EvidenceError;
use std::fmt;
use std::str::FromStr;
use uuid::{Uuid, Version};

/// A validated RFC 9562 UUID version 7 used to identify evidence-domain objects.
///
/// The wrapper prevents UUIDs from other versions from crossing the evidence
/// boundary accidentally. `UUIDv7` embeds a millisecond Unix timestamp for useful
/// database locality while retaining randomized uniqueness bits.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvidenceId(Uuid);

impl EvidenceId {
    /// Generate a new identifier from the current system clock and secure randomness.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Validate and wrap an existing UUID.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidEvidenceId`] when `value` is not `UUIDv7`.
    pub fn from_uuid(value: Uuid) -> Result<Self, EvidenceError> {
        if value.get_version() == Some(Version::SortRand) {
            Ok(Self(value))
        } else {
            Err(EvidenceError::InvalidEvidenceId)
        }
    }

    /// Return the validated UUID value.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EvidenceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EvidenceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for EvidenceId {
    type Err = EvidenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parsed = Uuid::parse_str(value).map_err(|_| EvidenceError::InvalidEvidenceId)?;
        Self::from_uuid(parsed)
    }
}
