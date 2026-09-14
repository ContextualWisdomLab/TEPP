//! Digest-bound interpreter/verifier output as an analysis-run profile.

use interpretation_gateway::{
    ClaimSupport, EvidenceBoundInterpretation, InterpretationError, InterpretationId,
    refuse_interpretation_as_estimator_result, refuse_interpretation_as_observed_fact,
    unsupported_claim_rate,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};
use uuid::Uuid;

use crate::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, format_digest, require_receipt_identity,
    valid_identifier,
};

/// Versioned schema for a completed interpreter/verifier artifact.
pub const INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION: &str = "tepp.interpreter_verifier.v1";
/// Model contract required by the interpreter/verifier execution path.
pub const INTERPRETER_VERIFIER_MODEL_CONTRACT_VERSION: &str = "interpreter_verifier_v1";
/// Analysis-run output profile required for an interpreter/verifier artifact.
pub const INTERPRETER_VERIFIER_OUTPUT_PROFILE: &str = "interpreter_verifier_v1";
/// Maximum canonical artifact JSON size accepted from an untrusted payload.
pub const INTERPRETER_VERIFIER_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const INTERPRETER_VERIFIER_INFERENCE_STATUS: &str =
    "hypothetical_interpretation_not_scientific_authority";
const HYPOTHETICAL_STATUS: &str = "hypothetical";
const VALIDATED_STATUS: &str = "validated";

type EvidenceSpanRecord = (Uuid, String, AvailableTime);
type ClaimRecord = (
    Uuid,
    InterpretationId,
    String,
    AvailableTime,
    ClaimSupport,
    ClaimSupport,
);

/// Offered evidence-bounded interpretation plus cutoff-bound known-truth claim labels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpreterVerifierInput {
    interpretation_id: InterpretationId,
    evidence_spans: Vec<EvidenceSpanRecord>,
    claims: Vec<ClaimRecord>,
}

impl InterpreterVerifierInput {
    /// Bundle one interpretation identity, cited spans, and known-truth labels.
    ///
    /// Each evidence tuple is `(span_id, snapshot_id, available_time)`. Each
    /// claim tuple is `(claim_id, interpretation_id, snapshot_id,
    /// available_time, truth, decided)`. The explicit per-record provenance is
    /// required so historical execution can exclude future evidence before
    /// identity admission and can refuse unrelated snapshot or interpretation
    /// labels rather than silently attributing them to this run.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::LimitExceeded`] when either offered
    /// population exceeds [`MAX_EVIDENCE_UNITS`], or
    /// [`AnalysisEngineError::InvalidEvidence`] for an invalid snapshot
    /// identifier.
    pub fn new(
        interpretation_id: InterpretationId,
        evidence_spans: Vec<EvidenceSpanRecord>,
        claims: Vec<ClaimRecord>,
    ) -> Result<Self, AnalysisEngineError> {
        if evidence_spans.len() > MAX_EVIDENCE_UNITS || claims.len() > MAX_EVIDENCE_UNITS {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        if evidence_spans
            .iter()
            .any(|(_, snapshot_id, _)| !valid_identifier(snapshot_id))
            || claims
                .iter()
                .any(|(_, _, snapshot_id, _, _, _)| !valid_identifier(snapshot_id))
        {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        Ok(Self {
            interpretation_id,
            evidence_spans,
            claims,
        })
    }

    /// Return the interpretation identity.
    #[must_use]
    pub const fn interpretation_id(&self) -> InterpretationId {
        self.interpretation_id
    }

    /// Borrow offered evidence span records.
    #[must_use]
    pub fn evidence_spans(&self) -> &[EvidenceSpanRecord] {
        &self.evidence_spans
    }

    /// Borrow offered claim records.
    #[must_use]
    pub fn claims(&self) -> &[ClaimRecord] {
        &self.claims
    }
}

/// Completed, bounded interpreter/verifier result for analysis-run clients.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterpreterVerifierArtifact {
    /// Exact versioned schema identity.
    pub schema_version: String,
    /// Opaque accepted-run identity.
    pub run_id: String,
    /// Immutable source snapshot identity.
    pub snapshot_id: String,
    /// Historical evidence cutoff used by the run.
    pub knowledge_cutoff: String,
    /// Opaque interpretation identity.
    pub interpretation_id: String,
    /// Number of cited evidence spans admitted at the cutoff.
    pub cited_span_count: u64,
    /// False-support rate over unsupported known truth.
    pub unsupported_claim_rate: f64,
    /// Whether estimator-result promotion was refused.
    pub estimator_result_refused: bool,
    /// Whether observed-fact promotion was refused.
    pub observed_fact_refused: bool,
    /// Interpretation remains hypothetical.
    pub interpretation_status: String,
    /// Fixed claim boundary for consumer copy.
    pub inference_status: String,
}

impl InterpreterVerifierArtifact {
    /// Parse and fully validate a bounded artifact JSON payload.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisEngineError::InvalidInterpreterVerifierArtifact`] when
    /// the schema, identifiers, counts, rate, or claim boundary fail.
    pub fn from_json(payload: &str) -> Result<Self, AnalysisEngineError> {
        if payload.len() > INTERPRETER_VERIFIER_ARTIFACT_BYTE_LIMIT {
            return Err(AnalysisEngineError::LimitExceeded);
        }
        let artifact: Self = serde_json::from_str(payload)
            .map_err(|_| AnalysisEngineError::InvalidInterpreterVerifierArtifact)?;
        artifact.validate()?;
        Ok(artifact)
    }

