//! Digest-bound support-edge refusals as an analysis-run profile.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use support_edge::{refuse_evidence_as_transition, EvidenceKind, SupportEdgeError};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    format_digest, require_receipt_identity, valid_identifier, AnalysisEngineError,
    MAX_EVIDENCE_UNITS,
};

/// Versioned schema for a completed support-edge artifact.
pub const SUPPORT_EDGE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.support_edge.v1";
/// Model contract required by the support-edge execution path.
pub const SUPPORT_EDGE_MODEL_CONTRACT_VERSION: &str = "support_edge_v1";
/// Analysis-run output profile required for a support-edge artifact.
pub const SUPPORT_EDGE_OUTPUT_PROFILE: &str = "support_edge_v1";
/// Maximum canonical artifact JSON size.
pub const SUPPORT_EDGE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const SUPPORT_EDGE_INFERENCE_STATUS: &str = "evidence_is_not_transition";

/// One cutoff-admitted evidential-kind assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupportEdgeAssignment {
    assignment_id: String,
    kind: EvidenceKind,
    available_time: AvailableTime,
}

impl SupportEdgeAssignment {
    /// Construct a bounded support-edge assignment.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the assignment
    /// identity is empty or oversized.
    pub fn new(
        assignment_id: impl Into<String>,
        kind: EvidenceKind,
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

    /// Return the closed evidential kind.
    #[must_use]
    pub const fn kind(&self) -> EvidenceKind {
        self.kind
    }

    /// Return the availability time used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded support-edge census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SupportEdgeArtifact {
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
    /// Support assignments admitted at the cutoff.
    pub support_count: u64,
    /// Contradiction assignments admitted at the cutoff.
    pub contradiction_count: u64,
    /// Summary assignments admitted at the cutoff.
    pub summarizes_count: u64,
    /// Inverse-production assignments admitted at the cutoff.
    pub outcome_of_count: u64,
    /// Evidential kinds refused as a state transition.
    pub refused_as_transition_count: u64,
    /// Fixed claim boundary for operator copy.
    pub inference_status: String,
}

impl SupportEdgeArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidSupportEdgeArtifact`] when
    /// the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > SUPPORT_EDGE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidSupportEdgeArtifact)?;
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
        if payload.len() > SUPPORT_EDGE_ARTIFACT_BYTE_LIMIT {
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
            .support_count
            .checked_add(self.contradiction_count)
            .and_then(|sum| sum.checked_add(self.summarizes_count))
            .and_then(|sum| sum.checked_add(self.outcome_of_count));
        if self.schema_version != SUPPORT_EDGE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.assignment_count < 4
            || self.assignment_count > MAX_EVIDENCE_UNITS as u64
            || self.support_count == 0
            || self.contradiction_count == 0
            || self.summarizes_count == 0
            || self.outcome_of_count == 0
            || kind_sum != Some(self.assignment_count)
            || self.refused_as_transition_count != self.assignment_count
            || self.inference_status != SUPPORT_EDGE_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidSupportEdgeArtifact);
        }
        Ok(())
    }
}

/// One completed support-edge artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct SupportEdgeExecution {
    /// Digest-bound completed support-edge census.
    pub artifact: SupportEdgeArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe support-edge refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_evidence_as_transition`] already on
/// protected main. Support, contradiction, summary, and `outcome_of` may
/// point at earlier event time. They never become state transitions.
/// `edge_kind_recovery_rate` stays library-side. It does not emit a
/// `scientific_acceptance` inspect metric, GPU kernels, MCMC, or topic
/// birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// incomplete mixed-kind corpus, missing refusal, duplicate assignment
/// identity, oversized corpus, or invalid artifact error.
pub fn execute_support_edge_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    assignments: &[SupportEdgeAssignment],
    completed_at: impl Into<String>,
) -> Result<SupportEdgeExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != SUPPORT_EDGE_MODEL_CONTRACT_VERSION
        || request.output_profile != SUPPORT_EDGE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if assignments.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let census = census_admitted_assignments(assignments, knowledge_cutoff)?;
    let artifact = SupportEdgeArtifact {
        schema_version: SUPPORT_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        assignment_count: census.assignment_count,
        support_count: census.support_count,
        contradiction_count: census.contradiction_count,
        summarizes_count: census.summarizes_count,
        outcome_of_count: census.outcome_of_count,
        refused_as_transition_count: census.refused_as_transition_count,
        inference_status: SUPPORT_EDGE_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary =
        AnalysisResultSummary::new("support_edge", census.assignment_count, 4, "validated")?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("support_edge_artifact_{}", &digest[..16]),
        digest,
        SUPPORT_EDGE_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(SupportEdgeExecution {
        artifact,
        terminal_result,
    })
}

