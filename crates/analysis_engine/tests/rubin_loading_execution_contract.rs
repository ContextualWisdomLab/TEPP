//! End-to-end contract for cutoff-safe Rubin loading uncertainty.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION,
    RUBIN_LOADING_MODEL_CONTRACT_VERSION, RUBIN_LOADING_OUTPUT_PROFILE, RubinLoadingObservation,
    RubinLoadingUncertaintyArtifact, execute_rubin_loading_uncertainty_run,
};
use psychometric_core::{IndicatorKind, PsychometricError};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

const SNAPSHOT_ID: &str = "snapshot-rubin-loading";

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn observation(
    snapshot_id: &str,
    factor_score: f64,
    indicator_draws: Vec<f64>,
    available_at: &str,
) -> RubinLoadingObservation {
    RubinLoadingObservation::new(
        snapshot_id,
        factor_score,
        indicator_draws,
        available(available_at),
    )
    .expect("observation")
}

fn noiseless_rows() -> Vec<RubinLoadingObservation> {
    vec![
        observation(SNAPSHOT_ID, -1.0, vec![-0.7, -0.9], "2026-07-01T00:00:00Z"),
        observation(SNAPSHOT_ID, 0.0, vec![0.0, 0.0], "2026-07-01T00:00:00Z"),
        observation(SNAPSHOT_ID, 1.0, vec![0.7, 0.9], "2026-07-01T00:00:00Z"),
    ]
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "rubin-loading-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: SNAPSHOT_ID.into(),
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
        SNAPSHOT_ID,
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
        execution
            .terminal_result
            .summary
            .as_ref()
            .expect("summary")
            .validation_status,
        "validated"
    );
    assert_eq!(
        execution.terminal_result.result_sha256.as_deref(),
        Some(execution.artifact.sha256().expect("digest").as_str())
    );
    assert_eq!(
        execution.terminal_result.result_schema_version.as_deref(),
        Some(RUBIN_LOADING_ARTIFACT_SCHEMA_VERSION)
    );
    assert_eq!(rows[0].snapshot_id(), SNAPSHOT_ID);
    assert!((rows[0].factor_score() + 1.0).abs() < f64::EPSILON);
    assert_eq!(rows[0].indicator_draws(), &[-0.7, -0.9]);
    assert_eq!(rows[0].available_time(), available("2026-07-01T00:00:00Z"));
}

#[test]
fn future_unavailable_rows_do_not_change_historical_replay() {
    let request = request();
    let accepted = accepted(&request);
    let baseline = execute(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &noiseless_rows(),
    )
    .expect("baseline");

    let mut rows = noiseless_rows();
    rows.push(observation(
        SNAPSHOT_ID,
        2.0,
        vec![10.0, 10.0, 10.0],
        "2026-08-15T00:00:00Z",
    ));
    let replay = execute(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &rows,
    )
    .expect("historical replay");

    assert_eq!(replay.artifact.observation_count, baseline.artifact.observation_count);
    assert_eq!(replay.artifact.excluded_after_cutoff_count, 1);
    assert_eq!(replay.artifact.point_estimate_mean, baseline.artifact.point_estimate_mean);
    assert_eq!(replay.artifact.mean_loading, baseline.artifact.mean_loading);
    assert_eq!(replay.artifact.total_variance, baseline.artifact.total_variance);
}