    /// Serialize canonical validated artifact JSON.
    ///
    /// # Errors
    ///
    /// Returns a typed validation or serialization failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)
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
        let canonical_cutoff = KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff)
            .map(|cutoff| cutoff.to_rfc3339())
            .map_err(|_| AnalysisEngineError::InvalidInterpreterVerifierArtifact)?;
        if self.schema_version != INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || canonical_cutoff != self.knowledge_cutoff
            || Uuid::parse_str(&self.interpretation_id).is_err()
            || self.cited_span_count == 0
            || !self.unsupported_claim_rate.is_finite()
            || self.unsupported_claim_rate < 0.0
            || self.unsupported_claim_rate > 1.0
            || !self.estimator_result_refused
            || !self.observed_fact_refused
            || self.interpretation_status != HYPOTHETICAL_STATUS
            || self.inference_status != INTERPRETER_VERIFIER_INFERENCE_STATUS
        {
            return Err(AnalysisEngineError::InvalidInterpreterVerifierArtifact);
        }
        Ok(())
    }
}

/// One completed interpreter/verifier artifact and its terminal result.
#[derive(Clone, Debug, PartialEq)]
pub struct InterpreterVerifierExecution {
    /// Digest-bound completed interpretation artifact.
    pub artifact: InterpreterVerifierArtifact,
    /// Terminal result carrying the artifact identity, digest, and schema.
    pub terminal_result: AnalysisRunTerminalResult,
}