#[allow(clippy::struct_field_names)]
struct SupportEdgeCensus {
    assignment_count: u64,
    support_count: u64,
    contradiction_count: u64,
    summarizes_count: u64,
    outcome_of_count: u64,
    refused_as_transition_count: u64,
}

fn census_admitted_assignments(
    assignments: &[SupportEdgeAssignment],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<SupportEdgeCensus, AnalysisEngineError> {
    let mut seen = BTreeSet::new();
    let mut support_count = 0_u64;
    let mut contradiction_count = 0_u64;
    let mut summarizes_count = 0_u64;
    let mut outcome_of_count = 0_u64;
    let mut refused_as_transition_count = 0_u64;
    for assignment in assignments {
        if assignment.available_time().instant() > knowledge_cutoff.instant() {
            continue;
        }
        if !seen.insert(assignment.assignment_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        classify_assignment(
            assignment.kind(),
            &mut support_count,
            &mut contradiction_count,
            &mut summarizes_count,
            &mut outcome_of_count,
            &mut refused_as_transition_count,
        )?;
    }

    let assignment_count = support_count
        .checked_add(contradiction_count)
        .and_then(|sum| sum.checked_add(summarizes_count))
        .and_then(|sum| sum.checked_add(outcome_of_count))
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if assignment_count < 4
        || support_count == 0
        || contradiction_count == 0
        || summarizes_count == 0
        || outcome_of_count == 0
        || refused_as_transition_count != assignment_count
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    Ok(SupportEdgeCensus {
        assignment_count,
        support_count,
        contradiction_count,
        summarizes_count,
        outcome_of_count,
        refused_as_transition_count,
    })
}

fn classify_assignment(
    kind: EvidenceKind,
    support_count: &mut u64,
    contradiction_count: &mut u64,
    summarizes_count: &mut u64,
    outcome_of_count: &mut u64,
    refused_as_transition_count: &mut u64,
) -> Result<(), AnalysisEngineError> {
    match refuse_evidence_as_transition(kind) {
        Err(SupportEdgeError::EvidenceIsNotTransition) => {
            *refused_as_transition_count = increment(*refused_as_transition_count)?;
            match kind {
                EvidenceKind::Support => *support_count = increment(*support_count)?,
                EvidenceKind::Contradiction => {
                    *contradiction_count = increment(*contradiction_count)?;
                }
                EvidenceKind::Summarizes => *summarizes_count = increment(*summarizes_count)?,
                EvidenceKind::OutcomeOf => *outcome_of_count = increment(*outcome_of_count)?,
            }
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
        SupportEdgeArtifact, SUPPORT_EDGE_ARTIFACT_BYTE_LIMIT,
        SUPPORT_EDGE_ARTIFACT_SCHEMA_VERSION, SUPPORT_EDGE_INFERENCE_STATUS,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> SupportEdgeArtifact {
        SupportEdgeArtifact {
            schema_version: SUPPORT_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            assignment_count: 4,
            support_count: 1,
            contradiction_count: 1,
            summarizes_count: 1,
            outcome_of_count: 1,
            refused_as_transition_count: 4,
            inference_status: SUPPORT_EDGE_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &SupportEdgeArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidSupportEdgeArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            SupportEdgeArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            SupportEdgeArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidSupportEdgeArtifact)
        );
        assert_eq!(
            SupportEdgeArtifact::from_json(&"x".repeat(SUPPORT_EDGE_ARTIFACT_BYTE_LIMIT + 1)),
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
                value.assignment_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.support_count = 0;
                value.assignment_count = 3;
                value.refused_as_transition_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.contradiction_count = 0;
                value.assignment_count = 3;
                value.refused_as_transition_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.summarizes_count = 0;
                value.assignment_count = 3;
                value.refused_as_transition_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.outcome_of_count = 0;
                value.assignment_count = 3;
                value.refused_as_transition_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_transition_count = 0;
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
