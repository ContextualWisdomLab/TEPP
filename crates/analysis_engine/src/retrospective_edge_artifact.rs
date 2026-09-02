//! Digest-bound retrospective-edge refusals as an analysis-run profile.

use std::collections::BTreeSet;

use retrospective_edge::{
    RetrospectiveEdgeError, RetrospectiveKind, refuse_retrospective_as_transition,
    refuse_retrospective_as_translation,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, format_digest, require_receipt_identity,
    valid_identifier,
};

/// Versioned schema for a completed retrospective-edge artifact.
pub const RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.retrospective_edge.v1";
/// Model contract required by the retrospective-edge execution path.
pub const RETROSPECTIVE_EDGE_MODEL_CONTRACT_VERSION: &str = "retrospective_edge_v1";
/// Analysis-run output profile required for a retrospective-edge artifact.
pub const RETROSPECTIVE_EDGE_OUTPUT_PROFILE: &str = "retrospective_edge_v1";
/// Maximum canonical artifact JSON size.
pub const RETROSPECTIVE_EDGE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const RETROSPECTIVE_EDGE_INFERENCE_STATUS: &str =
    "retrospective_report_is_not_transition_and_not_translation";

/// One cutoff-admitted retrospective-reporting assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetrospectiveEdgeAssignment {
    assignment_id: String,
    kind: RetrospectiveKind,
    available_time: AvailableTime,
}

impl RetrospectiveEdgeAssignment {
    /// Construct a bounded retrospective-edge assignment.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the assignment
    /// identity is empty or oversized.
    pub fn new(
        assignment_id: impl Into<String>,
        kind: RetrospectiveKind,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let assignment_id = assignment_id.into();
        if !valid_identifier(&assignment_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            assignment_id,
            kind,
            available_time,
        })
    }

    /// Return the opaque assignment identity.
    #[must_use]
    pub fn assignment_id(&self) -> &str {
        &self.assignment_id
    }

    /// Return the closed reporting kind.
    #[must_use]
    pub const fn kind(&self) -> RetrospectiveKind {
        self.kind
    }

    /// Return the availability time used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded retrospective-edge census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetrospectiveEdgeArtifact {
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
    /// Retrospective-report assignments admitted at the cutoff.
    pub retrospective_report_count: u64,
    /// Forward-report assignments admitted at the cutoff.
    pub forward_report_count: u64,
    /// Retrospective reports refused as a state transition.
    pub refused_as_transition_count: u64,
    /// Retrospective reports refused as a translation.
    pub refused_as_translation_count: u64,
    /// Forward reports that passed both refusals.
    pub compatible_forward_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl RetrospectiveEdgeArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidRetrospectiveEdgeArtifact`] when
    /// the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > RETROSPECTIVE_EDGE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidRetrospectiveEdgeArtifact)?;
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
        if payload.len() > RETROSPECTIVE_EDGE_ARTIFACT_BYTE_LIMIT {
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
        let kind_sum = self
            .retrospective_report_count
            .checked_add(self.forward_report_count);
        if self.schema_version != RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.assignment_count < 2
            || self.assignment_count > MAX_EVIDENCE_UNITS as u64
            || self.retrospective_report_count == 0
            || self.forward_report_count == 0
            || kind_sum != Some(self.assignment_count)
            || self.refused_as_transition_count != self.retrospective_report_count
            || self.refused_as_translation_count != self.retrospective_report_count
            || self.compatible_forward_count != self.forward_report_count
            || self.inference_status != RETROSPECTIVE_EDGE_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidRetrospectiveEdgeArtifact);
        }
        Ok(())
    }
}

/// One completed retrospective-edge artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct RetrospectiveEdgeExecution {
    /// Digest-bound completed retrospective-edge census.
    pub artifact: RetrospectiveEdgeArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe retrospective-edge refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_retrospective_as_transition`] and
/// [`refuse_retrospective_as_translation`] already on protected main. A later
/// report may point at earlier event time. It never becomes a state transition
/// or a translation. It does not emit `identity_recovery_rate`, a
/// `scientific_acceptance` inspect metric, GPU kernels, MCMC, or topic
/// birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-kind corpus, missing refusal or compatible forward, duplicate
/// assignment identity, oversized corpus, or invalid artifact error.
pub fn execute_retrospective_edge_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    assignments: &[RetrospectiveEdgeAssignment],
    completed_at: impl Into<String>,
) -> Result<RetrospectiveEdgeExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != RETROSPECTIVE_EDGE_MODEL_CONTRACT_VERSION
        || request.output_profile != RETROSPECTIVE_EDGE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if assignments.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let census = census_admitted_assignments(assignments, knowledge_cutoff)?;
    let artifact = RetrospectiveEdgeArtifact {
        schema_version: RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        assignment_count: census.assignment_count,
        retrospective_report_count: census.retrospective_report_count,
        forward_report_count: census.forward_report_count,
        refused_as_transition_count: census.refused_as_transition_count,
        refused_as_translation_count: census.refused_as_translation_count,
        compatible_forward_count: census.compatible_forward_count,
        inference_status: RETROSPECTIVE_EDGE_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "retrospective_edge",
        census.assignment_count,
        4,
        "validated",
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("retrospective_edge_artifact_{}", &digest[..16]),
        digest,
        RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(RetrospectiveEdgeExecution {
        artifact,
        terminal_result,
    })
}

