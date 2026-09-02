//! Digest-bound inferred-status refusals as an analysis-run profile.

use inferred_status::{
    EvidenceStatus, InferredStatusError, refuse_inferred_as_observed, refuse_inferred_as_transition,
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

/// Versioned schema for a completed inferred-status artifact.
pub const INFERRED_STATUS_ARTIFACT_SCHEMA_VERSION: &str = "tepp.inferred_status.v1";
/// Model contract required by the inferred-status execution path.
pub const INFERRED_STATUS_MODEL_CONTRACT_VERSION: &str = "inferred_status_v1";
/// Analysis-run output profile required for an inferred-status artifact.
pub const INFERRED_STATUS_OUTPUT_PROFILE: &str = "inferred_status_v1";
/// Maximum canonical artifact JSON size.
pub const INFERRED_STATUS_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const INFERRED_STATUS_INFERENCE_STATUS: &str = "inferred_is_not_observed_and_not_transition";

/// One cutoff-admitted evidence row with closed observed/inferred status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferredStatusEvidence {
    evidence_id: String,
    status: EvidenceStatus,
    available_time: AvailableTime,
}

impl InferredStatusEvidence {
    /// Construct a bounded inferred-status evidence row.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the evidence
    /// identity is empty or oversized.
    pub fn new(
        evidence_id: impl Into<String>,
        status: EvidenceStatus,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let evidence_id = evidence_id.into();
        if !valid_identifier(&evidence_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            evidence_id,
            status,
            available_time,
        })
    }

    /// Return the opaque evidence identity.
    #[must_use]
    pub fn evidence_id(&self) -> &str {
        &self.evidence_id
    }

    /// Return the closed observed/inferred status.
    #[must_use]
    pub const fn status(&self) -> EvidenceStatus {
        self.status
    }

    /// Return the availability time used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded inferred-status census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InferredStatusArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used to admit rows.
    pub knowledge_cutoff: String,
    /// Number of evidence rows admitted at the cutoff.
    pub evidence_count: u64,
    /// Directly observed rows admitted at the cutoff.
    pub observed_count: u64,
    /// Inferred rows admitted at the cutoff.
    pub inferred_count: u64,
    /// Inferred rows refused as observed evidence.
    pub refused_as_observed_count: u64,
    /// Inferred rows refused as state transitions.
    pub refused_as_transition_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl InferredStatusArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidInferredStatusArtifact`] when the
    /// schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > INFERRED_STATUS_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidInferredStatusArtifact)?;
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
        if payload.len() > INFERRED_STATUS_ARTIFACT_BYTE_LIMIT {
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
        let status_sum = self.observed_count.checked_add(self.inferred_count);
        if self.schema_version != INFERRED_STATUS_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.evidence_count < 2
            || self.evidence_count > MAX_EVIDENCE_UNITS as u64
            || self.observed_count == 0
            || self.inferred_count == 0
            || status_sum != Some(self.evidence_count)
            || self.refused_as_observed_count != self.inferred_count
            || self.refused_as_transition_count != self.inferred_count
            || self.inference_status != INFERRED_STATUS_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidInferredStatusArtifact);
        }
        Ok(())
    }
}

/// One completed inferred-status artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct InferredStatusExecution {
    /// Digest-bound completed inferred-status census.
    pub artifact: InferredStatusArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe inferred-status refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_inferred_as_observed`] and
/// [`refuse_inferred_as_transition`] already on protected main. Observed
/// statuses stay observed. Inferred statuses stay refusals, never observed
/// evidence and never transitions. It does not emit
/// `identity_recovery_rate`, a `scientific_acceptance` inspect metric, GPU
/// kernels, MCMC, or topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-class corpus, inferred treated as observed or transition,
/// duplicate evidence identity, oversized corpus, or invalid artifact
/// error.
pub fn execute_inferred_status_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    evidence: &[InferredStatusEvidence],
    completed_at: impl Into<String>,
) -> Result<InferredStatusExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != INFERRED_STATUS_MODEL_CONTRACT_VERSION
        || request.output_profile != INFERRED_STATUS_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if evidence.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let (observed_count, inferred_count, refused_as_observed_count, refused_as_transition_count) =
        census_evidence(evidence, knowledge_cutoff)?;
    let evidence_count = observed_count
        .checked_add(inferred_count)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if evidence_count < 2
        || observed_count == 0
        || inferred_count == 0
        || refused_as_observed_count != inferred_count
        || refused_as_transition_count != inferred_count
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = InferredStatusArtifact {
        schema_version: INFERRED_STATUS_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        evidence_count,
        observed_count,
        inferred_count,
        refused_as_observed_count,
        refused_as_transition_count,
        inference_status: INFERRED_STATUS_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new("inferred_status", evidence_count, 4, "validated")?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("inferred_status_artifact_{}", &digest[..16]),
        digest,
        INFERRED_STATUS_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(InferredStatusExecution {
        artifact,
        terminal_result,
    })
}

