//! Digest-bound interpreter/verifier output as an analysis-run profile.

use interpretation_gateway::{
    ClaimSupport, EvidenceBoundInterpretation, InterpretationError, InterpretationId,
    refuse_interpretation_as_estimator_result, refuse_interpretation_as_observed_fact,
    unsupported_claim_rate,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use temporal_core::KnowledgeCutoff;
use tepp_api::{
    AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalResult,
};
use uuid::Uuid;

use crate::{AnalysisEngineError, format_digest, require_receipt_identity, valid_identifier};

/// Versioned schema for a completed interpreter/verifier artifact.
pub const INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION: &str = "tepp.interpreter_verifier.v1";
/// Model contract required by the interpreter/verifier execution path.
pub const INTERPRETER_VERIFIER_MODEL_CONTRACT_VERSION: &str = "interpreter_verifier_v1";
/// Analysis-run output profile required for an interpreter/verifier artifact.
pub const INTERPRETER_VERIFIER_OUTPUT_PROFILE: &str = "interpreter_verifier_v1";
/// Maximum canonical artifact JSON size.
pub const INTERPRETER_VERIFIER_ARTIFACT_BYTE_LIMIT: usize = 256 * 1024;
const INTERPRETER_VERIFIER_INFERENCE_STATUS: &str =
    "hypothetical_interpretation_not_scientific_authority";
const HYPOTHETICAL_STATUS: &str = "hypothetical";

/// Offered evidence-bounded interpretation plus known-truth claim labels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterpreterVerifierInput {
    interpretation_id: InterpretationId,
    evidence_span_ids: Vec<Uuid>,
    truth: Vec<ClaimSupport>,
    decided: Vec<ClaimSupport>,
}

impl InterpreterVerifierInput {
    /// Bundle one interpretation identity, cited spans, and known-truth labels.
    #[must_use]
    pub fn new(
        interpretation_id: InterpretationId,
        evidence_span_ids: Vec<Uuid>,
        truth: Vec<ClaimSupport>,
        decided: Vec<ClaimSupport>,
    ) -> Self {
        Self {
            interpretation_id,
            evidence_span_ids,
            truth,
            decided,
        }
    }

    /// Return the interpretation identity.
    #[must_use]
    pub const fn interpretation_id(&self) -> InterpretationId {
        self.interpretation_id
    }

    /// Borrow the cited evidence span identities.
    #[must_use]
    pub fn evidence_span_ids(&self) -> &[Uuid] {
        &self.evidence_span_ids
    }

    /// Borrow the known-truth support labels.
    #[must_use]
    pub fn truth(&self) -> &[ClaimSupport] {
        &self.truth
    }

    /// Borrow the decided support labels.
    #[must_use]
    pub fn decided(&self) -> &[ClaimSupport] {
        &self.decided
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
    /// Number of cited evidence spans.
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
    /// Returns a typed validation, serialization, or size failure.
    pub fn to_json(&self) -> Result<String, AnalysisEngineError> {
        self.validate()?;
        let payload =
            serde_json::to_string(self).map_err(|_| AnalysisEngineError::SerializationFailure)?;
        if payload.len() > INTERPRETER_VERIFIER_ARTIFACT_BYTE_LIMIT {
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
        if self.schema_version != INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION
            || !valid_identifier(&self.run_id)
            || !valid_identifier(&self.snapshot_id)
            || KnowledgeCutoff::parse_rfc3339(&self.knowledge_cutoff).is_err()
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

/// Execute cutoff-safe interpreter/verifier composition as one analysis-run profile.
///
/// The executor invokes [`EvidenceBoundInterpretation::propose`],
/// [`refuse_interpretation_as_estimator_result`],
/// [`refuse_interpretation_as_observed_fact`], and [`unsupported_claim_rate`].
/// It does not call a live LLM provider, does not weaken numerical-authority
/// refusal, and cannot promote an interpretation to scientific truth.
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
    if request.knowledge_cutoff != knowledge_cutoff.to_rfc3339()
        || request.model_contract_version != INTERPRETER_VERIFIER_MODEL_CONTRACT_VERSION
        || request.output_profile != INTERPRETER_VERIFIER_OUTPUT_PROFILE
    {
        return Err(AnalysisEngineError::InvalidEvidence);
    }

    let interpretation =
        EvidenceBoundInterpretation::propose(input.interpretation_id(), input.evidence_span_ids())?;
    let estimator_result_refused =
        refuse_interpretation_as_estimator_result(interpretation.interpretation_id())
            == Err(InterpretationError::InterpretationIsNotEstimatorResult);
    let observed_fact_refused =
        refuse_interpretation_as_observed_fact(interpretation.interpretation_id())
            == Err(InterpretationError::InterpretationIsNotObservedFact);
    let rate = unsupported_claim_rate(input.truth(), input.decided())?;
    let cited_span_count = interpretation.evidence_span_ids().len() as u64;
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
        INTERPRETER_VERIFIER_INFERENCE_STATUS,
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
    use crate::AnalysisEngineError;
    use interpretation_gateway::{ClaimSupport, InterpretationId};
    use uuid::Uuid;

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
    fn input_accessors_preserve_offered_spans_and_labels() {
        let id = InterpretationId::from_uuid(Uuid::from_u128(2));
        let span = Uuid::from_u128(7);
        let input = InterpreterVerifierInput::new(
            id,
            vec![span],
            vec![ClaimSupport::Unsupported],
            vec![ClaimSupport::Supported],
        );
        assert_eq!(input.interpretation_id(), id);
        assert_eq!(input.evidence_span_ids(), &[span]);
        assert_eq!(input.truth(), &[ClaimSupport::Unsupported]);
        assert_eq!(input.decided(), &[ClaimSupport::Supported]);
    }
}
