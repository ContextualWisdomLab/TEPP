//! Digest-bound summarizes-edge refusals as an analysis-run profile.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use summarizes_edge::{
    SummarizesEdgeError, SummarizesKind, refuse_summary_as_source_identity,
    refuse_summary_as_transition,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, format_digest, require_receipt_identity,
    valid_identifier,
};

/// Versioned schema for a completed summarizes-edge artifact.
pub const SUMMARIZES_EDGE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.summarizes_edge.v1";
/// Model contract required by the summarizes-edge execution path.
pub const SUMMARIZES_EDGE_MODEL_CONTRACT_VERSION: &str = "summarizes_edge_v1";
/// Analysis-run output profile required for a summarizes-edge artifact.
pub const SUMMARIZES_EDGE_OUTPUT_PROFILE: &str = "summarizes_edge_v1";
/// Maximum canonical artifact JSON size.
pub const SUMMARIZES_EDGE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const SUMMARIZES_EDGE_INFERENCE_STATUS: &str = "summary_is_not_transition_and_not_source_identity";

/// One cutoff-admitted summary-versus-source assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SummarizesEdgeAssignment {
    assignment_id: String,
    kind: SummarizesKind,
    available_time: AvailableTime,
}

impl SummarizesEdgeAssignment {
    /// Construct a bounded summarizes-edge assignment.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the assignment
    /// identity is empty or oversized.
    pub fn new(
        assignment_id: impl Into<String>,
        kind: SummarizesKind,
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

    /// Return the closed summary-related kind.
    #[must_use]
    pub const fn kind(&self) -> SummarizesKind {
        self.kind
    }

    /// Return the availability time used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded summarizes-edge census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SummarizesEdgeArtifact {
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
    /// Summary assignments admitted at the cutoff.
    pub summary_count: u64,
    /// Source-document assignments admitted at the cutoff.
    pub source_document_count: u64,
    /// Summaries refused as a state transition.
    pub refused_as_transition_count: u64,
    /// Summaries refused as the source document identity.
    pub refused_as_source_identity_count: u64,
    /// Source documents that passed both refusals.
    pub compatible_source_count: u64,
    /// Fixed claim boundary for operator copy.
    pub inference_status: String,
}

impl SummarizesEdgeArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidSummarizesEdgeArtifact`] when
    /// the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > SUMMARIZES_EDGE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidSummarizesEdgeArtifact)?;
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
        if payload.len() > SUMMARIZES_EDGE_ARTIFACT_BYTE_LIMIT {
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
        let kind_sum = self.summary_count.checked_add(self.source_document_count);
        if self.schema_version != SUMMARIZES_EDGE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.assignment_count < 2
            || self.assignment_count > MAX_EVIDENCE_UNITS as u64
            || self.summary_count == 0
            || self.source_document_count == 0
            || kind_sum != Some(self.assignment_count)
            || self.refused_as_transition_count != self.summary_count
            || self.refused_as_source_identity_count != self.summary_count
            || self.compatible_source_count != self.source_document_count
            || self.inference_status != SUMMARIZES_EDGE_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidSummarizesEdgeArtifact);
        }
        Ok(())
    }
}

/// One completed summarizes-edge artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct SummarizesEdgeExecution {
    /// Digest-bound completed summarizes-edge census.
    pub artifact: SummarizesEdgeArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe summarizes-edge refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_summary_as_transition`] and
/// [`refuse_summary_as_source_identity`] already on protected main. A summary
/// may point at earlier event time. It never becomes a state transition or
/// the source document identity. It does not emit `identity_recovery_rate`, a
/// `scientific_acceptance` inspect metric, GPU kernels, MCMC, or topic
/// birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-kind corpus, missing refusal or compatible source, duplicate
/// assignment identity, oversized corpus, or invalid artifact error.
pub fn execute_summarizes_edge_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    assignments: &[SummarizesEdgeAssignment],
    completed_at: impl Into<String>,
) -> Result<SummarizesEdgeExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != SUMMARIZES_EDGE_MODEL_CONTRACT_VERSION
        || request.output_profile != SUMMARIZES_EDGE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if assignments.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let census = census_admitted_assignments(assignments, knowledge_cutoff)?;
    let artifact = SummarizesEdgeArtifact {
        schema_version: SUMMARIZES_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        assignment_count: census.assignment_count,
        summary_count: census.summary_count,
        source_document_count: census.source_document_count,
        refused_as_transition_count: census.refused_as_transition_count,
        refused_as_source_identity_count: census.refused_as_source_identity_count,
        compatible_source_count: census.compatible_source_count,
        inference_status: SUMMARIZES_EDGE_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary =
        AnalysisResultSummary::new("summarizes_edge", census.assignment_count, 4, "validated")?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("summarizes_edge_artifact_{}", &digest[..16]),
        digest,
        SUMMARIZES_EDGE_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(SummarizesEdgeExecution {
        artifact,
        terminal_result,
    })
}

