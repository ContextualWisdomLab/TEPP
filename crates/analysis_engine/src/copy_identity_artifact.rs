//! Digest-bound template-copy identity refusals as an analysis-run profile.

use copy_identity::{
    CopyIdentityError, CopyKind, refuse_copy_as_source_identity, refuse_copy_as_transition,
};
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

/// Versioned schema for a completed copy-identity artifact.
pub const COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION: &str = "tepp.copy_identity.v1";
/// Model contract required by the copy-identity execution path.
pub const COPY_IDENTITY_MODEL_CONTRACT_VERSION: &str = "copy_identity_v1";
/// Analysis-run output profile required for a copy-identity artifact.
pub const COPY_IDENTITY_OUTPUT_PROFILE: &str = "copy_identity_v1";
/// Maximum canonical artifact JSON size.
pub const COPY_IDENTITY_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const COPY_IDENTITY_INFERENCE_STATUS: &str = "template_copy_is_not_source_identity_not_transition";

/// One cutoff-admitted document with a closed copy-identity kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyIdentityDocument {
    document_id: String,
    kind: CopyKind,
    available_time: AvailableTime,
}

impl CopyIdentityDocument {
    /// Construct a bounded copy-identity document.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the document
    /// identity is empty or oversized.
    pub fn new(
        document_id: impl Into<String>,
        kind: CopyKind,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let document_id = document_id.into();
        if !valid_identifier(&document_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            document_id,
            kind,
            available_time,
        })
    }

    /// Return the opaque document identity.
    #[must_use]
    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    /// Return the closed copy-identity kind.
    #[must_use]
    pub const fn kind(&self) -> CopyKind {
        self.kind
    }

    /// Return when the document became available for historical analysis.
    #[must_use]
    pub const fn available_time(&self) -> &AvailableTime {
        &self.available_time
    }
}

/// Completed, bounded copy-identity census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CopyIdentityArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used to admit documents.
    pub knowledge_cutoff: String,
    /// Number of documents admitted at the cutoff.
    pub document_count: u64,
    /// Source documents admitted at the cutoff.
    pub source_document_count: u64,
    /// Template copies admitted at the cutoff.
    pub template_copy_count: u64,
    /// Template copies refused as the source identity.
    pub refused_as_source_count: u64,
    /// Template copies refused as a state transition.
    pub refused_as_transition_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl CopyIdentityArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidCopyIdentityArtifact`] when the
    /// schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > COPY_IDENTITY_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidCopyIdentityArtifact)?;
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
        if payload.len() > COPY_IDENTITY_ARTIFACT_BYTE_LIMIT {
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
            .source_document_count
            .checked_add(self.template_copy_count);
        if self.schema_version != COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.document_count < 2
            || self.document_count > MAX_EVIDENCE_UNITS as u64
            || self.source_document_count == 0
            || self.template_copy_count == 0
            || kind_sum != Some(self.document_count)
            || self.refused_as_source_count != self.template_copy_count
            || self.refused_as_transition_count != self.template_copy_count
            || self.inference_status != COPY_IDENTITY_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidCopyIdentityArtifact);
        }
        Ok(())
    }
}

/// One completed copy-identity artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct CopyIdentityExecution {
    /// Digest-bound completed copy-identity census.
    pub artifact: CopyIdentityArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe template-copy identity refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_copy_as_source_identity`] and
/// [`refuse_copy_as_transition`] already on protected main. It does not emit
/// `identity_recovery_rate`, a `scientific_acceptance` inspect metric, GPU
/// kernels, MCMC, or topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-kind corpus, duplicate document identity, oversized corpus, or
/// invalid artifact error.
pub fn execute_copy_identity_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    documents: &[CopyIdentityDocument],
    completed_at: impl Into<String>,
) -> Result<CopyIdentityExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != COPY_IDENTITY_MODEL_CONTRACT_VERSION
        || request.output_profile != COPY_IDENTITY_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if documents.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut source_document_count = 0_u64;
    let mut template_copy_count = 0_u64;
    let mut refused_as_source_count = 0_u64;
    let mut refused_as_transition_count = 0_u64;
    for document in documents {
        if !cutoff_eligible(document.available_time(), &knowledge_cutoff) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        if !seen.insert(document.document_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        match document.kind() {
            CopyKind::SourceDocument => {
                require_copy_result(refuse_copy_as_source_identity(document.kind()), Ok(()))?;
                require_copy_result(refuse_copy_as_transition(document.kind()), Ok(()))?;
                source_document_count = source_document_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            CopyKind::TemplateCopy => {
                #[rustfmt::skip]
                require_copy_result(refuse_copy_as_source_identity(document.kind()), Err(CopyIdentityError::CopyIsNotSourceIdentity))?;
                refused_as_source_count = refused_as_source_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                #[rustfmt::skip]
                require_copy_result(refuse_copy_as_transition(document.kind()), Err(CopyIdentityError::CopyIsNotTransition))?;
                refused_as_transition_count = refused_as_transition_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                template_copy_count = template_copy_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
        }
    }
    let document_count =
        u64::try_from(documents.len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    if document_count < 2 || source_document_count == 0 || template_copy_count == 0 {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = CopyIdentityArtifact {
        schema_version: COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        document_count,
        source_document_count,
        template_copy_count,
        refused_as_source_count,
        refused_as_transition_count,
        inference_status: COPY_IDENTITY_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    #[rustfmt::skip]
    let summary = AnalysisResultSummary::new("copy_identity", document_count, 4, COPY_IDENTITY_INFERENCE_STATUS)?;
    #[rustfmt::skip]
    let terminal_result = AnalysisRunTerminalResult::succeeded(request, accepted, format!("copy_identity_artifact_{}", &digest[..16]), digest, COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION, completed_at, summary)?;
    Ok(CopyIdentityExecution {
        artifact,
        terminal_result,
    })
}

fn require_copy_result(
    actual: Result<(), CopyIdentityError>,
    expected: Result<(), CopyIdentityError>,
) -> Result<(), AnalysisEngineError> {
    if actual != expected {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        COPY_IDENTITY_ARTIFACT_BYTE_LIMIT, COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION,
        COPY_IDENTITY_INFERENCE_STATUS, CopyIdentityArtifact, require_copy_result,
    };
    use crate::AnalysisEngineError;
    use copy_identity::CopyIdentityError;

    fn artifact() -> CopyIdentityArtifact {
        CopyIdentityArtifact {
            schema_version: COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            document_count: 3,
            source_document_count: 1,
            template_copy_count: 2,
            refused_as_source_count: 2,
            refused_as_transition_count: 2,
            inference_status: COPY_IDENTITY_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &CopyIdentityArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidCopyIdentityArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            CopyIdentityArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            CopyIdentityArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidCopyIdentityArtifact)
        );
        assert_eq!(
            CopyIdentityArtifact::from_json(&"x".repeat(COPY_IDENTITY_ARTIFACT_BYTE_LIMIT + 1)),
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
                value.source_document_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.template_copy_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.document_count = 4;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_source_count = 1;
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

    #[test]
    fn copy_result_contract_rejects_mismatched_library_outcomes() {
        assert_eq!(require_copy_result(Ok(()), Ok(())), Ok(()));
        assert_eq!(
            require_copy_result(Ok(()), Err(CopyIdentityError::CopyIsNotTransition)),
            Err(AnalysisEngineError::InvalidEvidence)
        );
        assert_eq!(
            require_copy_result(
                Err(CopyIdentityError::InvalidCopyPayload),
                Err(CopyIdentityError::CopyIsNotSourceIdentity),
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
