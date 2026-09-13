//! Digest-bound prompt-boilerplate refusals as an analysis-run profile.

use corpus_split::cutoff_eligible;
use prompt_source::{
    PromptKind, PromptSourceError, refuse_prompt_as_stopword_deletion,
    refuse_prompt_as_unique_content,
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

/// Versioned schema for a completed prompt-source artifact.
pub const PROMPT_SOURCE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.prompt_source.v1";
/// Model contract required by the prompt-source execution path.
pub const PROMPT_SOURCE_MODEL_CONTRACT_VERSION: &str = "prompt_source_v1";
/// Analysis-run output profile required for a prompt-source artifact.
pub const PROMPT_SOURCE_OUTPUT_PROFILE: &str = "prompt_source_v1";
/// Maximum canonical artifact JSON size.
pub const PROMPT_SOURCE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const PROMPT_SOURCE_INFERENCE_STATUS: &str =
    "prompt_boilerplate_is_not_unique_content_not_stopword_deletion";

/// One token treatment with immutable snapshot and availability provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptSourceDocument {
    document_id: String,
    kind: PromptKind,
    snapshot_id: String,
    available_time: AvailableTime,
}

impl PromptSourceDocument {
    /// Construct a bounded prompt-source document with explicit provenance.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the document or
    /// snapshot identity is empty or oversized.
    pub fn new(
        document_id: impl Into<String>,
        kind: PromptKind,
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

    /// Return the closed prompt-source kind.
    #[must_use]
    pub const fn kind(&self) -> PromptKind {
        self.kind
    }

    /// Return the immutable source snapshot identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Return when the document became available for historical analysis.
    #[must_use]
    pub const fn available_time(&self) -> &AvailableTime {
        &self.available_time
    }
}

/// Completed, bounded prompt-source census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PromptSourceArtifact {
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
    /// Unique-content treatments admitted at the cutoff.
    pub unique_content_count: u64,
    /// Prompt-boilerplate treatments admitted at the cutoff.
    pub prompt_boilerplate_count: u64,
    /// Prompt boilerplate refused as unique latent content.
    pub refused_as_unique_content_count: u64,
    /// Prompt boilerplate refused as stopword deletion.
    pub refused_as_stopword_deletion_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl PromptSourceArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidPromptSourceArtifact`] when the
    /// schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > PROMPT_SOURCE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidPromptSourceArtifact)?;
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
        if payload.len() > PROMPT_SOURCE_ARTIFACT_BYTE_LIMIT {
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
            .unique_content_count
            .checked_add(self.prompt_boilerplate_count);
        if self.schema_version != PROMPT_SOURCE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.document_count < 2
            || self.document_count > MAX_EVIDENCE_UNITS as u64
            || self.unique_content_count == 0
            || self.prompt_boilerplate_count == 0
            || kind_sum != Some(self.document_count)
            || self.refused_as_unique_content_count != self.prompt_boilerplate_count
            || self.refused_as_stopword_deletion_count != self.prompt_boilerplate_count
            || self.inference_status != PROMPT_SOURCE_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidPromptSourceArtifact);
        }
        Ok(())
    }
}

/// One completed prompt-source artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct PromptSourceExecution {
    /// Digest-bound completed prompt-source census.
    pub artifact: PromptSourceArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe prompt-boilerplate refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_prompt_as_unique_content`] and
/// [`refuse_prompt_as_stopword_deletion`] already on protected main. It does
/// not emit `identity_recovery_rate`, a `scientific_acceptance` inspect
/// metric, GPU kernels, MCMC, or topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-kind admitted corpus, duplicate admitted document identity,
/// oversized raw corpus, or invalid artifact error.
pub fn execute_prompt_source_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    documents: &[PromptSourceDocument],
    completed_at: impl Into<String>,
) -> Result<PromptSourceExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != PROMPT_SOURCE_MODEL_CONTRACT_VERSION
        || request.output_profile != PROMPT_SOURCE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if documents.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut unique_content_count = 0_u64;
    let mut prompt_boilerplate_count = 0_u64;
    let mut refused_as_unique_content_count = 0_u64;
    let mut refused_as_stopword_deletion_count = 0_u64;
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
        match document.kind() {
            PromptKind::UniqueContent => {
                refuse_prompt_as_unique_content(document.kind()).map_err(map_prompt_error)?;
                refuse_prompt_as_stopword_deletion(document.kind()).map_err(map_prompt_error)?;
                unique_content_count = unique_content_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            PromptKind::PromptBoilerplate => {
                match refuse_prompt_as_unique_content(document.kind()) {
                    Err(PromptSourceError::PromptIsNotUniqueContent) => {
                        refused_as_unique_content_count = refused_as_unique_content_count
                            .checked_add(1)
                            .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                match refuse_prompt_as_stopword_deletion(document.kind()) {
                    Err(PromptSourceError::PromptIsNotStopwordDeletion) => {
                        refused_as_stopword_deletion_count = refused_as_stopword_deletion_count
                            .checked_add(1)
                            .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                prompt_boilerplate_count = prompt_boilerplate_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
        }
    }
    let document_count =
        u64::try_from(seen.len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    if document_count < 2 || unique_content_count == 0 || prompt_boilerplate_count == 0 {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = PromptSourceArtifact {
        schema_version: PROMPT_SOURCE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        document_count,
        unique_content_count,
        prompt_boilerplate_count,
        refused_as_unique_content_count,
        refused_as_stopword_deletion_count,
        inference_status: PROMPT_SOURCE_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new("prompt_source", document_count, 4, "validated")?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("prompt_source_artifact_{}", &digest[..16]),
        digest,
        PROMPT_SOURCE_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(PromptSourceExecution {
        artifact,
        terminal_result,
    })
}

fn map_prompt_error(error: PromptSourceError) -> AnalysisEngineError {
    match error {
        PromptSourceError::PromptIsNotUniqueContent
        | PromptSourceError::PromptIsNotStopwordDeletion
        | PromptSourceError::InvalidPromptPayload
        | _ => AnalysisEngineError::InvalidEvidence,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PROMPT_SOURCE_ARTIFACT_BYTE_LIMIT, PROMPT_SOURCE_ARTIFACT_SCHEMA_VERSION,
        PROMPT_SOURCE_INFERENCE_STATUS, PromptSourceArtifact,
    };
    use crate::{AnalysisEngineError, MAX_EVIDENCE_UNITS};

    fn artifact() -> PromptSourceArtifact {
        PromptSourceArtifact {
            schema_version: PROMPT_SOURCE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            document_count: 3,
            unique_content_count: 1,
            prompt_boilerplate_count: 2,
            refused_as_unique_content_count: 2,
            refused_as_stopword_deletion_count: 2,
            inference_status: PROMPT_SOURCE_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &PromptSourceArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidPromptSourceArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            PromptSourceArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            PromptSourceArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidPromptSourceArtifact)
        );
        assert_eq!(
            PromptSourceArtifact::from_json(&"x".repeat(PROMPT_SOURCE_ARTIFACT_BYTE_LIMIT + 1)),
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
                value.unique_content_count = value.document_count - 1;
                value.prompt_boilerplate_count = 1;
                value.refused_as_unique_content_count = 1;
                value.refused_as_stopword_deletion_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.unique_content_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.prompt_boilerplate_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.unique_content_count = u64::MAX;
                value.prompt_boilerplate_count = 1;
                value.document_count = u64::MAX;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_unique_content_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_stopword_deletion_count = 1;
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
