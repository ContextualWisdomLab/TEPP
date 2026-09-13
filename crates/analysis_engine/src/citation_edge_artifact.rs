//! Digest-bound provenance-is-not-transition refusals as an analysis-run profile.

use citation_edge::{CitationEdgeError, ProvenanceKind, refuse_provenance_as_transition};
use corpus_split::cutoff_eligible;
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

/// Versioned schema for a completed citation-edge artifact.
pub const CITATION_EDGE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.citation_edge.v1";
/// Model contract required by the citation-edge execution path.
pub const CITATION_EDGE_MODEL_CONTRACT_VERSION: &str = "citation_edge_v1";
/// Analysis-run output profile required for a citation-edge artifact.
pub const CITATION_EDGE_OUTPUT_PROFILE: &str = "citation_edge_v1";
/// Maximum canonical artifact JSON size.
pub const CITATION_EDGE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const CITATION_EDGE_INFERENCE_STATUS: &str = "provenance_is_not_a_state_transition";

/// One provenance edge with immutable snapshot and availability provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CitationEdgeDocument {
    document_id: String,
    kind: ProvenanceKind,
    snapshot_id: String,
    available_time: AvailableTime,
}

impl CitationEdgeDocument {
    /// Construct a bounded citation-edge document with explicit provenance.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the document or
    /// snapshot identity is empty or oversized.
    pub fn new(
        document_id: impl Into<String>,
        kind: ProvenanceKind,
        snapshot_id: impl Into<String>,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let document_id = document_id.into();
        let snapshot_id = snapshot_id.into();
        if !valid_identifier(&document_id) || !valid_identifier(&snapshot_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            document_id,
            kind,
            snapshot_id,
            available_time,
        })
    }

    /// Return the opaque document identity.
    #[must_use]
    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    /// Return the closed provenance kind.
    #[must_use]
    pub const fn kind(&self) -> ProvenanceKind {
        self.kind
    }

    /// Return the immutable source snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return when the edge became available for historical analysis.
    #[must_use]
    pub const fn available_time(&self) -> &AvailableTime {
        &self.available_time
    }
}

/// Completed, bounded citation-edge census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CitationEdgeArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used to admit documents.
    pub knowledge_cutoff: String,
    /// Number of provenance edges admitted at the cutoff.
    pub document_count: u64,
    /// Citation edges admitted at the cutoff.
    pub citation_count: u64,
    /// Translation edges admitted at the cutoff.
    pub translation_count: u64,
    /// Revision edges admitted at the cutoff.
    pub revision_count: u64,
    /// Retrospective-report edges admitted at the cutoff.
    pub retrospective_report_count: u64,
    /// Provenance edges refused as forward state transitions.
    pub refused_as_transition_count: u64,
    /// Number of distinct provenance kinds in the census.
    pub distinct_kind_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl CitationEdgeArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidCitationEdgeArtifact`] when the
    /// schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > CITATION_EDGE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidCitationEdgeArtifact)?;
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
        if payload.len() > CITATION_EDGE_ARTIFACT_BYTE_LIMIT {
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
            .citation_count
            .checked_add(self.translation_count)
            .and_then(|value| value.checked_add(self.revision_count))
            .and_then(|value| value.checked_add(self.retrospective_report_count));
        let populated = u64::from(self.citation_count > 0)
            + u64::from(self.translation_count > 0)
            + u64::from(self.revision_count > 0)
            + u64::from(self.retrospective_report_count > 0);
        if self.schema_version != CITATION_EDGE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.document_count < 2
            || self.document_count > MAX_EVIDENCE_UNITS as u64
            || self.distinct_kind_count < 2
            || populated != self.distinct_kind_count
            || kind_sum != Some(self.document_count)
            || self.refused_as_transition_count != self.document_count
            || self.inference_status != CITATION_EDGE_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidCitationEdgeArtifact);
        }
        Ok(())
    }
}

/// One completed citation-edge artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct CitationEdgeExecution {
    /// Digest-bound completed citation-edge census.
    pub artifact: CitationEdgeArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe provenance-is-not-transition refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_provenance_as_transition`] already on
