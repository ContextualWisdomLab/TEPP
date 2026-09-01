//! Digest-bound input-process-outcome order refusals as an analysis-run profile.

use outcome_order::{
    OutcomeKind, OutcomeOrderError, refuse_outcome_of_as_transition, refuse_reverse_ipo_order,
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

/// Versioned schema for a completed outcome-order artifact.
pub const OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION: &str = "tepp.outcome_order.v1";
/// Model contract required by the outcome-order execution path.
pub const OUTCOME_ORDER_MODEL_CONTRACT_VERSION: &str = "outcome_order_v1";
/// Analysis-run output profile required for an outcome-order artifact.
pub const OUTCOME_ORDER_OUTPUT_PROFILE: &str = "outcome_order_v1";
/// Maximum canonical artifact JSON size.
pub const OUTCOME_ORDER_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const OUTCOME_ORDER_INFERENCE_STATUS: &str = "input_process_forward_outcome_of_is_not_transition";

/// One cutoff-admitted IPO edge with closed kind and opaque event-time ranks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutcomeOrderEdge {
    edge_id: String,
    kind: OutcomeKind,
    source_rank: u64,
    target_rank: u64,
    available_time: AvailableTime,
}

impl OutcomeOrderEdge {
    /// Construct a bounded outcome-order edge.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidEvidence`] when the edge identity
    /// is empty or oversized.
    pub fn new(
        edge_id: impl Into<String>,
        kind: OutcomeKind,
        source_rank: u64,
        target_rank: u64,
        available_time: AvailableTime,
    ) -> Result<Self, AnalysisEngineError> {
        let edge_id = edge_id.into();
        if !valid_identifier(&edge_id) {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            edge_id,
            kind,
            source_rank,
            target_rank,
            available_time,
        })
    }

    /// Return the opaque edge identity.
    #[must_use]
    pub fn edge_id(&self) -> &str {
        &self.edge_id
    }

    /// Return the closed IPO kind.
    #[must_use]
    pub const fn kind(&self) -> OutcomeKind {
        self.kind
    }

    /// Return the opaque source event-time rank.
    #[must_use]
    pub const fn source_rank(&self) -> u64 {
        self.source_rank
    }

    /// Return the opaque target event-time rank.
    #[must_use]
    pub const fn target_rank(&self) -> u64 {
        self.target_rank
    }

    /// Return the availability time used for cutoff eligibility.
    #[must_use]
    pub const fn available_time(&self) -> AvailableTime {
        self.available_time
    }
}

