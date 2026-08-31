//! End-to-end contract for cutoff-safe Rubin loading uncertainty.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION,
    RUBIN_LOADING_MODEL_CONTRACT_VERSION, RUBIN_LOADING_OUTPUT_PROFILE, RubinLoadingObservation,
    execute_rubin_loading_uncertainty_run,
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

fn noiseless_rows() -> Vec<RubinLoadingObservation> {
    vec![
        RubinLoadingObservation::new(-1.0, vec![-0.7, -0.9], available("2026-07-01T00:00:00Z"))
            .expect("r1"),
        RubinLoadingObservation::new(0.0, vec![0.0, 0.0], available("2026-07-01T00:00:00Z"))
            .expect("r2"),
        RubinLoadingObservation::new(1.0, vec![0.7, 0.9], available("2026-07-01T00:00:00Z"))
            .expect("r3"),
    ]
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "rubin-loading-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-rubin-loading".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: RUBIN_LOADING_MODEL_CONTRACT_VERSION.into(),
        output_profile: RUBIN_LOADING_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-rubin-loading", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
    accepted: &AnalysisRunAccepted,
    snapshot_id: &str,
    knowledge_cutoff: KnowledgeCutoff,
    kind: IndicatorKind,
    observations: &[RubinLoadingObservation],
) -> Result<analysis_engine::RubinLoadingUncertaintyExecution, AnalysisEngineError> {
    execute_rubin_loading_uncertainty_run(
        request,
        accepted,
        snapshot_id,
        knowledge_cutoff,
        kind,
        observations,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn noiseless_draws_emit_digest_bound_point_mean_and_rubin_t() {
    let request = request();
    let accepted = accepted(&request);
    let rows = noiseless_rows();
    let execution = execute(
        &request,
        &accepted,
        "snapshot-rubin-loading",
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &rows,
    )
    .expect("execution");

    assert_eq!(
        execution.artifact.schema_version,
        RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.observation_count, 3);
    assert_eq!(execution.artifact.draw_count, 2);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 0);
    assert_eq!(execution.artifact.indicator_kind, "alr");
    assert!((execution.artifact.point_estimate_mean - 0.8).abs() < 1e-12);
    assert!((execution.artifact.mean_loading - 0.8).abs() < 1e-12);
    assert!(execution.artifact.within_variance.abs() < 1e-12);
    assert!(execution.artifact.between_variance > 0.0);
    let expected_total = execution.artifact.within_variance
        + (1.0 + 1.0 / 2.0) * execution.artifact.between_variance;
    assert!((execution.artifact.total_variance - expected_total).abs() < 1e-15);
    assert_eq!(
        execution.artifact.inference_status,
        "rubin_combined_ols_loadings_not_mislevy_pv"
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
        Some(RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION)
    );
    assert!((rows[0].factor_score() + 1.0).abs() < f64::EPSILON);
    assert_eq!(rows[0].indicator_draws(), &[-0.7, -0.9]);
    assert_eq!(rows[0].available_time(), available("2026-07-01T00:00:00Z"));
}

#[test]
fn execution_excludes_rows_unavailable_at_the_request_cutoff() {
    let request = request();
    let accepted = accepted(&request);
    let mut rows = noiseless_rows();
    rows.push(
        RubinLoadingObservation::new(2.0, vec![10.0, 10.0], available("2026-08-15T00:00:00Z"))
            .expect("late"),
    );
    let execution = execute(
        &request,
        &accepted,
        "snapshot-rubin-loading",
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &rows,
    )
    .expect("execution");
    assert_eq!(execution.artifact.observation_count, 3);
    assert_eq!(execution.artifact.excluded_after_cutoff_count, 1);
    assert!((execution.artifact.mean_loading - 0.8).abs() < 1e-12);
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let accepted = accepted(&request);
    let rows = noiseless_rows();
    assert_eq!(
        execute(
            &request,
            &accepted,
            "other-snapshot",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &rows,
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
                "snapshot-rubin-loading",
                cutoff(),
                IndicatorKind::AdditiveLogRatio,
                &rows,
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
        RubinLoadingObservation::new(f64::NAN, vec![1.0, 2.0], available("2026-07-01T00:00:00Z")),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        RubinLoadingObservation::new(1.0, vec![], available("2026-07-01T00:00:00Z")),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        RubinLoadingObservation::new(1.0, vec![1.0, f64::NAN], available("2026-07-01T00:00:00Z")),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut early_request = request.clone();
    early_request.knowledge_cutoff = "2026-06-01T00:00:00Z".into();
    let too_early = KnowledgeCutoff::parse_rfc3339("2026-06-01T00:00:00Z").expect("cutoff");
    assert_eq!(
        execute(
            &early_request,
            &accepted,
            "snapshot-rubin-loading",
            too_early,
            IndicatorKind::AdditiveLogRatio,
            &noiseless_rows(),
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput
        ))
    );
}

#[test]
fn execution_refuses_raw_proportion_single_draw_and_unequal_lengths() {
    let request = request();
    let accepted = accepted(&request);
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-rubin-loading",
            cutoff(),
            IndicatorKind::RawProportion,
            &noiseless_rows(),
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::RawProportionForbidden
        ))
    );

    let single_draw = vec![
        RubinLoadingObservation::new(-1.0, vec![-0.7], available("2026-07-01T00:00:00Z"))
            .expect("d1"),
        RubinLoadingObservation::new(1.0, vec![0.7], available("2026-07-01T00:00:00Z"))
            .expect("d2"),
    ];
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-rubin-loading",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &single_draw,
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InsufficientDraws
        ))
    );

    let unequal = vec![
        RubinLoadingObservation::new(-1.0, vec![-0.7, -0.9], available("2026-07-01T00:00:00Z"))
            .expect("u1"),
        RubinLoadingObservation::new(1.0, vec![0.7, 0.9, 1.1], available("2026-07-01T00:00:00Z"))
            .expect("u2"),
    ];
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-rubin-loading",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &unequal,
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput
        ))
    );
}

#[test]
fn execution_refuses_receipt_mismatch_and_oversized_corpus() {
    let request = request();
    let accepted = accepted(&request);
    let wrong_receipt =
        AnalysisRunAccepted::new("run-rubin-loading", "accepted", "other-key").expect("accepted");
    assert_eq!(
        execute(
            &request,
            &wrong_receipt,
            "snapshot-rubin-loading",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &noiseless_rows(),
        )
        .expect_err("receipt"),
        AnalysisEngineError::Api(tepp_api::ApiError::InvalidWirePayload)
    );

    let oversized =
        vec![
            RubinLoadingObservation::new(1.0, vec![1.0, 2.0], available("2026-07-01T00:00:00Z"))
                .expect("row");
            MAX_EVIDENCE_UNITS + 1
        ];
    assert_eq!(
        execute(
            &request,
            &accepted,
            "snapshot-rubin-loading",
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &oversized,
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