#[test]
fn cross_snapshot_rows_fail_before_scientific_composition() {
    let request = request();
    let accepted = accepted(&request);
    let mut rows = noiseless_rows();
    rows.push(observation(
        "other-snapshot",
        2.0,
        vec![10.0, 10.0, 10.0],
        "2026-08-15T00:00:00Z",
    ));
    assert_eq!(
        execute(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &rows,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn equivalent_rfc3339_cutoff_instants_bind_identically() {
    let mut request = request();
    request.knowledge_cutoff = "2026-08-01T01:00:00+01:00".into();
    let accepted = accepted(&request);
    let execution = execute(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &noiseless_rows(),
    )
    .expect("same instant");
    assert_eq!(execution.artifact.knowledge_cutoff, "2026-08-01T00:00:00Z");
}

#[test]
fn robust_point_estimate_is_not_replaced_by_naive_rubin_mean() {
    let request = request();
    let accepted = accepted(&request);
    let rows = vec![
        observation(
            SNAPSHOT_ID,
            -1.0,
            vec![-1.0e16, -1.0, 1.0e16],
            "2026-07-01T00:00:00Z",
        ),
        observation(
            SNAPSHOT_ID,
            0.0,
            vec![0.0, 0.0, 0.0],
            "2026-07-01T00:00:00Z",
        ),
        observation(
            SNAPSHOT_ID,
            1.0,
            vec![1.0e16, 1.0, -1.0e16],
            "2026-07-01T00:00:00Z",
        ),
    ];
    let execution = execute(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &rows,
    )
    .expect("execution");
    assert_eq!(execution.artifact.mean_loading, 0.0);
    assert!(execution.artifact.point_estimate_mean.abs() > 0.1);
    assert_ne!(
        execution.artifact.point_estimate_mean.to_bits(),
        execution.artifact.mean_loading.to_bits()
    );
}

#[test]
fn artifact_refuses_inconsistent_rubin_total_and_unreachable_counts() {
    let request = request();
    let accepted = accepted(&request);
    let execution = execute(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        IndicatorKind::AdditiveLogRatio,
        &noiseless_rows(),
    )
    .expect("valid artifact");
    let canonical = execution.artifact.to_json().expect("artifact json");
    let mut value: serde_json::Value = serde_json::from_str(&canonical).expect("valid json");

    value["total_variance"] = serde_json::json!(0.04);
    assert_eq!(
        RubinLoadingUncertaintyArtifact::from_json(&value.to_string()),
        Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)
    );

    let mut oversized: serde_json::Value = serde_json::from_str(&canonical).expect("valid json");
    oversized["observation_count"] =
        serde_json::json!(u64::try_from(MAX_EVIDENCE_UNITS).expect("bound") + 1);
    assert_eq!(
        RubinLoadingUncertaintyArtifact::from_json(&oversized.to_string()),
        Err(AnalysisEngineError::InvalidRubinLoadingUncertaintyArtifact)
    );
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
                SNAPSHOT_ID,
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
        RubinLoadingObservation::new(
            "",
            1.0,
            vec![1.0, 2.0],
            available("2026-07-01T00:00:00Z"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        RubinLoadingObservation::new(
            SNAPSHOT_ID,
            f64::NAN,
            vec![1.0, 2.0],
            available("2026-07-01T00:00:00Z"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        RubinLoadingObservation::new(
            SNAPSHOT_ID,
            1.0,
            vec![],
            available("2026-07-01T00:00:00Z"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        RubinLoadingObservation::new(
            SNAPSHOT_ID,
            1.0,
            vec![1.0, f64::NAN],
            available("2026-07-01T00:00:00Z"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        RubinLoadingObservation::new(
            SNAPSHOT_ID,
            1.0,
            vec![1.0; 257],
            available("2026-07-01T00:00:00Z"),
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );

    let mut early_request = request.clone();
    early_request.knowledge_cutoff = "2026-06-01T00:00:00Z".into();
    let too_early = KnowledgeCutoff::parse_rfc3339("2026-06-01T00:00:00Z").expect("cutoff");
    assert_eq!(
        execute(
            &early_request,
            &accepted,
            SNAPSHOT_ID,
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
            SNAPSHOT_ID,
            cutoff(),
            IndicatorKind::RawProportion,
            &noiseless_rows(),
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::RawProportionForbidden
        ))
    );

    let single_draw = vec![
        observation(SNAPSHOT_ID, -1.0, vec![-0.7], "2026-07-01T00:00:00Z"),
        observation(SNAPSHOT_ID, 1.0, vec![0.7], "2026-07-01T00:00:00Z"),
    ];
    assert_eq!(
        execute(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &single_draw,
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InsufficientDraws
        ))
    );

    let unequal = vec![
        observation(SNAPSHOT_ID, -1.0, vec![-0.7, -0.9], "2026-07-01T00:00:00Z"),
        observation(SNAPSHOT_ID, 1.0, vec![0.7, 0.9, 1.1], "2026-07-01T00:00:00Z"),
    ];
    assert_eq!(
        execute(
            &request,
            &accepted,
            SNAPSHOT_ID,
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
fn matrix_resource_budget_fails_before_scientific_fit() {
    let request = request();
    let accepted = accepted(&request);
    let rows = vec![
        observation(
            SNAPSHOT_ID,
            1.0,
            vec![1.0; 256],
            "2026-07-01T00:00:00Z",
        );
        3_907
    ];
    assert_eq!(
        execute(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &rows,
        ),
        Err(AnalysisEngineError::LimitExceeded)
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
            SNAPSHOT_ID,
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &noiseless_rows(),
        )
        .expect_err("receipt"),
        AnalysisEngineError::Api(tepp_api::ApiError::InvalidWirePayload)
    );

    let oversized = vec![
        observation(
            SNAPSHOT_ID,
            1.0,
            vec![1.0, 2.0],
            "2026-07-01T00:00:00Z",
        );
        MAX_EVIDENCE_UNITS + 1
    ];
    assert_eq!(
        execute(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &oversized,
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );
}

#[test]
fn execution_refuses_invalid_completion_time() {
    let request = request();
    let accepted = accepted(&request);
    assert_eq!(
        execute_rubin_loading_uncertainty_run(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            IndicatorKind::AdditiveLogRatio,
            &noiseless_rows(),
            "invalid",
        ),
        Err(AnalysisEngineError::Api(
            tepp_api::ApiError::InvalidWirePayload
        ))
    );
}
