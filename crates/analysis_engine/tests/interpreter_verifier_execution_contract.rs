//! End-to-end contract for cutoff-safe interpreter/verifier composition.

use analysis_engine::{
    AnalysisEngineError, INTERPRETER_VERIFIER_ARTIFACT_SCHEMA_VERSION,
    INTERPRETER_VERIFIER_MODEL_CONTRACT_VERSION, INTERPRETER_VERIFIER_OUTPUT_PROFILE,
    InterpreterVerifierExecution, InterpreterVerifierInput, execute_interpreter_verifier_run,
};
use interpretation_gateway::{ClaimSupport, InterpretationError, InterpretationId};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState, ApiError};
use uuid::Uuid;

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff")
}

fn available(value: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(value).expect("available time")
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
    let interpretation_id = InterpretationId::from_uuid(Uuid::from_u128(2));
    InterpreterVerifierInput::new(
        interpretation_id,
        vec![(
            Uuid::from_u128(7),
            "snapshot-interpreter-verifier".into(),
            available("2026-01-15T00:00:00Z"),
        )],
        vec![
            (
                Uuid::from_u128(101),
                interpretation_id,
                "snapshot-interpreter-verifier".into(),
                available("2026-01-15T00:00:00Z"),
                ClaimSupport::Unsupported,
                ClaimSupport::Unsupported,
            ),
            (
                Uuid::from_u128(102),
                interpretation_id,
                "snapshot-interpreter-verifier".into(),
                available("2026-01-15T00:00:00Z"),
                ClaimSupport::Unsupported,
                ClaimSupport::Unsupported,
            ),
            (
                Uuid::from_u128(103),
                interpretation_id,
                "snapshot-interpreter-verifier".into(),
                available("2026-01-15T00:00:00Z"),
                ClaimSupport::Supported,
                ClaimSupport::Supported,
            ),
        ],
    )
    .expect("input")
}

fn uncited_promotion_input() -> InterpreterVerifierInput {
    let interpretation_id = InterpretationId::from_uuid(Uuid::from_u128(2));
    InterpreterVerifierInput::new(
        interpretation_id,
        vec![(
            Uuid::from_u128(7),
            "snapshot-interpreter-verifier".into(),
            available("2026-01-15T00:00:00Z"),
        )],
        vec![
            (
                Uuid::from_u128(101),
                interpretation_id,
                "snapshot-interpreter-verifier".into(),
                available("2026-01-15T00:00:00Z"),
                ClaimSupport::Unsupported,
                ClaimSupport::Supported,
            ),
            (
                Uuid::from_u128(102),
                interpretation_id,
                "snapshot-interpreter-verifier".into(),
                available("2026-01-15T00:00:00Z"),
                ClaimSupport::Unsupported,
                ClaimSupport::Supported,
            ),
            (
                Uuid::from_u128(103),
                interpretation_id,
                "snapshot-interpreter-verifier".into(),
                available("2026-01-15T00:00:00Z"),
                ClaimSupport::Supported,
                ClaimSupport::Supported,
            ),
        ],
    )
    .expect("input")
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
    assert_eq!(summary.validation_status, "validated");
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
fn equivalent_cutoff_spelling_binds_to_the_same_instant() {
    let mut request = request();
    request.knowledge_cutoff = "2026-02-01T01:00:00+01:00".into();
    let execution = execute(&request, &cited_input()).expect("equivalent cutoff");
    assert_eq!(execution.artifact.knowledge_cutoff, "2026-02-01T00:00:00Z");
}

#[test]
fn future_duplicate_records_cannot_change_a_historical_result() {
    let request = request();
    let baseline_input = cited_input();
    let baseline = execute(&request, &baseline_input).expect("baseline");
    let interpretation_id = baseline_input.interpretation_id();
    let mut evidence_spans = baseline_input.evidence_spans().to_vec();
    evidence_spans.insert(
        0,
        (
            Uuid::from_u128(7),
            "snapshot-interpreter-verifier".into(),
            available("2026-02-02T00:00:00Z"),
        ),
    );
    let mut claims = baseline_input.claims().to_vec();
    claims.insert(
        0,
        (
            Uuid::from_u128(101),
            interpretation_id,
            "snapshot-interpreter-verifier".into(),
            available("2026-02-02T00:00:00Z"),
            ClaimSupport::Unsupported,
            ClaimSupport::Supported,
        ),
    );
    let replay_input = InterpreterVerifierInput::new(interpretation_id, evidence_spans, claims)
        .expect("replay input");
    let replay = execute(&request, &replay_input).expect("replay");
    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result, baseline.terminal_result);
}

