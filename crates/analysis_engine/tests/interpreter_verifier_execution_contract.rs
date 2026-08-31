//! End-to-end contract for cutoff-safe interpreter/verifier composition.

use analysis_engine::{
    AnalysisEngineError, INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION,
    INTERPRETER_VERIFIER_MODEL_CONTRACT_VERSION, INTERPRETER_VERIFIER_OUTPUT_PROFILE,
    InterpreterVerifierExecution, InterpreterVerifierInput, execute_interpreter_verifier_run,
};
use interpretation_gateway::{ClaimSupport, InterpretationError, InterpretationId};
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};
use uuid::Uuid;

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "interpreter-verifier-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-interpreter-verifier".into(),
        knowledge_cutoff: "2026-02-01T00:00:00Z".into(),
        model_contract_version: INTERPRETER_VERIFIER_MODEL_CONTRACT_VERSION.into(),
        output_profile: INTERPRETER_VERIFIER_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-interpreter-verifier",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn cited_input() -> InterpreterVerifierInput {
    InterpreterVerifierInput::new(
        InterpretationId::from_uuid(Uuid::from_u128(2)),
        vec![Uuid::from_u128(7)],
        vec![
            ClaimSupport::Unsupported,
            ClaimSupport::Unsupported,
            ClaimSupport::Supported,
        ],
        vec![
            ClaimSupport::Unsupported,
            ClaimSupport::Unsupported,
            ClaimSupport::Supported,
        ],
    )
}

fn uncited_promotion_input() -> InterpreterVerifierInput {
    InterpreterVerifierInput::new(
        InterpretationId::from_uuid(Uuid::from_u128(2)),
        vec![Uuid::from_u128(7)],
        vec![
            ClaimSupport::Unsupported,
            ClaimSupport::Unsupported,
            ClaimSupport::Supported,
        ],
        vec![
            ClaimSupport::Supported,
            ClaimSupport::Supported,
            ClaimSupport::Supported,
        ],
    )
}

fn execute(
    request: &AnalysisRunRequest,
    input: &InterpreterVerifierInput,
) -> Result<InterpreterVerifierExecution, AnalysisEngineError> {
    execute_interpreter_verifier_run(
        request,
        &accepted(request),
        "snapshot-interpreter-verifier",
        cutoff(),
        input,
        "2026-02-02T00:00:00Z",
    )
}

#[test]
fn cited_interpretation_stays_hypothetical_and_records_zero_unsupported_rate() {
    let request = request();
    let execution = execute(&request, &cited_input()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.cited_span_count, 1);
    assert!((execution.artifact.unsupported_claim_rate - 0.0).abs() < f64::EPSILON);
    assert!(execution.artifact.estimator_result_refused);
    assert!(execution.artifact.observed_fact_refused);
    assert_eq!(execution.artifact.interpretation_status, "hypothetical");
    assert_eq!(
        execution.artifact.inference_status,
        "hypothetical_interpretation_not_scientific_authority"
    );
    assert_eq!(
        execution.terminal_result.run_state,
        AnalysisRunTerminalState::Succeeded
    );
    assert_eq!(
        execution.terminal_result.result_sha256.as_deref(),
        Some(execution.artifact.sha256().expect("digest").as_str())
    );
    assert_eq!(
        execution.terminal_result.result_schema_version.as_deref(),
        Some(INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION)
    );
    let summary = execution.terminal_result.summary.as_ref().expect("summary");
    assert_eq!(summary.analysis_family, "interpreter_verifier");
    assert_eq!(
        summary.validation_status,
        "hypothetical_interpretation_not_scientific_authority"
    );
}

#[test]
fn uncited_promotion_records_unit_rate_and_cannot_become_scientific_authority() {
    let execution = execute(&request(), &uncited_promotion_input()).expect("execution");
    assert!((execution.artifact.unsupported_claim_rate - 1.0).abs() < f64::EPSILON);
    assert!(execution.artifact.estimator_result_refused);
    assert!(execution.artifact.observed_fact_refused);
    assert_eq!(execution.artifact.interpretation_status, "hypothetical");
    assert_eq!(
        execution.artifact.inference_status,
        "hypothetical_interpretation_not_scientific_authority"
    );
}

#[test]
fn missing_spans_and_invalid_support_payloads_fail_closed() {
    let request = request();
    let missing_spans = InterpreterVerifierInput::new(
        InterpretationId::from_uuid(Uuid::from_u128(2)),
        Vec::new(),
        vec![ClaimSupport::Unsupported],
        vec![ClaimSupport::Unsupported],
    );
    assert_eq!(
        execute(&request, &missing_spans),
        Err(AnalysisEngineError::Interpretation(
            InterpretationError::MissingEvidenceSpan
        ))
    );
    let invalid_support = InterpreterVerifierInput::new(
        InterpretationId::from_uuid(Uuid::from_u128(2)),
        vec![Uuid::from_u128(7)],
        vec![ClaimSupport::Supported],
        vec![ClaimSupport::Supported],
    );
    assert_eq!(
        execute(&request, &invalid_support),
        Err(AnalysisEngineError::Interpretation(
            InterpretationError::InvalidSupportPayload
        ))
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    assert_eq!(
        execute_interpreter_verifier_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &cited_input(),
            "2026-02-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    for invalid_request in [
        {
            let mut value = request.clone();
            value.knowledge_cutoff = "2026-08-02T00:00:00Z".into();
            value
        },
        {
            let mut value = request.clone();
            value.model_contract_version = "other-model".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "trsl_topic_lineage_v1".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request, &cited_input()),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
