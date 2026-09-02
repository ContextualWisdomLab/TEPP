//! Digest-bound subevent-containment refusals as an analysis-run profile.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subevent_containment::{EventInterval, SubeventContainmentError, refuse_escaped_subevent};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, format_digest, require_receipt_identity,
    valid_identifier,
};

/// Versioned schema for a completed subevent-containment artifact.
pub const SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION: &str = "tepp.subevent_containment.v1";
/// Model contract required by the subevent-containment execution path.
pub const SUBEVENT_CONTAINMENT_MODEL_CONTRACT_VERSION: &str = "subevent_containment_v1";
/// Analysis-run output profile required for a subevent-containment artifact.
pub const SUBEVENT_CONTAINMENT_OUTPUT_PROFILE: &str = "subevent_containment_v1";
/// Maximum canonical artifact JSON size.
pub const SUBEVENT_CONTAINMENT_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const SUBEVENT_CONTAINMENT_INFERENCE_STATUS: &str =
    "subevent_interval_cannot_escape_parent_interval";

/// One cutoff-admitted child interval against a parent interval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubeventContainmentAssignment {
    assignment_id: String,
    parent: EventInterval,
    child: EventInterval,
    available_time: AvailableTime,
}

impl SubeventContainmentAssignment {
    /// Construct a bounded subevent-containment assignment.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the assignment
    /// identity is empty or oversized.
    pub fn new(
        assignment_id: impl Into<String>,
        parent: EventInterval,
        child: EventInterval,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let assignment_id = assignment_id.into();
        if !valid_identifier(&assignment_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            assignment_id,
            parent,
            child,
            available_time,
        })
    }

    /// Return the opaque assignment identity.
    #[must_use]
    pub fn assignment_id(&self) -> &str {
        &self.assignment_id
    }

    /// Return the parent event-time interval.
    #[must_use]
    pub const fn parent(&self) -> EventInterval {
        self.parent
    }

    /// Return the child subevent interval.
    #[must_use]
    pub const fn child(&self) -> EventInterval {
        self.child
    }

    /// Return the availability time used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded subevent-containment census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubeventContainmentArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used to admit assignments.
    pub knowledge_cutoff: String,
    /// Number of assignments admitted at the cutoff.
    pub assignment_count: u64,
    /// Assignments contained in their parent interval.
    pub contained_count: u64,
    /// Assignments that escaped their parent interval.
    pub escaped_count: u64,
    /// Escaped assignments refused as subevent-outside-parent.
    pub refused_as_escape_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl SubeventContainmentArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidSubeventContainmentArtifact`] when
    /// the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > SUBEVENT_CONTAINMENT_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidSubeventContainmentArtifact)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize canonical validated artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation, serialization, or size failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        if payload.len() > SUBEVENT_CONTAINMENT_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        Ok(payload)
    }

    /// Return the lowercase SHA-256 digest of canonical artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation or serialization failure.
    pub fn sha256(&self) -> Result<String, AnalysisEngineError> {
        self.to_json()
            .map(|json| format_digest(Sha256::digest(json.into_bytes())))
    }

    fn validate(&self) -> Result<(), AnalysisEngineError> {
        let status_sum = self.contained_count.checked_add(self.escaped_count);
        if self.schema_version != SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.assignment_count < 2
            || self.assignment_count > MAX_EVIDENCE_UNITS as u64
            || self.contained_count == 0
            || self.escaped_count == 0
            || status_sum != Some(self.assignment_count)
            || self.refused_as_escape_count != self.escaped_count
            || self.inference_status != SUBEVENT_CONTAINMENT_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidSubeventContainmentArtifact);
        }
        Ok(())
    }
}

/// One completed subevent-containment artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct SubeventContainmentExecution {
    /// Digest-bound completed subevent-containment census.
    pub artifact: SubeventContainmentArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe subevent-containment refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_escaped_subevent`] already on protected
