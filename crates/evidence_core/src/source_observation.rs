//! Evidence-owned source-ingress observation clocks.

use crate::{ContentDigest, EvidenceError, EvidenceId, SourceArtifact};
use jiff::Timestamp;
use temporal_core::{AvailableTime, SystemTime};

/// Evidence-owned observation of one immutable source artifact entering TEPP.
///
/// Production creation reads the wall clock inside the Evidence boundary. The
/// same absolute ingress instant is represented separately as [`SystemTime`]
/// (when TEPP observed/recorded the source) and [`AvailableTime`] (when that
/// ingested evidence became available to TEPP analysis). Callers cannot supply
/// either clock. This record does not assert document/event/publication time,
/// external source authenticity, or protection against a compromised host
/// clock.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceObservation {
    source_artifact_id: EvidenceId,
    source_snapshot_sha256: ContentDigest,
    system_observed_at: SystemTime,
    available_at: AvailableTime,
}

impl SourceObservation {
    /// Observe an immutable source artifact at the Evidence ingress boundary.
    ///
    /// One owner-controlled wall-clock reading is converted into distinct
    /// system-time and availability-time domain values. Production callers have
    /// no API for selecting or backdating either timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] only if the trusted clock
    /// reading cannot be represented by TEPP's strict temporal contract.
    pub fn observe(source_artifact: &SourceArtifact) -> Result<Self, EvidenceError> {
        let observed_at = Timestamp::now().to_string();
        Self::from_trusted_timestamp(source_artifact, &observed_at)
    }

    /// Return the immutable Evidence source-artifact identity.
    #[must_use]
    pub const fn source_artifact_id(&self) -> EvidenceId {
        self.source_artifact_id
    }

    /// Return the immutable source-content SHA-256 observed at ingress.
    #[must_use]
    pub const fn source_snapshot_sha256(&self) -> ContentDigest {
        self.source_snapshot_sha256
    }

    /// Return the nominal system clock for the Evidence ingress observation.
    #[must_use]
    pub const fn system_observed_at(&self) -> SystemTime {
        self.system_observed_at
    }

    /// Return when the ingested evidence became available to TEPP analysis.
    #[must_use]
    pub const fn available_at(&self) -> AvailableTime {
        self.available_at
    }

    fn from_trusted_timestamp(
        source_artifact: &SourceArtifact,
        observed_at: &str,
    ) -> Result<Self, EvidenceError> {
        let system_observed_at = SystemTime::parse_rfc3339(observed_at)
            .map_err(|_| EvidenceError::InvalidWirePayload)?;
        let available_at = AvailableTime::parse_rfc3339(observed_at)
            .map_err(|_| EvidenceError::InvalidWirePayload)?;
        Ok(Self {
            source_artifact_id: source_artifact.id(),
            source_snapshot_sha256: source_artifact.content_digest(),
            system_observed_at,
            available_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SourceObservation;
    use crate::{EvidenceError, SourceArtifact};

    #[test]
    fn trusted_ingress_clock_preserves_distinct_nominal_clocks() {
        let artifact = SourceArtifact::from_bytes(b"snapshot").expect("artifact");
        let observation = SourceObservation::from_trusted_timestamp(
            &artifact,
            "2026-09-15T02:00:00.123456789Z",
        )
        .expect("observation");

        assert_eq!(observation.source_artifact_id(), artifact.id());
        assert_eq!(
            observation.source_snapshot_sha256(),
            artifact.content_digest()
        );
        assert_eq!(
            observation.system_observed_at().instant(),
            observation.available_at().instant()
        );
        assert_eq!(
            observation.system_observed_at().to_rfc3339(),
            "2026-09-15T02:00:00.123456789Z"
        );
    }

    #[test]
    fn invalid_trusted_clock_representation_fails_closed() {
        let artifact = SourceArtifact::from_bytes(b"snapshot").expect("artifact");
        assert_eq!(
            SourceObservation::from_trusted_timestamp(&artifact, "not-a-time"),
            Err(EvidenceError::InvalidWirePayload)
        );
    }
}