fn cutoff_admitted_spans(
    input: &InterpreterVerifierInput,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<Vec<Uuid>, AnalysisEngineError> {
    let mut seen = BTreeSet::new();
    let mut admitted = Vec::new();
    for (span_id, source_snapshot_id, available_time) in input.evidence_spans() {
        if source_snapshot_id != snapshot_id {
            return Err(AnalysisEngineError::SnapshotMismatch);
        }
        if available_time.instant() > knowledge_cutoff.instant() {
            continue;
        }
        if !seen.insert(*span_id) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        admitted.push(*span_id);
    }
    Ok(admitted)
}

fn cutoff_admitted_claim_labels(
    input: &InterpreterVerifierInput,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
) -> Result<(Vec<ClaimSupport>, Vec<ClaimSupport>), AnalysisEngineError> {
    let mut seen = BTreeSet::new();
    let mut truth = Vec::new();
    let mut decided = Vec::new();
    for (
        claim_id,
        interpretation_id,
        source_snapshot_id,
        available_time,
        truth_label,
        decided_label,
    ) in input.claims()
    {
        if source_snapshot_id != snapshot_id {
            return Err(AnalysisEngineError::SnapshotMismatch);
        }
        if *interpretation_id != input.interpretation_id() {
            return Err(AnalysisEngineError::InvalidEvidence);
        }
        if available_time.instant() > knowledge_cutoff.instant() {
            continue;
        }
        if !seen.insert(*claim_id) {
            return Err(AnalysisEngineError::DuplicateEvidence);
        }
        truth.push(*truth_label);
        decided.push(*decided_label);
    }
    Ok((truth, decided))
}

/// Execute cutoff-safe interpreter/verifier composition as one analysis-run profile.
///
/// The executor invokes [`EvidenceBoundInterpretation::propose`],
/// [`refuse_interpretation_as_estimator_result`],
/// [`refuse_interpretation_as_observed_fact`], and [`unsupported_claim_rate`].
/// Evidence and claim records are admitted by immutable snapshot and
/// `AvailableTime <= KnowledgeCutoff` before duplicate or interpretation-link
/// checks can affect the historical result. It does not call a live LLM
/// provider, weaken numerical-authority refusal, or promote an interpretation
/// to scientific truth.
///
/// # Errors
///
/// Returns a request/receipt/snapshot/cutoff/profile error, interpretation
/// refusal, support-rate failure, or invalid artifact error.
pub fn execute_interpreter_verifier_run(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    input: &InterpreterVerifierInput,
    completed_at: impl Into<String>,
) -> Result<InterpreterVerifierExecution, AnalysisEngineError> {
    request.to_json()?;
    accepted.to_json()?;
    require_receipt_identity(request, accepted)?;
    if request.snapshot_id != snapshot_id {
        return Err(AnalysisEngineError::SnapshotMismatch);
    }
    let request_cutoff = KnowledgeCutoff::parse_rfc3339(&request.knowledge_cutoff)
        .map_err(|_| AnalysisEngineError::InvalidEvidence)?;
    if request_cutoff.instant() != knowledge_cutoff.instant()
        || request.model_contract_version != INTERPRETER_VERIFIER_MODEL_CONTRACT_VERSION
        || request.output_profile != INTERPRETER_VERIFIER_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let evidence_span_ids = cutoff_admitted_spans(input, snapshot_id, knowledge_cutoff)?;
    let (truth, decided) = cutoff_admitted_claim_labels(input, snapshot_id, knowledge_cutoff)?;
    let interpretation =
        EvidenceBoundInterpretation::propose(input.interpretation_id(), &evidence_span_ids)?;
    let estimator_result_refused =
        refuse_interpretation_as_estimator_result(interpretation.interpretation_id())
            == Err(InterpretationError::InterpretationIsNotEstimatorResult);
    let observed_fact_refused =
        refuse_interpretation_as_observed_fact(interpretation.interpretation_id())
            == Err(InterpretationError::InterpretationIsNotObservedFact);
    let rate = unsupported_claim_rate(&truth, &decided)?;
    let cited_span_count = u64::try_from(interpretation.evidence_span_ids().len())
        .map_err(|_| AnalysisEngineError::LimitExceeded)?;
    let artifact = InterpreterVerifierArtifact {
        schema_version: INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: accepted.run_id.clone(),
        snapshot_id: snapshot_id.to_owned(),
        knowledge_cutoff: knowledge_cutoff.to_rfc3339(),
        interpretation_id: interpretation.interpretation_id().as_uuid().to_string(),
        cited_span_count,
        unsupported_claim_rate: rate,
        estimator_result_refused,
        observed_fact_refused,
        interpretation_status: HYPOTHETICAL_STATUS.into(),
        inference_status: INTERPRETER_VERIFIER_INFERENCE_STATUS.into(),
    };
    let digest = artifact.sha256()?;
    let summary = AnalysisResultSummary::new(
        "interpreter_verifier",
        cited_span_count,
        2,
        VALIDATED_STATUS,
    )?;
    let terminal_result = AnalysisRunTerminalResult::succeeded(
        request,
        accepted,
        format!("interpreter_verifier_artifact_{}", &digest[..16]),
        digest,
        INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION,
        completed_at,
        summary,
    )?;
    Ok(InterpreterVerifierExecution {
        artifact,
        terminal_result,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        HYPOTHETICAL_STATUS, INTERPRETER_VERIFIER_ARTIFACT_BYTE_LIMIT,
        INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION, INTERPRETER_VERIFIER_INFERENCE_STATUS,
        InterpreterVerifierArtifact, InterpreterVerifierInput,
    };
    use crate::{AnalysisEngineError, MAX_EVIDENCE_UNITS};
    use interpretation_gateway::{ClaimSupport, InterpretationId};
    use temporal_core::AvailableTime;
    use uuid::Uuid;

    fn available_time() -> AvailableTime {
        AvailableTime::parse_rfc3339("2026-01-01T00:00:00Z").expect("available time")
    }

    fn artifact() -> InterpreterVerifierArtifact {
        InterpreterVerifierArtifact {
            schema_version: INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION.into(),
            run_id: "run-1".into(),
            snapshot_id: "snapshot-1".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            interpretation_id: "00000000-0000-0000-0000-000000000002".into(),
            cited_span_count: 1,
            unsupported_claim_rate: 0.0,
            estimator_result_refused: true,
            observed_fact_refused: true,
            interpretation_status: HYPOTHETICAL_STATUS.into(),
            inference_status: INTERPRETER_VERIFIER_INFERENCE_STATUS.into(),
        }
    }

    fn assert_invalid(artifact: &InterpreterVerifierArtifact) {
        assert_eq!(
            artifact.to_json(),
            Err(AnalysisEngineError::InvalidInterpreterVerifierArtifact)
        );
    }

    #[test]
    fn artifact_round_trip_and_size_bounds_fail_closed() {
        let artifact = artifact();
        let payload = artifact.to_json().expect("json");
        assert_eq!(
            InterpreterVerifierArtifact::from_json(&payload),
            Ok(artifact.clone())
        );
        assert_eq!(artifact.sha256().expect("digest").len(), 64);
        assert_eq!(
            InterpreterVerifierArtifact::from_json("{}"),
            Err(AnalysisEngineError::InvalidInterpreterVerifierArtifact)
        );
        assert_eq!(
            InterpreterVerifierArtifact::from_json(
                &"x".repeat(INTERPRETER_VERIFIER_ARTIFACT_BYTE_LIMIT + 1)
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
                value.knowledge_cutoff = "2026-08-01T01:00:00+01:00".into();
                value
            },
            {
                let mut value = artifact.clone();
                value.interpretation_id.clear();
                value
            },
            {
                let mut value = artifact.clone();
                value.cited_span_count = 0;
                value
            },
            {
                let mut value = artifact.clone();
                value.unsupported_claim_rate = -0.1;
                value
            },
            {
                let mut value = artifact.clone();
                value.unsupported_claim_rate = f64::NAN;
                value
            },
            {
                let mut value = artifact.clone();
                value.unsupported_claim_rate = 1.5;
                value
            },
            {
                let mut value = artifact.clone();
                value.estimator_result_refused = false;
                value
            },
            {
                let mut value = artifact.clone();
                value.observed_fact_refused = false;
                value
            },
            {
                let mut value = artifact.clone();
                value.interpretation_status = "scientific".into();
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
    fn input_accessors_preserve_offered_provenance_and_labels() {
        let id = InterpretationId::from_uuid(Uuid::from_u128(2));
        let span = Uuid::from_u128(7);
        let claim = Uuid::from_u128(8);
        let available_time = available_time();
        let input = InterpreterVerifierInput::new(
            id,
            vec![(span, "snapshot-1".into(), available_time)],
            vec![(
                claim,
                id,
                "snapshot-1".into(),
                available_time,
                ClaimSupport::Unsupported,
                ClaimSupport::Supported,
            )],
        )
        .expect("input");
        assert_eq!(input.interpretation_id(), id);
        assert_eq!(input.evidence_spans()[0].0, span);
        assert_eq!(input.claims()[0].0, claim);
    }

    #[test]
    fn input_population_is_bounded_before_execution() {
        let id = InterpretationId::from_uuid(Uuid::from_u128(2));
        let span = (Uuid::from_u128(7), "snapshot-1".into(), available_time());
        assert_eq!(
            InterpreterVerifierInput::new(id, vec![span; MAX_EVIDENCE_UNITS + 1], Vec::new()),
            Err(AnalysisEngineError::LimitExceeded)
        );
    }
}