fn census_evidence(
    evidence: &[InferredStatusEvidence],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<(u64, u64, u64, u64), AnalysisEngineError> {
    let mut seen = std::collections::BTreeSet::new();
    let mut observed_count = 0_u64;
    let mut inferred_count = 0_u64;
    let mut refused_as_observed_count = 0_u64;
    let mut refused_as_transition_count = 0_u64;
    for row in evidence {
        if !seen.insert(row.evidence_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        if row.available_time().instant() > knowledge_cutoff.instant() {
            continue;
        }
        match row.status() {
            EvidenceStatus::Observed => {
                refuse_inferred_as_observed(row.status()).map_err(map_inferred_status_error)?;
                refuse_inferred_as_transition(row.status()).map_err(map_inferred_status_error)?;
                observed_count = increment(observed_count)?;
            }
            EvidenceStatus::Inferred => {
                match refuse_inferred_as_observed(row.status()) {
                    Err(InferredStatusError::InferredIsNotObserved) => {
                        refused_as_observed_count = increment(refused_as_observed_count)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                match refuse_inferred_as_transition(row.status()) {
                    Err(InferredStatusError::InferredIsNotTransition) => {
                        refused_as_transition_count = increment(refused_as_transition_count)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                inferred_count = increment(inferred_count)?;
            }
        }
    }
    Ok((
        observed_count,
        inferred_count,
        refused_as_observed_count,
        refused_as_transition_count,
    ))
}

fn increment(count: u64) -> Result<u64, AnalysisEngineError> {
    count
        .checked_add(1)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)
}

fn map_inferred_status_error(error: InferredStatusError) -> AnalysisEngineError {
    match error {
        InferredStatusError::InferredIsNotObserved
        | InferredStatusError::InferredIsNotTransition
        | InferredStatusError::InvalidStatusPayload
        | _ => AnalysisEngineError::InvalidEvidence,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        INFERRED_STATUS_ARTIFACT_BYTE_LIMIT, INFERRED_STATUS_ARTIFACT_SCHEMA_VERSION,
        INFERRED_STATUS_INFERENCE_STATUS, InferredStatusArtifact,
    };
    use crate::{AnalysisEngineError, MAX_EVIDENCE_UNITS};

    fn artifact() -> InferredStatusArtifact {
        InferredStatusArtifact {
            schema_version: INFERRED_STATUS_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            evidence_count: 2,
            observed_count: 1,
            inferred_count: 1,
            refused_as_observed_count: 1,
            refused_as_transition_count: 1,
            inference_status: INFERRED_STATUS_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &InferredStatusArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidInferredStatusArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            InferredStatusArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            InferredStatusArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidInferredStatusArtifact)
        );
        assert_eq!(
            InferredStatusArtifact::from_json(&"x".repeat(INFERRED_STATUS_ARTIFACT_BYTE_LIMIT + 1)),
            Err(AnalysisEngineError::LimitExceeded)
        );
    }

    #[test]
    fn artifact_claimed_count_above_execution_limit_fails_parse_and_serialize() {
        let mut oversized = artifact();
        oversized.evidence_count = MAX_EVIDENCE_UNITS as u64 + 1;
        oversized.observed_count = 1;
        oversized.inferred_count = MAX_EVIDENCE_UNITS as u64;
        oversized.refused_as_observed_count = MAX_EVIDENCE_UNITS as u64;
        oversized.refused_as_transition_count = MAX_EVIDENCE_UNITS as u64;

        assert_eq!(
            oversized.to_json(),
            Err(AnalysisEngineError::InvalidInferredStatusArtifact)
        );
        let unchecked_payload = serde_json::to_string(&oversized).expect("unchecked fixture json");
        assert_eq!(
            InferredStatusArtifact::from_json(&unchecked_payload),
            Err(AnalysisEngineError::InvalidInferredStatusArtifact)
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
                value.evidence_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.observed_count = 0;
                value.evidence_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.inferred_count = 0;
                value.refused_as_observed_count = 0;
                value.refused_as_transition_count = 0;
                value.evidence_count = 1;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_observed_count = 0;
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
