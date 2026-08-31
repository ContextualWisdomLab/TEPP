//! Digest-bound corpus-background refusals as an analysis-run profile.

use corpus_background::{
    CorpusBackgroundError, CorpusBackgroundKind, refuse_corpus_background_as_stopword_deletion,
    refuse_corpus_background_as_unique_content,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed corpus-background artifact.
pub const CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION: &str = "tepp.corpus_background.v1";
/// Model contract required by the corpus-background execution path.
pub const CORPUS_BACKGROUND_MODEL_CONTRACT_VERSION: &str = "corpus_background_v1";
/// Analysis-run output profile required for a corpus-background artifact.
pub const CORPUS_BACKGROUND_OUTPUT_PROFILE: &str = "corpus_background_v1";
/// Maximum canonical artifact JSON size.
pub const CORPUS_BACKGROUND_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const CORPUS_BACKGROUND_INFERENCE_STATUS: &str =
    "corpus_background_is_not_unique_content_not_stopword_deletion";

/// One cutoff-admitted token treatment with a closed corpus-background kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusBackgroundDocument {
    document_id: String,
    kind: CorpusBackgroundKind,
}

impl CorpusBackgroundDocument {
    /// Construct a bounded corpus-background document.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the document
    /// identity is empty or oversized.
    pub fn new(
        document_id: impl Into<String>,
        kind: CorpusBackgroundKind,
    ) -> Result<Self, AnalysisEngineError> {
        let document_id = document_id.into();
        if !valid_identifier(&document_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self { document_id, kind })
    }

    /// Return the opaque document identity.
    #[must_use]
    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    /// Return the closed corpus-background kind.
    #[must_use]
    pub const fn kind(&self) -> CorpusBackgroundKind {
        self.kind
    }
}

/// Completed, bounded corpus-background census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusBackgroundArtifact {
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
    /// Corpus-background treatments admitted at the cutoff.
    pub corpus_background_count: u64,
    /// Corpus-background wording refused as unique latent content.
    pub refused_as_unique_content_count: u64,
    /// Corpus-background wording refused as stopword deletion.
    pub refused_as_stopword_deletion_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl CorpusBackgroundArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidCorpusBackgroundArtifact`] when
    /// the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > CORPUS_BACKGROUND_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidCorpusBackgroundArtifact)?;
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
        if payload.len() > CORPUS_BACKGROUND_ARTIFACT_BYTE_LIMIT {
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
            .checked_add(self.corpus_background_count);
        if self.schema_version != CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.document_count < 2
            || self.unique_content_count == 0
            || self.corpus_background_count == 0
            || kind_sum != Some(self.document_count)
            || self.refused_as_unique_content_count != self.corpus_background_count
            || self.refused_as_stopword_deletion_count != self.corpus_background_count
            || self.inference_status != CORPUS_BACKGROUND_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidCorpusBackgroundArtifact);
        }
        Ok(())
    }
}

/// One completed corpus-background artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct CorpusBackgroundExecution {
    /// Digest-bound completed corpus-background census.
    pub artifact: CorpusBackgroundArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe corpus-background refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_corpus_background_as_unique_content`] and
/// [`refuse_corpus_background_as_stopword_deletion`] already on protected
/// main. It does not emit `identity_recovery_rate`, a
/// `scientific_acceptance` inspect metric, GPU kernels, MCMC, or topic
/// birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-kind corpus, duplicate document identity, or invalid artifact error.
pub fn execute_corpus_background_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    documents: &[CorpusBackgroundDocument],
    completed_at: impl Into<String>,
) -> Result<CorpusBackgroundExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != CORPUS_BACKGROUND_MODEL_CONTRACT_VERSION
        || request.output_profile != CORPUS_BACKGROUND_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut unique_content_count = 0_u64;
    let mut corpus_background_count = 0_u64;
    let mut refused_as_unique_content_count = 0_u64;
    let mut refused_as_stopword_deletion_count = 0_u64;
    for document in documents {
        if !seen.insert(document.document_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        match document.kind() {
            CorpusBackgroundKind::UniqueContent => {
                refuse_corpus_background_as_unique_content(document.kind())
                    .map_err(map_background_error)?;
                refuse_corpus_background_as_stopword_deletion(document.kind())
                    .map_err(map_background_error)?;
                unique_content_count = unique_content_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
            CorpusBackgroundKind::CorpusBackground => {
                match refuse_corpus_background_as_unique_content(document.kind()) {
                    Err(CorpusBackgroundError::CorpusBackgroundIsNotUniqueContent) => {
                        refused_as_unique_content_count = refused_as_unique_content_count
                            .checked_add(1)
                            .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                match refuse_corpus_background_as_stopword_deletion(document.kind()) {
                    Err(CorpusBackgroundError::CorpusBackgroundIsNotStopwordDeletion) => {
                        refused_as_stopword_deletion_count = refused_as_stopword_deletion_count
                            .checked_add(1)
                            .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                corpus_background_count = corpus_background_count
                    .checked_add(1)
                    .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
            }
        }
    }
    let document_count =
        u64::try_from(documents.len()).map_err(|_| AnalysisEngineError::ArithmeticOverflow)?;
    if document_count < 2 || unique_content_count == 0 || corpus_background_count == 0 {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = CorpusBackgroundArtifact {
        schema_version: CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        document_count,
        unique_content_count,
        corpus_background_count,
        refused_as_unique_content_count,
        refused_as_stopword_deletion_count,
        inference_status: CORPUS_BACKGROUND_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "corpus_background",
        document_count,
        4,
        CORPUS_BACKGROUND_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("corpus_background_artifact_{}", &digest[..16]),
        digest,
        CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(CorpusBackgroundExecution {
        artifact,
        terminal_result,
    })
}

fn map_background_error(error: CorpusBackgroundError) -> AnalysisEngineError {
    match error {
        CorpusBackgroundError::CorpusBackgroundIsNotUniqueContent
        | CorpusBackgroundError::CorpusBackgroundIsNotStopwordDeletion
        | CorpusBackgroundError::InvalidCorpusBackgroundPayload
        | _ => AnalysisEngineError::InvalidEvidence,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CORPUS_BACKGROUND_ARTIFACT_BYTE_LIMIT, CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION,
        CORPUS_BACKGROUND_INFERENCE_STATUS, CorpusBackgroundArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> CorpusBackgroundArtifact {
        CorpusBackgroundArtifact {
            schema_version: CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            document_count: 3,
            unique_content_count: 1,
            corpus_background_count: 2,
            refused_as_unique_content_count: 2,
            refused_as_stopword_deletion_count: 2,
            inference_status: CORPUS_BACKGROUND_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &CorpusBackgroundArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidCorpusBackgroundArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            CorpusBackgroundArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            CorpusBackgroundArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidCorpusBackgroundArtifact)
        );
        assert_eq!(
            CorpusBackgroundArtifact::from_json(
                &"x".repeat(CORPUS_BACKGROUND_ARTIFACT_BYTE_LIMIT + 1)
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
                value.document_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.unique_content_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.corpus_background_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_unique_content_count = 1;
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