/// protected main. It does not emit `edge_kind_recovery_rate`, a
/// `scientific_acceptance` inspect metric, GPU kernels, MCMC, or topic
/// birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-kind admitted corpus, duplicate admitted document identity,
/// oversized raw corpus, or invalid artifact error.
pub fn execute_citation_edge_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    documents: &[CitationEdgeDocument],
    completed_at: impl Into<String>,
) -> Result<CitationEdgeExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != CITATION_EDGE_MODEL_CONTRACT_VERSION
        || request.output_profile != CITATION_EDGE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if documents.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut citation_count = 0_u64;
    let mut translation_count = 0_u64;
    let mut revision_count = 0_u64;
    let mut retrospective_report_count = 0_u64;
    let mut refused_as_transition_count = 0_u64;
    for document in documents {
        if document.snapshot_id() != snapshot_id {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        if !cutoff_eligible(document.available_time(), &knowledge_cutoff) {
            continue;
        }
        if !seen.insert(document.document_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        match refuse_provenance_as_transition(document.kind()) {
            Err(CitationEdgeError::ProvenanceIsNotTransition) => {
                refused_as_transition_count = refused_as_transition_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
        }
        match document.kind() {
            ProvenanceKind::Citation => {
                citation_count = citation_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            ProvenanceKind::Translation => {
                translation_count = translation_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            ProvenanceKind::Revision => {
                revision_count = revision_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            ProvenanceKind::RetrospectiveReport => {
                retrospective_report_count = retrospective_report_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
        }
    }
    let document_count =
        u64::try_from(seen.len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    let distinct_kind_count = u64::from(citation_count > 0)
        + u64::from(translation_count > 0)
        + u64::from(revision_count > 0)
        + u64::from(retrospective_report_count > 0);
    if document_count < 2 || distinct_kind_count < 2 {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = CitationEdgeArtifact {
        schema_version: CITATION_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        document_count,
        citation_count,
        translation_count,
        revision_count,
        retrospective_report_count,
        refused_as_transition_count,
        distinct_kind_count,
        inference_status: CITATION_EDGE_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new("citation_edge", document_count, 4, "validated")?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("citation_edge_artifact_{}", &digest[..16]),
        digest,
        CITATION_EDGE_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(CitationEdgeExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CITATION_EDGE_ARTIFACT_BYTE_LIMIT, CITATION_EDGE_ARTIFACT_SCHEMA_VERSION,
        CITATION_EDGE_INFERENCE_STATUS, CitationEdgeArtifact,
    };
    use crate::{AnalysisEngineError, MAX_EVIDENCE_UNITS};

    fn artifact() -> CitationEdgeArtifact {
        CitationEdgeArtifact {
            schema_version: CITATION_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            document_count: 3,
            citation_count: 2,
            translation_count: 0,
            revision_count: 1,
            retrospective_report_count: 0,
            refused_as_transition_count: 3,
            distinct_kind_count: 2,
            inference_status: CITATION_EDGE_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &CitationEdgeArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidCitationEdgeArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            CitationEdgeArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            CitationEdgeArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidCitationEdgeArtifact)
        );
        assert_eq!(
            CitationEdgeArtifact::from_json(&"x".repeat(CITATION_EDGE_ARTIFACT_BYTE_LIMIT + 1)),
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
                value.document_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.document_count = u64::try_from(MAX_EVIDENCE_UNITS).expect("bound") + 1;
                value.citation_count = value.document_count - 1;
                value.translation_count = 0;
                value.revision_count = 1;
                value.retrospective_report_count = 0;
                value.refused_as_transition_count = value.document_count;
                value.distinct_kind_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.distinct_kind_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_transition_count = 1;
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
