//! GAP-003A: scientific acceptance is a terminal result, never a receipt.

use tepp_api::{
    ANALYSIS_RUN_CONTRACT_VERSION, AnalysisResultSummary, AnalysisRunAccepted, AnalysisRunRequest,
    AnalysisRunStatus, AnalysisRunTerminalResult, ApiError, SCIENTIFIC_ACCEPTANCE_BACKEND,
    SCIENTIFIC_ACCEPTANCE_MODEL, SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE,
    SCIENTIFIC_ACCEPTANCE_PRECISION, SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION,
    ScientificAcceptanceArtifact, ScientificAcceptanceReport,
    receipt_json_carries_scientific_metrics,
};

fn report() -> ScientificAcceptanceReport {
    ScientificAcceptanceReport {
        study_label: "gap-003a-api".into(),
        rmse: 0.02,
        rmse_standard_error: 0.01,
        mean_bias: 0.0,
        bias_standard_error: 0.02,
        interval_coverage: 0.95,
        coverage_wilson_lower: 0.90,
        coverage_wilson_upper: 0.98,
        temporal_order_accuracy: 1.0,
    }
}

fn artifact() -> ScientificAcceptanceArtifact {
    ScientificAcceptanceArtifact {
        schema_version: SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION.into(),
        run_id: "tepp-validation-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        binding_sha256: "a".repeat(64),
        snapshot_id: "snapshot-1".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model: SCIENTIFIC_ACCEPTANCE_MODEL.into(),
        seed: 7,
        backend: SCIENTIFIC_ACCEPTANCE_BACKEND.into(),
        precision: SCIENTIFIC_ACCEPTANCE_PRECISION.into(),
        output_profile: SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE.into(),
        eligible_evidence_count: 4,
        se_gate_accepted: true,
        se_gate_k: 3.0,
        report: report(),
    }
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
        idempotency_key: "idem-1".into(),
        tenant_workspace_id: "tenant-ws-1".into(),
        snapshot_id: "snapshot-1".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: SCIENTIFIC_ACCEPTANCE_MODEL.into(),
        output_profile: SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE.into(),
    }
}

fn accepted() -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "tepp-validation-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "accepted",
        "idem-1",
    )
    .expect("accepted")
}

fn summary() -> AnalysisResultSummary {
    AnalysisResultSummary::new("scientific_acceptance", 4, 8, "validated").expect("summary")
}

#[test]
fn request_and_accepted_receipts_stay_metric_free() {
    let request = request();
    let json = request.to_json().expect("request json");
    assert!(!receipt_json_carries_scientific_metrics(&json));
    assert!(!json.contains("rmse"));
    assert_eq!(
        AnalysisRunRequest::from_json(&json).expect("decode"),
        request
    );
    let with_rmse = json.replacen('{', r#"{"rmse":0.04,"#, 1);
    assert_eq!(
        AnalysisRunRequest::from_json(&with_rmse),
        Err(ApiError::InvalidWirePayload)
    );
    let with_artifact = json.replacen('{', r#"{"scientific_acceptance":{},"#, 1);
    assert_eq!(
        AnalysisRunRequest::from_json(&with_artifact),
        Err(ApiError::InvalidWirePayload)
    );

    let accepted = accepted();
    let accepted_json = accepted.to_json().expect("accepted json");
    assert!(!receipt_json_carries_scientific_metrics(&accepted_json));
    let accepted_metrics = accepted_json.replacen('{', r#"{"report":{},"#, 1);
    assert_eq!(
        AnalysisRunAccepted::from_json(&accepted_metrics),
        Err(ApiError::InvalidWirePayload)
    );
    assert_eq!(
        AnalysisRunStatus::accepted(&accepted)
            .expect("status")
            .terminal_result,
        None
    );
}

#[test]
fn only_succeeded_terminal_may_carry_scientific_acceptance() {
    let terminal = AnalysisRunTerminalResult::succeeded_scientific_acceptance(
        &request(),
        &accepted(),
        "artifact-1",
        "2026-08-02T03:04:05Z",
        summary(),
        artifact(),
    )
    .expect("terminal");
    assert_eq!(
        terminal.result_schema_version.as_deref(),
        Some(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION)
    );
    let json = terminal.to_json().expect("json");
    assert!(json.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA_VERSION));
    assert_eq!(
        AnalysisRunTerminalResult::from_json(&json).expect("round-trip"),
        terminal
    );
    let status =
        AnalysisRunStatus::terminal(&request(), &accepted(), terminal.clone()).expect("status");
    assert!(status.terminal_result.is_some());

    let mut mismatched = artifact();
    mismatched.run_id = "tepp-validation-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into();
    mismatched.binding_sha256 = "b".repeat(64);
    assert_eq!(
        AnalysisRunTerminalResult::succeeded_scientific_acceptance(
            &request(),
            &accepted(),
            "artifact-1",
            "2026-08-02T03:04:05Z",
            summary(),
            mismatched,
        ),
        Err(ApiError::InvalidWirePayload)
    );

    let mut wrong_model = request();
    wrong_model.model_contract_version = "other-model".into();
    assert_eq!(
        AnalysisRunTerminalResult::succeeded_scientific_acceptance(
            &wrong_model,
            &accepted(),
            "artifact-1",
            "2026-08-02T03:04:05Z",
            summary(),
            artifact(),
        ),
        Err(ApiError::InvalidWirePayload)
    );

    let mut ordinary = request();
    ordinary.output_profile = "validation-report".into();
    assert_eq!(
        AnalysisRunTerminalResult::succeeded_scientific_acceptance(
            &ordinary,
            &accepted(),
            "artifact-1",
            "2026-08-02T03:04:05Z",
            summary(),
            artifact(),
        ),
        Err(ApiError::InvalidWirePayload)
    );

    let failed = AnalysisRunTerminalResult::failed(
        &request(),
        &accepted(),
        "2026-08-02T03:04:05Z",
        "estimation_failed",
    )
    .expect("failed");
    assert!(failed.scientific_acceptance.is_none());
    let mut failed_with_artifact = failed.clone();
    failed_with_artifact.scientific_acceptance = Some(artifact());
    assert_eq!(
        failed_with_artifact.to_json(),
        Err(ApiError::InvalidWirePayload)
    );

    let mut tampered = terminal;
    tampered.result_sha256 = Some("b".repeat(64));
    assert_eq!(tampered.to_json(), Err(ApiError::InvalidWirePayload));
}

#[test]
fn scientific_acceptance_profile_without_artifact_fails_closed() {
    let mut profiled = request();
    profiled.output_profile = SCIENTIFIC_ACCEPTANCE_OUTPUT_PROFILE.into();
    let ordinary = AnalysisRunTerminalResult::succeeded(
        &profiled,
        &accepted(),
        "artifact-1",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "tepp-result-v1",
        "2026-08-02T03:04:05Z",
        summary(),
    );
    assert_eq!(ordinary, Err(ApiError::InvalidWirePayload));

    let mut other_profile = request();
    other_profile.output_profile = "validation-report".into();
    let mut stuffed = AnalysisRunTerminalResult::succeeded(
        &other_profile,
        &accepted(),
        "artifact-1",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "tepp-result-v1",
        "2026-08-02T03:04:05Z",
        summary(),
    )
    .expect("ordinary");
    stuffed.scientific_acceptance = Some(artifact());
    assert_eq!(stuffed.to_json(), Err(ApiError::InvalidWirePayload));
}
