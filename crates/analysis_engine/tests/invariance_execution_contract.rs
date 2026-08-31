//! End-to-end contract for cutoff-safe two-group OLS invariance.

use analysis_engine::{
    AnalysisEngineError, InvarianceObservation, MAX_EVIDENCE_UNITS,
    TWO_GROUP_OLS_INVARIANCE_ARTIFACT_SCHEMA_VERSION,
    TWO_GROUP_OLS_INVARIANCE_MODEL_CONTRACT_VERSION, TWO_GROUP_OLS_INVARIANCE_OUTPUT_PROFILE,
    execute_two_group_ols_invariance_run,
};
use psychometric_core::{IndicatorKind, PsychometricError};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn series(factors: &[f64], intercept: f64, loading: f64) -> Vec<InvarianceObservation> {
    factors
        .iter()
        .map(|score| {
            InvarianceObservation::new(
                *score,
                intercept + loading * score,
                available("2026-07-01T00:00:00Z"),
            )
            .expect("row")
        })
        .collect()
}

fn strict_reference() -> Vec<InvarianceObservation> {
    series(&[-1.0, 0.0, 1.0], 0.5, 1.2)
}

fn strict_comparison() -> Vec<InvarianceObservation> {
    series(&[1.0, 2.0, 3.0], 0.5, 1.2)
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "invariance-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-invariance".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: TWO_GROUP_OLS_INVARIANCE_MODEL_CONTRACT_VERSION.into(),
        output_profile: TWO_GROUP_OLS_INVARIANCE_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-invariance", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    kind: IndicatorKind,
    reference: &[InvarianceObservation],
    comparison: &[InvarianceObservation],
) -> Result<analysis_engine::TwoGroupOlsInvarianceExecution, AnalysisEngineError> {
    execute_two_group_ols_invariance_run(
        request,
        accepted,
        snapshot_id,
        knowledge_cutoff,
        kind,
        reference,
        comparison,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn noiseless_strict_series_emit_digest_bound_latent_mean_difference() {
    let request = request();
    let accepted = accepted(&request);
    let reference = strict_reference();
    let comparison = strict_comparison();
    let execution = execute(
        &request,
        &accepted,
        "snapshot-invariance",
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &reference,
        &comparison,
    )
    .expect("execution");

    assert_eq!(
        execution.artifact.schema_version,
        TWO_GROUP_OLS_INVARIANCE_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.reference_observation_count, 3);
    assert_eq!(execution.artifact.comparison_observation_count, 3);
    assert_eq!(execution.artifact.excluded_after_cutoff_reference_count, 0);
    assert_eq!(execution.artifact.excluded_after_cutoff_comparison_count, 0);
    assert_eq!(execution.artifact.indicator_kind, "alr");
    assert_eq!(execution.artifact.invariance_status, "strict");
    assert_eq!(execution.artifact.measurement_invariance_wire_name, None);
    assert!(execution.artifact.licenses_latent_mean_comparison);
    assert!((execution.artifact.latent_mean_difference - 2.0).abs() < 1e-12);
    assert!((execution.artifact.reference_intercept - 0.5).abs() < 1e-12);
    assert!((execution.artifact.reference_loading - 1.2).abs() < 1e-12);
    assert!((execution.artifact.comparison_intercept - 0.5).abs() < 1e-12);
    assert!((execution.artifact.comparison_loading - 1.2).abs() < 1e-12);
    assert!(execution.artifact.reference_residual_variance.abs() < 1e-12);
    assert!(execution.artifact.comparison_residual_variance.abs() < 1e-12);
    assert_eq!(
        execution.artifact.inference_status,
        "two_group_ols_invariance_not_mgcfa"
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
        Some(TWO_GROUP_OLS_INVARIANCE_ARTIFACT_SCHEMA_VERSION)
    );
    assert!((reference[0].factor_score() + 1.0).abs() < f64::EPSILON);
    assert!((reference[0].indicator() - (0.5 - 1.2)).abs() < 1e-12);
    assert_eq!(
        reference[0].available_time(),
        available("2026-07-01T00:00:00Z")
    );
}

#[test]
fn two_observation_series_cap_at_strong_and_recover_difference() {
    let request = request();
    let accepted = accepted(&request);
    let reference = series(&[-1.0, 1.0], 0.5, 1.2);
    let comparison = series(&[0.0, 2.0], 0.5, 1.2);
    let execution = execute(
        &request,
        &accepted,
        "snapshot-invariance",
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &reference,
        &comparison,
    )
    .expect("execution");
    assert_eq!(execution.artifact.invariance_status, "strong");
    assert_eq!(
        execution
            .artifact
            .measurement_invariance_wire_name
            .as_deref(),
        Some("scalar")
    );
    assert!(execution.artifact.licenses_latent_mean_comparison);
    assert!((execution.artifact.latent_mean_difference - 1.0).abs() < 1e-12);
    assert_eq!(
        execution.artifact.reference_residual_variance.to_bits(),
        0.0_f64.to_bits()
    );
    assert_eq!(
        execution.artifact.comparison_residual_variance.to_bits(),
        0.0_f64.to_bits()
    );
}

#[test]
fn execution_excludes_rows_unavailable_at_the_request_cutoff() {
    let request = request();
    let accepted = accepted(&request);
    let reference = strict_reference();
    let mut comparison = strict_comparison();
    comparison.push(
        InvarianceObservation::new(10.0, 100.0, available("2026-08-15T00:00:00Z")).expect("late"),
    );
    let execution = execute(
        &request,
        &accepted,
        "snapshot-invariance",
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &reference,
        &comparison,
    )
    .expect("execution");
    assert_eq!(execution.artifact.comparison_observation_count, 3);
    assert_eq!(execution.artifact.excluded_after_cutoff_comparison_count, 1);
    assert!((execution.artifact.latent_mean_difference - 2.0).abs() < 1e-12);
    assert_eq!(execution.artifact.invariance_status, "strict");
}

#[test]
fn metric_only_and_configural_refuse_latent_means() {
    let request = request();
    let accepted = accepted(&request);
    let reference = strict_reference();
    let metric_only = series(&[1.0, 2.0, 3.0], 1.5, 1.2);
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-invariance",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &reference,
            &metric_only,
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::StrongInvarianceRequired
        ))
    );

    let configural = series(&[1.0, 2.0, 3.0], 0.5, 0.4);
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-invariance",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &reference,
            &configural,
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::StrongInvarianceRequired
        ))
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let accepted = accepted(&request);
    let reference = strict_reference();
    let comparison = strict_comparison();
    assert_eq!(
        execute(
            &request,
            &accepted,
            "other-snapshot",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &reference,
            &comparison,
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
            value.output_profile = "other-profile".into();
            value
        },
    ] {
        assert_eq!(
            execute(
                &invalid_request,
                &accepted,
                "snapshot-invariance",
                cutoff(),
                IndicatorKind::AdditiveLogRatio,
                &reference,
                &comparison,
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}

#[test]
fn constructor_and_empty_cutoff_fail_closed() {
    let request = request();
    let accepted = accepted(&request);
    assert_eq!(
        InvarianceObservation::new(f64::NAN, 1.0, available("2026-07-01T00:00:00Z")),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        InvarianceObservation::new(1.0, f64::INFINITY, available("2026-07-01T00:00:00Z")),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut early_request = request.clone();
    early_request.knowledge_cutoff = "2026-06-01T00:00:00Z".into();
    let too_early = KnowledgeCutoff::parse_rfc3339("2026-06-01T00:00:00Z").expect("cutoff");
    assert_eq!(
        execute(
            &early_request,
            &accepted,
            "snapshot-invariance",
            too_early,
            IndicatorKind::AdditiveLogRatio,
            &strict_reference(),
            &strict_comparison(),
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput
        ))
    );
}

#[test]
fn execution_refuses_raw_proportion_and_singular_loading() {
    let request = request();
    let accepted = accepted(&request);
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-invariance",
            cutoff(),
            IndicatorKind::RawProportion,
            &strict_reference(),
            &strict_comparison(),
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::RawProportionForbidden
        ))
    );

    let zero = series(&[-1.0, 0.0, 1.0], 2.0, 0.0);
    let other = series(&[0.0, 1.0, 2.0], 2.0, 0.0);
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-invariance",
            cutoff(),
            IndicatorKind::IsometricLogRatio,
            &zero,
            &other,
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::SingularDesign
        ))
    );
}

#[test]
fn execution_refuses_receipt_mismatch_and_oversized_corpus() {
    let request = request();
    let accepted = accepted(&request);
    let wrong_receipt =
        AnalysisRunAccepted::new("run-invariance", "accepted", "other-key").expect("accepted");
    assert_eq!(
        execute(
            &request,
            &wrong_receipt,
            "snapshot-invariance",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &strict_reference(),
            &strict_comparison(),
        )
        .expect_err("receipt"),
        AnalysisEngineError::Api(tepp_api::ApiError::InvalidWirePayload)
    );

    let oversized_reference =
        vec![
            InvarianceObservation::new(1.0, 1.0, available("2026-07-01T00:00:00Z")).expect("row");
            MAX_EVIDENCE_UNITS
        ];
    let oversized_comparison =
        vec![InvarianceObservation::new(2.0, 2.0, available("2026-07-01T00:00:00Z")).expect("row")];
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-invariance",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &oversized_reference,
            &oversized_comparison,
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
