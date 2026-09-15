//! Evidence-owned source observation and availability clocks.

use crate::{ContentDigest, EvidenceError, EvidenceId, SourceArtifact};
use jiff::Timestamp;
use temporal_core::{AvailableTime, SystemTime};

/// Evidence-owned observation of one immutable source artifact entering TEPP.
///
/// Production creation reads the wall clock inside the Evidence boundary and
/// records only the nominal [`SystemTime`] at which TEPP observed the source.
/// Callers cannot select or backdate that clock. Observation does not itself
/// mean that the evidence is already available to an analyst or model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceObservation {
    source_artifact_id: EvidenceId,
    source_snapshot_sha256: ContentDigest,
    system_observed_at: SystemTime,
}

/// Evidence-owned availability of one previously observed source artifact.
///
/// Production creation takes a second owner-controlled wall-clock reading when
/// the observed source becomes available to TEPP analysis. System observation
/// and evidence availability therefore remain separate nominal clocks and may
/// differ. Availability may never precede the owning source observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceAvailability {
    source_artifact_id: EvidenceId,
    source_snapshot_sha256: ContentDigest,
    system_observed_at: SystemTime,
    available_at: AvailableTime,
}

impl SourceObservation {
    /// Observe an immutable source artifact at the Evidence ingress boundary.
    ///
    /// Production callers have no API for selecting or backdating the recorded
    /// system timestamp.
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

    /// Return the nominal system clock for the Evidence source observation.
    #[must_use]
    pub const fn system_observed_at(&self) -> SystemTime {
        self.system_observed_at
    }

    /// Build an observation from one owner-controlled timestamp.
    ///
    /// This helper stays private so deterministic tests can exercise temporal
    /// refusal branches without exposing a caller-selectable production clock.
    fn from_trusted_timestamp(
        source_artifact: &SourceArtifact,
        observed_at: &str,
    ) -> Result<Self, EvidenceError> {
        let system_observed_at = SystemTime::parse_rfc3339(observed_at)
            .map_err(|_| EvidenceError::InvalidWirePayload)?;
        Ok(Self {
            source_artifact_id: source_artifact.id(),
            source_snapshot_sha256: source_artifact.content_digest(),
            system_observed_at,
        })
    }
}

impl SourceAvailability {
    /// Mark an observed immutable source artifact available to TEPP analysis.
    ///
    /// A fresh Evidence-owned clock reading is used for availability. The
    /// operation fails closed if the trusted wall clock has moved behind the
    /// source observation, rather than fabricating an earlier availability.
    ///
    /// # Errors
    ///
    /// Returns [`EvidenceError::InvalidWirePayload`] when the trusted clock
    /// cannot be represented by TEPP's temporal contract or would place
    /// availability before the owning source observation.
    pub fn make_available(observation: &SourceObservation) -> Result<Self, EvidenceError> {
        let available_at = Timestamp::now().to_string();
        Self::from_trusted_timestamp(observation, &available_at)
    }

    /// Return the immutable Evidence source-artifact identity.
    #[must_use]
    pub const fn source_artifact_id(&self) -> EvidenceId {
        self.source_artifact_id
    }

    /// Return the immutable source-content SHA-256 made available to analysis.
    #[must_use]
    pub const fn source_snapshot_sha256(&self) -> ContentDigest {
        self.source_snapshot_sha256
    }

    /// Return the owning nominal system observation clock.
    #[must_use]
    pub const fn system_observed_at(&self) -> SystemTime {
        self.system_observed_at
    }

    /// Return the nominal time at which the evidence became available.
    #[must_use]
    pub const fn available_at(&self) -> AvailableTime {
        self.available_at
    }

    /// Build availability from one owner-controlled timestamp.
    ///
    /// This helper remains private so tests can exercise clock-ordering failure
    /// without adding a production API that permits caller-selected backdating.
    fn from_trusted_timestamp(
        observation: &SourceObservation,
        available_at: &str,
    ) -> Result<Self, EvidenceError> {
        let available_at = AvailableTime::parse_rfc3339(available_at)
            .map_err(|_| EvidenceError::InvalidWirePayload)?;
        if available_at.instant() < observation.system_observed_at().instant() {
            return Err(EvidenceError::InvalidWirePayload);
        }
        Ok(Self {
            source_artifact_id: observation.source_artifact_id(),
            source_snapshot_sha256: observation.source_snapshot_sha256(),
            system_observed_at: observation.system_observed_at(),
            available_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceAvailability, SourceObservation};
    use crate::{EvidenceError, SourceArtifact};

    fn observed_at(timestamp: &str) -> SourceObservation {
        let artifact = SourceArtifact::from_bytes(b"snapshot").expect("artifact");
        SourceObservation::from_trusted_timestamp(&artifact, timestamp).expect("observation")
    }

    #[test]
    fn trusted_observation_clock_binds_source_identity_and_content() {
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
            observation.system_observed_at().to_rfc3339(),
            "2026-09-15T02:00:00.123456789Z"
        );
    }

    #[test]
    fn availability_is_a_distinct_later_owner_clock() {
        let observation = observed_at("2026-09-15T02:00:00Z");
        let availability = SourceAvailability::from_trusted_timestamp(
            &observation,
            "2026-09-15T02:00:01Z",
        )
        .expect("availability");

        assert_eq!(
            availability.source_artifact_id(),
            observation.source_artifact_id()
        );
        assert_eq!(
            availability.source_snapshot_sha256(),
            observation.source_snapshot_sha256()
        );
        assert_eq!(
            availability.system_observed_at(),
            observation.system_observed_at()
        );
        assert!(availability.available_at().instant() > observation.system_observed_at().instant());
    }

    #[test]
    fn availability_before_observation_fails_closed() {
        let observation = observed_at("2026-09-15T02:00:01Z");
        assert_eq!(
            SourceAvailability::from_trusted_timestamp(
                &observation,
                "2026-09-15T02:00:00Z"
            ),
            Err(EvidenceError::InvalidWirePayload)
        );
    }

    #[test]
    fn invalid_trusted_clock_representations_fail_closed() {
        let artifact = SourceArtifact::from_bytes(b"snapshot").expect("artifact");
        assert_eq!(
            SourceObservation::from_trusted_timestamp(&artifact, "not-a-time"),
            Err(EvidenceError::InvalidWirePayload)
        );
        let observation = observed_at("2026-09-15T02:00:00Z");
        assert_eq!(
            SourceAvailability::from_trusted_timestamp(&observation, "not-a-time"),
            Err(EvidenceError::InvalidWirePayload)
        );
    }
}