#[allow(clippy::struct_field_names)]
struct SummarizesEdgeCensus {
    assignment_count: u64,
    summary_count: u64,
    source_document_count: u64,
    refused_as_transition_count: u64,
    refused_as_source_identity_count: u64,
    compatible_source_count: u64,
}

fn census_admitted_assignments(
    assignments: &[SummarizesEdgeAssignment],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<SummarizesEdgeCensus, AnalysisEngineError> {
    let mut seen = BTreeSet::new();
    let mut summary_count = 0_u64;
    let mut source_document_count = 0_u64;
    let mut refused_as_transition_count = 0_u64;
    let mut refused_as_source_identity_count = 0_u64;
    let mut compatible_source_count = 0_u64;
    for assignment in assignments {
        if assignment.available_time().instant() > knowledge_cutoff.instant() {
            continue;
        }
        if !seen.insert(assignment.assignment_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        classify_assignment(
            assignment.kind(),
            &mut summary_count,
            &mut source_document_count,
            &mut refused_as_transition_count,
            &mut refused_as_source_identity_count,
            &mut compatible_source_count,
        )?;
    }

    let assignment_count = summary_count
        .checked_add(source_document_count)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if assignment_count < 2
        || summary_count == 0
        || source_document_count == 0
        || refused_as_transition_count != summary_count
        || refused_as_source_identity_count != summary_count
        || compatible_source_count != source_document_count
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    Ok(SummarizesEdgeCensus {
        assignment_count,
        summary_count,
        source_document_count,
        refused_as_transition_count,
        refused_as_source_identity_count,
        compatible_source_count,
    })
}

fn classify_assignment(
    kind: SummarizesKind,
    summary_count: &mut u64,
    source_document_count: &mut u64,
    refused_as_transition_count: &mut u64,
    refused_as_source_identity_count: &mut u64,
    compatible_source_count: &mut u64,
) -> Result<(), AnalysisEngineError> {
    match (
        refuse_summary_as_transition(kind),
        refuse_summary_as_source_identity(kind),
    ) {
        (
            Err(SummarizesEdgeError::SummaryIsNotTransition),
            Err(SummarizesEdgeError::SummaryIsNotSourceIdentity),
        ) => {
            if kind != SummarizesKind::Summary {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
            *refused_as_transition_count = increment(*refused_as_transition_count)?;
            *refused_as_source_identity_count = increment(*refused_as_source_identity_count)?;
            *summary_count = increment(*summary_count)?;
        }
        (Ok(()), Ok(())) => {
            if kind != SummarizesKind::SourceDocument {
                return Err(AnalysisEngineError::InvalidEvidence);
            }
            *compatible_source_count = increment(*compatible_source_count)?;
            *source_document_count = increment(*source_document_count)?;
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
        SUMMARIZES_EDGE_ARTIFACT_BYTE_LIMIT, SUMMARIZES_EDGE_ARTIFACT_SCHEMA_VERSION,
        SUMMARIZES_EDGE_INFERENCE_STATUS, SummarizesEdgeArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> SummarizesEdgeArtifact {
        SummarizesEdgeArtifact {
            schema_version: SUMMARIZES_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            assignment_count: 3,
            summary_count: 2,
            source_document_count: 1,
            refused_as_transition_count: 2,
            refused_as_source_identity_count: 2,
            compatible_source_count: 1,
            inference_status: SUMMARIZES_EDGE_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &SummarizesEdgeArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidSummarizesEdgeArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            SummarizesEdgeArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            SummarizesEdgeArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidSummarizesEdgeArtifact)
        );
        assert_eq!(
            SummarizesEdgeArtifact::from_json(&"x".repeat(SUMMARIZES_EDGE_ARTIFACT_BYTE_LIMIT + 1)),
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
                value.summary_count = 0;
                value.assignment_count = 1;
                value.refused_as_transition_count = 0;
                value.refused_as_source_identity_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.source_document_count = 0;
                value.assignment_count = 2;
                value.compatible_source_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_transition_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_source_identity_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.compatible_source_count = 0;
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
