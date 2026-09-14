//! Topic-context posterior analysis-run boundary.
//!
//! The producer-owned artifact schema remains in the adjacent base module;
//! this boundary adds analysis-run temporal and snapshot-manifest admission.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

mod base {
    include!("topic_context_posterior_base.rs");
}

pub use base::{
    TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT, TOPIC_CONTEXT_POSTERIOR_MODEL_CONTRACT_VERSION,
    TOPIC_CONTEXT_POSTERIOR_OUTPUT_PROFILE, TOPIC_CONTEXT_POSTERIOR_PRODUCER_CONTRACT_VERSION,
    TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION, TopicActivityInterval, TopicContextMembership,
    TopicContextPosteriorArtifact, TopicContextPosteriorExecution, TopicDocumentRelation,
    TopicLineageEvent, TopicPostPlausibleValue,
};

const MANIFEST_ENTRY_LIMIT: usize = 1_000_000;
const TOPIC_CONTEXT_POSTERIOR_INFERENCE_STATUS: &str =
    "posterior_topic_coordinates_not_importance";

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn canonical_cutoff(value: &str) -> Option<KnowledgeCutoff> {
    KnowledgeCutoff::parse_rfc3339(value)
        .ok()
        .filter(|instant| instant.to_rfc3339() == value)
}

fn canonical_available_time(value: &str) -> Option<AvailableTime> {
    AvailableTime::parse_rfc3339(value)
        .ok()
        .filter(|instant| instant.to_rfc3339() == value)
}

/// Authoritative snapshot and cutoff-eligibility manifest for one artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TopicContextPosteriorSnapshotManifest {
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Canonical digest of the resolved source snapshot bytes.
    pub source_snapshot_sha256: String,
    /// Historical cutoff applied while resolving eligibility.
    pub knowledge_cutoff: String,
    /// Canonical digest of the exact artifact admitted from this snapshot.
    pub artifact_sha256: String,
    /// Availability instant for every document represented by the artifact.
    pub document_available_at: BTreeMap<String, String>,
}

impl TopicContextPosteriorSnapshotManifest {
    fn validate(&self) -> Result<(), AnalysisEngineError> {
        if !valid_identifier(&self.snapshot_id)
            || !digest(&self.source_snapshot_sha256)
            || !digest(&self.artifact_sha256)
            || canonical_cutoff(&self.knowledge_cutoff).is_none()
            || self.document_available_at.len() > MANIFEST_ENTRY_LIMIT
            || self.document_available_at.iter().any(|(document_id, available_at)| {
                !valid_identifier(document_id)
                    || canonical_available_time(available_at).is_none()
            })
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(())
    }

    /// Parse and validate one bounded snapshot manifest.
    ///
    /// # Errors
    ///
    /// Returns a size, wire, or evidence error for an oversized or malformed
    /// manifest.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let manifest: Self =
            serde_json::from_str(payload).map_err(|_| AnalysisEngineError::InvalidEvidence)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Serialize one validated snapshot manifest to bounded canonical JSON.
    ///
    /// # Errors
    ///
    /// Returns a validation, serialization, or size error.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        if payload.len() > TOPIC_CONTEXT_POSTERIOR_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        Ok(payload)
    }
}

/// Execute posterior topic-context validation as one analysis-run profile.
///
/// The executor validates an already-constructed producer artifact. Request
/// cutoffs are compared by temporal instant; persisted artifact and manifest
/// cutoffs remain canonical RFC 3339 evidence.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error or a producer
/// contract refusal.
pub fn execute_topic_context_posterior_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    manifest: &TopicContextPosteriorSnapshotManifest,
    artifact: &TopicContextPosteriorArtifact,
    completed_at: impl Into<String>,
) -> Result<TopicContextPosteriorExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    manifest.validate()?;

    if request.snapshot_id != manifest.snapshot_id || artifact.snapshot_id != manifest.snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }

    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    let manifest_cutoff = canonical_cutoff(&manifest.knowledge_cutoff)
        .ok_or(AnalysisEngineError::InvalidEvidence)?;
    let artifact_cutoff = canonical_cutoff(&artifact.knowledge_cutoff)
        .ok_or(AnalysisEngineError::InvalidEvidence)?;

    if request_cutoff.instant() != manifest_cutoff.instant()
        || artifact_cutoff.instant() != manifest_cutoff.instant()
        || artifact.source_snapshot_sha256 != manifest.source_snapshot_sha256
        || request.model_contract_version != TOPIC_CONTEXT_POSTERIOR_MODEL_CONTRACT_VERSION
        || request.output_profile != TOPIC_CONTEXT_POSTERIOR_OUTPUT_PROFILE
        || artifact.model_contract_version != TOPIC_CONTEXT_POSTERIOR_PRODUCER_CONTRACT_VERSION
        || artifact.run_id != accepted.run_id
        || artifact.inference_status != TOPIC_CONTEXT_POSTERIOR_INFERENCE_STATUS
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact_digest = artifact.sha256()?;
    if artifact_digest != manifest.artifact_sha256 {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let document_ids: BTreeSet<&str> = artifact
        .plausible_values
        .iter()
        .map(|value| value.document_id.as_str())
        .collect();
    if document_ids.len() != manifest.document_available_at.len()
        || document_ids.iter().any(|document_id| {
            manifest
                .document_available_at
                .get(*document_id)
                .and_then(|available_at| canonical_available_time(available_at))
                .is_none_or(|available_at| available_at.instant() > manifest_cutoff.instant())
        })
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let document_count = u64::try_from(document_ids.len())
        .map_err(|_| AnalysisEngineError::LimitExceeded)?;
    let statistic_count = artifact.plausible_values.iter().try_fold(0_u64, |total, value| {
        let coordinates = u64::try_from(value.logistic_normal_coordinates.len())
            .map_err(|_| AnalysisEngineError::LimitExceeded)?;
        total
            .checked_add(coordinates)
            .ok_or(AnalysisEngineError::LimitExceeded)
    })?;
    let summary = AnalysisResultSummary {
        analysis_family: "topic_context_posterior".into(),
        evidence_count: document_count,
        statistic_count,
        validation_status: "validated".into(),
    };
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("topic_context_posterior_artifact_{}", &artifact_digest[..16]),
        artifact_digest,
        TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(TopicContextPosteriorExecution {
        artifact: artifact.clone(),
        terminal_result,
    })
}