#[allow(clippy::struct_field_names)]
struct RetrospectiveEdgeCensus {
    assignment_count: u64,
    retrospective_report_count: u64,
    forward_report_count: u64,
    refused_as_transition_count: u64,
    refused_as_translation_count: u64,
    compatible_forward_count: u64,
}

fn census_admitted_assignments(
    assignments: &[RetrospectiveEdgeAssignment],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<RetrospectiveEdgeCensus, AnalysisEngineError> {
    let mut seen = BTreeSet::new();
    let mut retrospective_report_count = 0_u64;
    let mut forward_report_count = 0_u64;
    let mut refused_as_transition_count = 0_u64;
    let mut refused_as_translation_count = 0_u64;
    let mut compatible_forward_count = 0_u64;
    for assignment in assignments {
        if assignment.available_time().instant() > knowledge_cutoff.instant() {
            continue;
        }
        if !seen.insert(assignment.assignment_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        classify_assignment(
            assignment.kind(),
            &mut retrospective_report_count,
            &mut forward_report_count,
            &mut refused_as_transition_count,
            &mut refused_as_translation_count,
            &mut compatible_forward_count,
        )?;
    }

    let assignment_count = retrospective_report_count
        .checked_add(forward_report_count)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if assignment_count < 2
        || retrospective_report_count == 0
        || forward_report_count == 0
        || refused_as_transition_count != retrospective_report_count
        || refused_as_translation_count != retrospective_report_count
        || compatible_forward_count != forward_report_count
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    Ok(RetrospectiveEdgeCensus {
        assignment_count,
        retrospective_report_count,
        forward_report_count,
        refused_as_transition_count,
        refused_as_translation_count,
        compatible_forward_count,
    })
}

fn classify_assignment(
    kind: RetrospectiveKind,
    retrospective_report_count: &mut u64,
    forward_report_count: &mut u64,
    refused_as_transition_count: &mut u64,
    refused_as_translation_count: &mut u64,
    compatible_forward_count: &mut u64,
) -> Result<(), AnalysisEngineError> {
    match (
        refuse_retrospective_as_transition(kind),
        refuse_retrospective_as_translation(kind),
    ) {
        (
            Err(RetrospectiveEdgeError::RetrospectiveIsNotTransition),
            Err(RetrospectiveEdgeError::RetrospectiveIsNotTranslation),
        ) => {
            if kind != RetrospectiveKind::RetrospectiveReport {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
            *refused_as_transition_count = increment(*refused_as_transition_count)?;
            *refused_as_translation_count = increment(*refused_as_translation_count)?;
            *retrospective_report_count = increment(*retrospective_report_count)?;
        }
        (Ok(()), Ok(())) => {
            if kind != RetrospectiveKind::ForwardReport {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
            *compatible_forward_count = increment(*compatible_forward_count)?;
            *forward_report_count = increment(*forward_report_count)?;
        }
        _ => return Err(AnalysisEngineError::InvalidEvidence),
    }
    Ok(())
}

fn increment(count: u64) -> Result<u64, AnalysisEngineError> {
    count
        .checked_add(1)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::{
        RETROSPECTIVE_EDGE_ARTIFACT_BYTE_LIMIT, RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION,
        RETROSPECTIVE_EDGE_INFERENCE_STATUS, RetrospectiveEdgeArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> RetrospectiveEdgeArtifact {
        RetrospectiveEdgeArtifact {
            schema_version: RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            assignment_count: 3,
            retrospective_report_count: 2,
            forward_report_count: 1,
            refused_as_transition_count: 2,
            refused_as_translation_count: 2,
            compatible_forward_count: 1,
            inference_status: RETROSPECTIVE_EDGE_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &RetrospectiveEdgeArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidRetrospectiveEdgeArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            RetrospectiveEdgeArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            RetrospectiveEdgeArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidRetrospectiveEdgeArtifact)
        );
        assert_eq!(
            RetrospectiveEdgeArtifact::from_json(
                &"x".repeat(RETROSPECTIVE_EDGE_ARTIFACT_BYTE_LIMIT + 1)
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
                value.retrospective_report_count = 0;
                value.assignment_count = 1;
                value.refused_as_transition_count = 0;
                value.refused_as_translation_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.forward_report_count = 0;
                value.assignment_count = 2;
                value.compatible_forward_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_transition_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_translation_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.compatible_forward_count = 0;
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