#[test]
fn unrelated_snapshot_and_interpretation_claims_fail_closed() {
    let interpretation_id = InterpretationId::from_uuid(Uuid::from_u128(2));
    let other_interpretation_id = InterpretationId::from_uuid(Uuid::from_u128(3));
    let wrong_snapshot = InterpreterVerifierInput::new(
        interpretation_id,
        vec![(
            Uuid::from_u128(7),
            "other-snapshot".into(),
            available("2026-01-15T00:00:00Z"),
        )],
        vec![(
            Uuid::from_u128(101),
            interpretation_id,
            "snapshot-interpreter-verifier".into(),
            available("2026-01-15T00:00:00Z"),
            ClaimSupport::Unsupported,
            ClaimSupport::Unsupported,
        )],
    )
    .expect("wrong snapshot input");
    assert_eq!(
        execute(&request(), &wrong_snapshot),
        Err(AnalysisEngineError::SnapshotMismatch)
    );

    let wrong_interpretation = InterpreterVerifierInput::new(
        interpretation_id,
        vec![(
            Uuid::from_u128(7),
            "snapshot-interpreter-verifier".into(),
            available("2026-01-15T00:00:00Z"),
        )],
        vec![(
            Uuid::from_u128(101),
            other_interpretation_id,
            "snapshot-interpreter-verifier".into(),
            available("2026-01-15T00:00:00Z"),
            ClaimSupport::Unsupported,
            ClaimSupport::Unsupported,
        )],
    )
    .expect("wrong interpretation input");
    assert_eq!(
        execute(&request(), &wrong_interpretation),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn missing_spans_and_invalid_support_payloads_fail_closed() {
    let request = request();
    let interpretation_id = InterpretationId::from_uuid(Uuid::from_u128(2));
    let missing_spans = InterpreterVerifierInput::new(
        interpretation_id,
        Vec::new(),
        vec![(
            Uuid::from_u128(101),
            interpretation_id,
            "snapshot-interpreter-verifier".into(),
            available("2026-01-15T00:00:00Z"),
            ClaimSupport::Unsupported,
            ClaimSupport::Unsupported,
        )],
    )
    .expect("missing spans input");
    assert_eq!(
        execute(&request, &missing_spans),
        Err(AnalysisEngineError::Interpretation(
            InterpretationError::MissingEvidenceSpan
        ))
    );
    let invalid_support = InterpreterVerifierInput::new(
        interpretation_id,
        vec![(
            Uuid::from_u128(7),
            "snapshot-interpreter-verifier".into(),
            available("2026-01-15T00:00:00Z"),
        )],
        vec![(
            Uuid::from_u128(101),
            interpretation_id,
            "snapshot-interpreter-verifier".into(),
            available("2026-01-15T00:00:00Z"),
            ClaimSupport::Supported,
            ClaimSupport::Supported,
        )],
    )
    .expect("invalid support input");
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

#[test]
fn invalid_completed_at_fails_terminal_result_construction() {
    let request = request();
    assert_eq!(
        execute_interpreter_verifier_run(
            &request,
            &accepted(&request),
            "snapshot-interpreter-verifier",
            cutoff(),
            &cited_input(),
            "not-a-timestamp",
        ),
        Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
    );
}