/// main. Contained children stay attachments. Escaped children stay refusals,
/// never episode-membership windows. It does not emit
/// `identity_recovery_rate`, a `scientific_acceptance` inspect metric, GPU
/// kernels, MCMC, or topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-class corpus, inverted or escaped subevent treated as success,
/// duplicate assignment identity, oversized corpus, or invalid artifact
/// error.
pub fn execute_subevent_containment_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    assignments: &[SubeventContainmentAssignment],
    completed_at: impl Into<String>,
) -> Result<SubeventContainmentExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != SUBEVENT_CONTAINMENT_MODEL_CONTRACT_VERSION
        || request.output_profile != SUBEVENT_CONTAINMENT_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if assignments.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let (contained_count, escaped_count, refused_as_escape_count) =
        census_assignments(assignments, knowledge_cutoff)?;
    let assignment_count = contained_count
        .checked_add(escaped_count)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if assignment_count < 2
        || contained_count == 0
        || escaped_count == 0
        || refused_as_escape_count != escaped_count
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = SubeventContainmentArtifact {
        schema_version: SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        assignment_count,
        contained_count,
        escaped_count,
        refused_as_escape_count,
        inference_status: SUBEVENT_CONTAINMENT_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "subevent_containment",
        assignment_count,
        4,
        "validated",
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("subevent_containment_artifact_{}", &digest[..16]),
        digest,
        SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(SubeventContainmentExecution {
        artifact,
        terminal_result,
    })
}

fn census_assignments(
    assignments: &[SubeventContainmentAssignment],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<(u64, u64, u64), AnalysisEngineError> {
    let mut seen = std::collections::BTreeSet::new();
    let mut contained_count = 0_u64;
    let mut escaped_count = 0_u64;
    let mut refused_as_escape_count = 0_u64;
    for assignment in assignments {
        if !seen.insert(assignment.assignment_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        if assignment.available_time().instant() > knowledge_cutoff.instant() {
            continue;
        }
        match refuse_escaped_subevent(assignment.parent(), assignment.child()) {
            Ok(()) => {
                contained_count = increment(contained_count)?;
            }
            Err(SubeventContainmentError::SubeventEscapesParent) => {
                refused_as_escape_count = increment(refused_as_escape_count)?;
                escaped_count = increment(escaped_count)?;
            }
            Err(SubeventContainmentError::InvalidIntervalPayload | _) => {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
        }
    }
    Ok((contained_count, escaped_count, refused_as_escape_count))
}

fn increment(count: u64) -> Result<u64, AnalysisEngineError> {
    count
        .checked_add(1)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::{
        SUBEVENT_CONTAINMENT_ARTIFACT_BYTE_LIMIT, SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION,
        SUBEVENT_CONTAINMENT_INFERENCE_STATUS, SubeventContainmentArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> SubeventContainmentArtifact {
        SubeventContainmentArtifact {
            schema_version: SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            assignment_count: 2,
            contained_count: 1,
            escaped_count: 1,
            refused_as_escape_count: 1,
            inference_status: SUBEVENT_CONTAINMENT_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &SubeventContainmentArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidSubeventContainmentArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            SubeventContainmentArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            SubeventContainmentArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidSubeventContainmentArtifact)
        );
        assert_eq!(
            SubeventContainmentArtifact::from_json(
                &"x".repeat(SUBEVENT_CONTAINMENT_ARTIFACT_BYTE_LIMIT + 1)
            ),
            Err(AnalysisEngineError::LimitExceeded)
        );
    }

    #[test]
    fn artifact_metadata_tampering_fails_closed() {
        let artifact = artifact();
        let invalid_artifacts = [
            {
                let mut value = artifact.clone();
                value.schema_version.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.run_id.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.snapshot_id.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.knowledge_cutoff = "invalid".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.assignment_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.contained_count = 0;
                value.assignment_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.escaped_count = 0;
                value.refused_as_escape_count = 0;
                value.assignment_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_escape_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.inference_status.clear();
                value
            },
        ];
        for invalid in invalid_artifacts {
            assert_invalid(&invalid);
        }
    }
}
