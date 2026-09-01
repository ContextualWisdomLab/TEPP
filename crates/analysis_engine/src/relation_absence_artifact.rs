//! Digest-bound relation-absence refusals as an analysis-run profile.

use relation_absence::{ObservationStatus, RelationAbsenceError, refuse_absence_as_negative};
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

/// Versioned schema for a completed relation-absence artifact.
pub const RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION: &str = "tepp.relation_absence.v1";
/// Model contract required by the relation-absence execution path.
pub const RELATION_ABSENCE_MODEL_CONTRACT_VERSION: &str = "relation_absence_v1";
/// Analysis-run output profile required for a relation-absence artifact.
pub const RELATION_ABSENCE_OUTPUT_PROFILE: &str = "relation_absence_v1";
/// Maximum canonical artifact JSON size.
pub const RELATION_ABSENCE_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const RELATION_ABSENCE_INFERENCE_STATUS: &str =
    "unobserved_is_not_negative_observed_inferred_are_presence";

/// One cutoff-admitted relation pair with closed observation status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationAbsencePair {
    pair_id: String,
    status: ObservationStatus,
    available_time: AvailableTime,
}

impl RelationAbsencePair {
    /// Construct a bounded relation-absence pair.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the pair identity
    /// is empty or oversized.
    pub fn new(
        pair_id: impl Into<String>,
        status: ObservationStatus,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let pair_id = pair_id.into();
        if !valid_identifier(&pair_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            pair_id,
            status,
            available_time,
        })
    }

    /// Return the opaque pair identity.
    #[must_use]
    pub fn pair_id(&self) -> &str {
        &self.pair_id
    }

    /// Return the closed observation status.
    #[must_use]
    pub const fn status(&self) -> ObservationStatus {
        self.status
    }

    /// Return the availability time used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded relation-absence census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RelationAbsenceArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used to admit pairs.
    pub knowledge_cutoff: String,
    /// Number of pairs admitted at the cutoff.
    pub pair_count: u64,
    /// Directly observed pairs admitted at the cutoff.
    pub observed_count: u64,
    /// Inferred pairs admitted at the cutoff.
    pub inferred_count: u64,
    /// Unobserved pairs admitted at the cutoff.
    pub unobserved_count: u64,
    /// Unobserved pairs refused as negative evidence.
    pub refused_as_negative_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl RelationAbsenceArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidRelationAbsenceArtifact`] when the
    /// schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > RELATION_ABSENCE_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidRelationAbsenceArtifact)?;
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
        if payload.len() > RELATION_ABSENCE_ARTIFACT_BYTE_LIMIT {
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
        let presence_sum = self.observed_count.checked_add(self.inferred_count);
        let status_sum = presence_sum.and_then(|value| value.checked_add(self.unobserved_count));
        if self.schema_version != RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.pair_count < 3
            || self.pair_count > MAX_EVIDENCE_UNITS as u64
            || self.observed_count == 0
            || self.inferred_count == 0
            || self.unobserved_count == 0
            || status_sum != Some(self.pair_count)
            || self.refused_as_negative_count != self.unobserved_count
            || self.inference_status != RELATION_ABSENCE_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidRelationAbsenceArtifact);
        }
        Ok(())
    }
}

/// One completed relation-absence artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct RelationAbsenceExecution {
    /// Digest-bound completed relation-absence census.
    pub artifact: RelationAbsenceArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe relation-absence refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_absence_as_negative`] already on protected
/// main. Observed and inferred statuses stay presence. Unobserved stays a
/// missing-status, never a negative edge. It does not emit
/// `status_recovery_rate`, a `scientific_acceptance` inspect metric, GPU
/// kernels, MCMC, or topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-class corpus, unobserved treated as negative, duplicate pair
/// identity, oversized corpus, or invalid artifact error.
pub fn execute_relation_absence_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    pairs: &[RelationAbsencePair],
    completed_at: impl Into<String>,
) -> Result<RelationAbsenceExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != RELATION_ABSENCE_MODEL_CONTRACT_VERSION
        || request.output_profile != RELATION_ABSENCE_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if pairs.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut observed_count = 0_u64;
    let mut inferred_count = 0_u64;
    let mut unobserved_count = 0_u64;
    let mut refused_as_negative_count = 0_u64;
    for pair in pairs {
        if !seen.insert(pair.pair_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        if pair.available_time().instant() > knowledge_cutoff.instant() {
            continue;
        }
        match pair.status() {
            ObservationStatus::Observed => {
                refuse_absence_as_negative(pair.status()).map_err(map_relation_absence_error)?;
                observed_count = increment(observed_count)?;
            }
            ObservationStatus::Inferred => {
                refuse_absence_as_negative(pair.status()).map_err(map_relation_absence_error)?;
                inferred_count = increment(inferred_count)?;
            }
            ObservationStatus::Unobserved => {
                match refuse_absence_as_negative(pair.status()) {
                    Err(RelationAbsenceError::AbsenceIsNotNegative) => {
                        refused_as_negative_count = increment(refused_as_negative_count)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                unobserved_count = increment(unobserved_count)?;
            }
        }
    }

    let pair_count = observed_count
        .checked_add(inferred_count)
        .and_then(|value| value.checked_add(unobserved_count))
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if pair_count < 3
        || observed_count == 0
        || inferred_count == 0
        || unobserved_count == 0
        || refused_as_negative_count != unobserved_count
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = RelationAbsenceArtifact {
        schema_version: RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        pair_count,
        observed_count,
        inferred_count,
        unobserved_count,
        refused_as_negative_count,
        inference_status: RELATION_ABSENCE_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "relation_absence",
        pair_count,
        4,
        RELATION_ABSENCE_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("relation_absence_artifact_{}", &digest[..16]),
        digest,
        RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(RelationAbsenceExecution {
        artifact,
        terminal_result,
    })
}

fn increment(count: u64) -> Result<u64, AnalysisEngineError> {
    count
        .checked_add(1)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)
}

fn map_relation_absence_error(error: RelationAbsenceError) -> AnalysisEngineError {
    match error {
        RelationAbsenceError::AbsenceIsNotNegative
        | RelationAbsenceError::InvalidObservationPayload
        | _ => AnalysisEngineError::InvalidEvidence,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RELATION_ABSENCE_ARTIFACT_BYTE_LIMIT, RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION,
        RELATION_ABSENCE_INFERENCE_STATUS, RelationAbsenceArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> RelationAbsenceArtifact {
        RelationAbsenceArtifact {
            schema_version: RELATION_ABSENCE_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            pair_count: 3,
            observed_count: 1,
            inferred_count: 1,
            unobserved_count: 1,
            refused_as_negative_count: 1,
            inference_status: RELATION_ABSENCE_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &RelationAbsenceArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidRelationAbsenceArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            RelationAbsenceArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            RelationAbsenceArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidRelationAbsenceArtifact)
        );
        assert_eq!(
            RelationAbsenceArtifact::from_json(
                &"x".repeat(RELATION_ABSENCE_ARTIFACT_BYTE_LIMIT + 1)
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
                value.pair_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.observed_count = 0;
                value.pair_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.inferred_count = 0;
                value.pair_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.unobserved_count = 0;
                value.refused_as_negative_count = 0;
                value.pair_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_as_negative_count = 0;
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
