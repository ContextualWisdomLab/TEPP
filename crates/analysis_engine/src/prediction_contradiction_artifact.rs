//! Digest-bound prediction-contradiction refusals as an analysis-run profile.

use std::collections::BTreeSet;

use prediction_contradiction::{PredictionContradictionError, refuse_promotion};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::{AvailableTime, EventTime, KnowledgeCutoff, TemporalInterval};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};

use crate::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, format_digest, require_receipt_identity,
    valid_identifier,
};

/// Versioned schema for a completed prediction-contradiction artifact.
pub const PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION: &str =
    "tepp.prediction_contradiction.v1";
/// Model contract required by the prediction-contradiction execution path.
pub const PREDICTION_CONTRADICTION_MODEL_CONTRACT_VERSION: &str = "prediction_contradiction_v1";
/// Analysis-run output profile required for a prediction-contradiction artifact.
pub const PREDICTION_CONTRADICTION_OUTPUT_PROFILE: &str = "prediction_contradiction_v1";
/// Maximum canonical artifact JSON size.
pub const PREDICTION_CONTRADICTION_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const PREDICTION_CONTRADICTION_INFERENCE_STATUS: &str = "unmatched_prediction_is_not_observed";

/// One cutoff-admitted predicted-versus-observed interval pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredictionContradictionAssignment {
    assignment_id: String,
    predicted: TemporalInterval<EventTime>,
    observed: TemporalInterval<EventTime>,
    available_time: AvailableTime,
}

impl PredictionContradictionAssignment {
    /// Construct a bounded prediction-contradiction assignment.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the assignment
    /// identity is empty or oversized.
    pub fn new(
        assignment_id: impl Into<String>,
        predicted: TemporalInterval<EventTime>,
        observed: TemporalInterval<EventTime>,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let assignment_id = assignment_id.into();
        if !valid_identifier(&assignment_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            assignment_id,
            predicted,
            observed,
            available_time,
        })
    }

    /// Return the opaque assignment identity.
    #[must_use]
    pub fn assignment_id(&self) -> &str {
        &self.assignment_id
    }

    /// Return the predicted event-time interval.
    #[must_use]
    pub const fn predicted(&self) -> TemporalInterval<EventTime> {
        self.predicted
    }

    /// Return the observed event-time interval.
    #[must_use]
    pub const fn observed(&self) -> TemporalInterval<EventTime> {
        self.observed
    }

    /// Return the availability time used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded prediction-contradiction census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PredictionContradictionArtifact {
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
    /// Predicted intervals covered by later-observed evidence.
    pub covered_count: u64,
    /// Predicted intervals with interior overlap but unmatched mass.
    pub partial_overlap_count: u64,
    /// Predicted intervals that only meet observed evidence.
    pub adjacent_count: u64,
    /// Predicted intervals disjoint from observed evidence.
    pub contradictory_count: u64,
    /// Predicted intervals refused as observed fact.
    pub refused_promotion_count: u64,
    /// Fixed claim boundary for operator copy.
    pub inference_status: String,
}