/// Completed, bounded IPO-order census for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeOrderArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used to admit edges.
    pub knowledge_cutoff: String,
    /// Number of edges admitted at the cutoff.
    pub edge_count: u64,
    /// Forward `input_to` transitions admitted at the cutoff.
    pub input_to_count: u64,
    /// Forward `process_to` transitions admitted at the cutoff.
    pub process_to_count: u64,
    /// `outcome_of` provenance edges admitted at the cutoff.
    pub outcome_of_count: u64,
    /// Provenance edges refused as state transitions.
    pub refused_as_transition_count: u64,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl OutcomeOrderArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidOutcomeOrderArtifact`] when the
    /// schema, identifiers, counts, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > OUTCOME_ORDER_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidOutcomeOrderArtifact)?;
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
        if payload.len() > OUTCOME_ORDER_ARTIFACT_BYTE_LIMIT {
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
        let transition_sum = self.input_to_count.checked_add(self.process_to_count);
        let kind_sum = transition_sum.and_then(|value| value.checked_add(self.outcome_of_count));
        if self.schema_version != OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
            || self.edge_count < 3
            || self.edge_count > MAX_EVIDENCE_UNITS as u64
            || self.input_to_count == 0
            || self.process_to_count == 0
            || self.outcome_of_count == 0
            || kind_sum != Some(self.edge_count)
            || self.refused_as_transition_count != self.outcome_of_count
            || self.inference_status != OUTCOME_ORDER_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidOutcomeOrderArtifact);
        }
        Ok(())
    }
}

/// One completed outcome-order artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct OutcomeOrderExecution {
    /// Digest-bound completed IPO-order census.
    pub artifact: OutcomeOrderArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

/// Execute cutoff-safe IPO-order refusals as one analysis-run profile.
///
/// The executor invokes [`refuse_reverse_ipo_order`] and
/// [`refuse_outcome_of_as_transition`] already on protected main. `input_to`
/// and `process_to` stay forward transitions. `outcome_of` stays provenance.
/// It does not emit `kind_recovery_rate`, a `scientific_acceptance` inspect
/// metric, GPU kernels, MCMC, or topic birth/split/merge events.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, empty or
/// single-class corpus, reverse or uncertain IPO order, duplicate edge
/// identity, oversized corpus, or invalid artifact error.
#[allow(clippy::too_many_lines)]
pub fn execute_outcome_order_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    edges: &[OutcomeOrderEdge],
    completed_at: impl Into<String>,
) -> Result<OutcomeOrderExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != OUTCOME_ORDER_MODEL_CONTRACT_VERSION
        || request.output_profile != OUTCOME_ORDER_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }
    if edges.len() > MAX_EVIDENCE_UNITS {
        return Err(AnalysisEngineError::LimitExceeded);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut input_to_count = 0_u64;
    let mut process_to_count = 0_u64;
    let mut outcome_of_count = 0_u64;
    let mut refused_as_transition_count = 0_u64;
    for edge in edges {
        if !seen.insert(edge.edge_id()) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        if edge.available_time().instant() > knowledge_cutoff.instant() {
            continue;
        }
        match edge.kind() {
            OutcomeKind::InputTo => {
                refuse_reverse_ipo_order(edge.kind(), edge.source_rank(), edge.target_rank())
                    .map_err(map_outcome_order_error)?;
                refuse_outcome_of_as_transition(edge.kind()).map_err(map_outcome_order_error)?;
                input_to_count = increment(input_to_count)?;
            }
            OutcomeKind::ProcessTo => {
                refuse_reverse_ipo_order(edge.kind(), edge.source_rank(), edge.target_rank())
                    .map_err(map_outcome_order_error)?;
                refuse_outcome_of_as_transition(edge.kind()).map_err(map_outcome_order_error)?;
                process_to_count = increment(process_to_count)?;
            }
            OutcomeKind::OutcomeOf => {
                match refuse_outcome_of_as_transition(edge.kind()) {
                    Err(OutcomeOrderError::OutcomeOfIsNotTransition) => {
                        refused_as_transition_count = increment(refused_as_transition_count)?;
                    }
                    Ok(()) | Err(_) => return Err(AnalysisEngineError::InvalidEvidence),
                }
                outcome_of_count = increment(outcome_of_count)?;
            }
        }
    }

    let edge_count = input_to_count
        .checked_add(process_to_count)
        .and_then(|value| value.checked_add(outcome_of_count))
        .ok_or(AnalysisEngineError::ArithmeticOverflow)?;
    if edge_count < 3
        || input_to_count == 0
        || process_to_count == 0
        || outcome_of_count == 0
        || refused_as_transition_count != outcome_of_count
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let artifact = OutcomeOrderArtifact {
        schema_version: OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        edge_count,
        input_to_count,
        process_to_count,
        outcome_of_count,
        refused_as_transition_count,
        inference_status: OUTCOME_ORDER_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "outcome_order",
        edge_count,
        4,
        OUTCOME_ORDER_INFERENCE_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("outcome_order_artifact_{}", &digest[..16]),
        digest,
        OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(OutcomeOrderExecution {
        artifact,
        terminal_result,
    })
}

fn increment(count: u64) -> Result<u64, AnalysisEngineError> {
    count
        .checked_add(1)
        .ok_or(AnalysisEngineError::ArithmeticOverflow)
}

fn map_outcome_order_error(error: OutcomeOrderError) -> AnalysisEngineError {
    match error {
        OutcomeOrderError::ReverseIpoOrder
        | OutcomeOrderError::UncertainIpoOrder
        | OutcomeOrderError::OutcomeOfIsNotTransition
        | OutcomeOrderError::InvalidEdgePayload
        | _ => AnalysisEngineError::InvalidEvidence,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        OUTCOME_ORDER_ARTIFACT_BYTE_LIMIT, OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION,
        OUTCOME_ORDER_INFERENCE_STATUS, OutcomeOrderArtifact,
    };
    use crate::AnalysisEngineError;

    fn artifact() -> OutcomeOrderArtifact {
        OutcomeOrderArtifact {
            schema_version: OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            edge_count: 3,
            input_to_count: 1,
            process_to_count: 1,
            outcome_of_count: 1,
            refused_as_transition_count: 1,
            inference_status: OUTCOME_ORDER_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &OutcomeOrderArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidOutcomeOrderArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            OutcomeOrderArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            OutcomeOrderArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidOutcomeOrderArtifact)
        );
        assert_eq!(
            OutcomeOrderArtifact::from_json(&"x".repeat(OUTCOME_ORDER_ARTIFACT_BYTE_LIMIT + 1)),
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
                value.edge_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.input_to_count = 0;
                value.edge_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.process_to_count = 0;
                value.edge_count = 2;
                value
            },
            {
                let mut value = artifact.clone();
                value.outcome_of_count = 0;
                value.refused_as_transition_count = 0;
                value.edge_count = 2;
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