impl PredictionContradictionArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidPredictionContradictionArtifact`]
    /// when the schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > PREDICTION_CONTRADICTION_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidPredictionContradictionArtifact)?;
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
        if payload.len() > PREDICTION_CONTRADICTION_ARTIFACT_BYTE_LIMIT {
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
            .covered_count
            .checked_add(self.partial_overlap_count)
            .and_then(|sum| sum.checked_add(self.adjacent_count))
            .and_then(|sum| sum.checked_add(self.contradictory_count));
        let refused_sum = self
            .partial_overlap_count
            .checked_add(self.adjacent_count)
            .and_then(|sum| sum.checked_add(self.contradictory_count));
        if self.schema_version != PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.assignment_count < 4
            || self.assignment_count > MAX_EVIDENCE_UNITS as u64
            || self.covered_count == 0
            || self.partial_overlap_count == 0
            || self.adjacent_count == 0
            || self.contradictory_count == 0
            || kind_sum != Some(self.assignment_count)
            || refused_sum != Some(self.refused_promotion_count)
            || self.refused_promotion_count.checked_add(self.covered_count)
                != Some(self.assignment_count)
            || self.inference_status != PREDICTION_CONTRADICTION_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidPredictionContradictionArtifact);
        }
        Ok(())
    }
}

/// One completed prediction-contradiction artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct PredictionContradictionExecution {
    /// Digest-bound completed prediction-contradiction census.
    pub artifact: PredictionContradictionArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe prediction-contradiction refusals as one analysis-run
/// profile.
///
/// The executor invokes [`refuse_promotion`] already on protected main.
/// Covered pairs may authorize promotion. Partial overlap, adjacency, and
/// Allen `before` / `after` stay hypothetical. `contradiction_agreement_rate`
/// stays library-side. It does not emit a `scientific_acceptance` inspect
/// metric, GPU kernels, MCMC, or topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// incomplete mixed-kind corpus, missing refusal, duplicate assignment
/// identity, oversized corpus, or invalid artifact error.
pub fn execute_prediction_contradiction_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    assignments: &[PredictionContradictionAssignment],
    completed_at: impl Into<String>,
) -> Result<PredictionContradictionExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != PREDICTION_CONTRADICTION_MODEL_CONTRACT_VERSION
        || request.output_profile != PREDICTION_CONTRADICTION_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if assignments.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let census = census_admitted_assignments(assignments, knowledge_cutoff)?;
    let artifact = PredictionContradictionArtifact {
        schema_version: PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        assignment_count: census.assignment_count,
        covered_count: census.covered_count,
        partial_overlap_count: census.partial_overlap_count,
        adjacent_count: census.adjacent_count,
        contradictory_count: census.contradictory_count,
        refused_promotion_count: census.refused_promotion_count,
        inference_status: PREDICTION_CONTRADICTION_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "prediction_contradiction",
        census.assignment_count,
        4,
        "validated",
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("prediction_contradiction_artifact_{}", &digest[..16]),
        digest,
        PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(PredictionContradictionExecution {
        artifact,
        terminal_result,
    })
}

#[allow(clippy::struct_field_names)]
struct PredictionContradictionCensus {
    assignment_count: u64,
    covered_count: u64,
    partial_overlap_count: u64,
    adjacent_count: u64,
    contradictory_count: u64,
    refused_promotion_count: u64,
}

fn census_admitted_assignments(
    assignments: &[PredictionContradictionAssignment],
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<PredictionContradictionCensus, AnalysisEngineError> {
    let mut seen = BTreeSet::new();
    let mut covered_count = 0_u64;
    let mut partial_overlap_count = 0_u64;
    let mut adjacent_count = 0_u64;
    let mut contradictory_count = 0_u64;
    let mut refused_promotion_count = 0_u64;
    for assignment in assignments {
        if assignment.available_time().instant() > knowledge_cutoff.instant() {
            continue;
        }
        if !seen.insert(assignment.assignment_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        classify_assignment(
            assignment,
            knowledge_cutoff,
            &mut covered_count,
            &mut partial_overlap_count,
            &mut adjacent_count,
            &mut contradictory_count,
            &mut refused_promotion_count,
        )?;
    }

    let assignment_count = covered_count
        .checked_add(partial_overlap_count)
        .and_then(|sum| sum.checked_add(adjacent_count))
        .and_then(|sum| sum.checked_add(contradictory_count))
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if assignment_count < 4
        || covered_count == 0
        || partial_overlap_count == 0
        || adjacent_count == 0
        || contradictory_count == 0
        || refused_promotion_count
            .checked_add(covered_count)
            .ok_or(AnalysisEngineError::ArithmeticOverflow)?
            != assignment_count
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    Ok(PredictionContradictionCensus {
        assignment_count,
        covered_count,
        partial_overlap_count,
        adjacent_count,
        contradictory_count,
        refused_promotion_count,
    })
}

fn classify_assignment(
    assignment: &PredictionContradictionAssignment,
    knowledge_cutoff: KnowledgeCutoff,
    covered_count: &mut u64,
    partial_overlap_count: &mut u64,
    adjacent_count: &mut u64,
    contradictory_count: &mut u64,
    refused_promotion_count: &mut u64,
) -> Result<(), AnalysisEngineError> {
    match refuse_promotion(
        &assignment.predicted(),
        &assignment.observed(),
        assignment.available_time(),
        knowledge_cutoff,
    ) {
        Ok(()) => *covered_count = increment(*covered_count)?,
        Err(PredictionContradictionError::PredictionNotCoveredByObservation) => {
            *partial_overlap_count = increment(*partial_overlap_count)?;
            *refused_promotion_count = increment(*refused_promotion_count)?;
        }
        Err(PredictionContradictionError::PredictionLacksOverlappingSupport) => {
            *adjacent_count = increment(*adjacent_count)?;
            *refused_promotion_count = increment(*refused_promotion_count)?;
        }
        Err(PredictionContradictionError::PredictionContradictsObservation) => {
            *contradictory_count = increment(*contradictory_count)?;
            *refused_promotion_count = increment(*refused_promotion_count)?;
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
        PREDICTION_CONTRADICTION_ARTIFACT_BYTE_LIMIT,
        PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION,
        PREDICTION_CONTRADICTION_INFERENCE_STATUS, PredictionContradictionArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> PredictionContradictionArtifact {
        PredictionContradictionArtifact {
            schema_version: PREDICTION_CONTRADICTION_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            assignment_count: 4,
            covered_count: 1,
            partial_overlap_count: 1,
            adjacent_count: 1,
            contradictory_count: 1,
            refused_promotion_count: 3,
            inference_status: PREDICTION_CONTRADICTION_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &PredictionContradictionArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidPredictionContradictionArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            PredictionContradictionArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            PredictionContradictionArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidPredictionContradictionArtifact)
        );
        assert_eq!(
            PredictionContradictionArtifact::from_json(
                &"x".repeat(PREDICTION_CONTRADICTION_ARTIFACT_BYTE_LIMIT + 1)
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
                value.assignment_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.covered_count = 0;
                value.assignment_count = 3;
                value.refused_promotion_count = 3;
                value
            },
            {
                let mut value = artifact.clone();
                value.partial_overlap_count = 0;
                value.assignment_count = 3;
                value.refused_promotion_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.adjacent_count = 0;
                value.assignment_count = 3;
                value.refused_promotion_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.contradictory_count = 0;
                value.assignment_count = 3;
                value.refused_promotion_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.refused_promotion_count = 0;
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
