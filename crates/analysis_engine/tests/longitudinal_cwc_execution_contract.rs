//! End-to-end contract for cutoff-safe longitudinal CWC composition.

use analysis_engine::{
    AnalysisEngineError, LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION,
    LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION, LONGITUDINAL_CWC_OUTPUT_PROFILE,
    LongitudinalClusterScore, MAX_EVIDENCE_UNITS, execute_longitudinal_cwc_run,
};
use psychometric_core::PsychometricError;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

const SNAPSHOT_ID: &str = "snapshot-longitudinal-cwc";

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn score(
    cluster_key: u64,
    predictor: f64,
    outcome: f64,
    available_at: &str,
) -> LongitudinalClusterScore {
    let evidence_id = format!(
        "evidence-{cluster_key}-{:016x}-{:016x}-{available_at}",
        predictor.to_bits(),
        outcome.to_bits()
    );
    LongitudinalClusterScore::new(
        evidence_id,
        SNAPSHOT_ID,
        cluster_key,
        predictor,
        outcome,
        available(available_at),
    )
    .expect("score")
}

fn noiseless_rows() -> Vec<LongitudinalClusterScore> {
    vec![
        score(1, 0.0, 2.0, "2026-07-01T00:00:00Z"),
        score(1, 2.0, 3.0, "2026-07-01T00:00:00Z"),
        score(2, 4.0, 10.0, "2026-07-01T00:00:00Z"),
        score(2, 6.0, 11.0, "2026-07-01T00:00:00Z"),
    ]
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "longitudinal-cwc-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: SNAPSHOT_ID.into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: LONGITUDINAL_CWC_MODEL_CONTRACT_VERSION.into(),
        output_profile: LONGITUDINAL_CWC_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-longitudinal-cwc", "accepted", &request.idempotency_key)
        .expect("accepted")
}

#[test]
fn noiseless_cwc_emits_digest_bound_within_between_and_contextual() {
    let request = request();
    let accepted = accepted(&request);
    let rows = noiseless_rows();
    let execution = execute_longitudinal_cwc_run(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        &rows,
        "2026-08-02T00:00:00Z",
    )
    .expect("execution");

    assert_eq!(
        execution.artifact.schema_version,
        LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.row_count, 4);
    assert_eq!(execution.artifact.cluster_count, 2);
    assert!((execution.artifact.within_slope - 0.5).abs() < 1e-12);
    assert!((execution.artifact.between_slope - 2.0).abs() < 1e-12);
    assert!((execution.artifact.contextual_effect - 1.5).abs() < 1e-12);
    assert_eq!(
        execution.artifact.inference_status,
        "composed_cwc_slopes_not_causal"
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
        Some(LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION)
    );
    assert!(!rows[0].evidence_id().is_empty());
    assert_eq!(rows[0].snapshot_id(), SNAPSHOT_ID);
    assert_eq!(rows[0].cluster_key(), 1);
    assert!((rows[0].predictor() - 0.0).abs() < f64::EPSILON);
    assert!((rows[0].outcome() - 2.0).abs() < f64::EPSILON);
    assert_eq!(rows[0].available_time(), available("2026-07-01T00:00:00Z"));
}

#[test]
fn execution_excludes_future_rows_without_changing_historical_output() {
    let request = request();
    let accepted = accepted(&request);
    let baseline_rows = noiseless_rows();
    let baseline = execute_longitudinal_cwc_run(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        &baseline_rows,
        "2026-08-02T00:00:00Z",
    )
    .expect("baseline");

    let mut replay_rows = baseline_rows;
    replay_rows.push(score(3, 8.0, 20.0, "2026-08-15T00:00:00Z"));
    let replay = execute_longitudinal_cwc_run(
        &request,
        &accepted,
        SNAPSHOT_ID,
        cutoff(),
        &replay_rows,
        "2026-08-02T00:00:00Z",
    )
    .expect("replay");

    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result, baseline.terminal_result);
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let accepted = accepted(&request);
    let rows = noiseless_rows();
    assert_eq!(
        execute_longitudinal_cwc_run(
            &request,
            &accepted,
            "other-snapshot",
            cutoff(),
            &rows,
            "2026-08-02T00:00:00Z",
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
            execute_longitudinal_cwc_run(
                &invalid_request,
                &accepted,
                SNAPSHOT_ID,
                cutoff(),
                &rows,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}

#[test]
fn execution_refuses_empty_cutoff_one_cluster_and_receipt_mismatch() {
    let request = request();
    let accepted = accepted(&request);
    assert_eq!(
        LongitudinalClusterScore::new(
            "evidence-invalid-predictor",
            SNAPSHOT_ID,
            1,
            f64::NAN,
            1.0,
            available("2026-07-01T00:00:00Z")
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        LongitudinalClusterScore::new(
            "evidence-invalid-outcome",
            SNAPSHOT_ID,
            1,
            1.0,
            f64::NAN,
            available("2026-07-01T00:00:00Z")
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        LongitudinalClusterScore::new(
            "evidence-invalid-snapshot",
            "",
            1,
            1.0,
            1.0,
            available("2026-07-01T00:00:00Z")
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut early_request = request.clone();
    early_request.knowledge_cutoff = "2026-06-01T00:00:00Z".into();
    let too_early = KnowledgeCutoff::parse_rfc3339("2026-06-01T00:00:00Z").expect("cutoff");
    assert_eq!(
        execute_longitudinal_cwc_run(
            &early_request,
            &accepted,
            SNAPSHOT_ID,
            too_early,
            &noiseless_rows(),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InvalidNumericInput
        ))
    );

    let late_cluster_two = vec![
        score(1, 0.0, 2.0, "2026-07-01T00:00:00Z"),
        score(1, 2.0, 3.0, "2026-07-01T00:00:00Z"),
        score(2, 4.0, 10.0, "2026-08-15T00:00:00Z"),
        score(2, 6.0, 11.0, "2026-08-15T00:00:00Z"),
    ];
    assert_eq!(
        execute_longitudinal_cwc_run(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            &late_cluster_two,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::Psychometric(
            PsychometricError::InsufficientClusters
        ))
    );

    let wrong_receipt = AnalysisRunAccepted::new("run-longitudinal-cwc", "accepted", "other-key")
        .expect("accepted");
    assert_eq!(
        execute_longitudinal_cwc_run(
            &request,
            &wrong_receipt,
            SNAPSHOT_ID,
            cutoff(),
            &noiseless_rows(),
            "2026-08-02T00:00:00Z",
        )
        .expect_err("receipt"),
        AnalysisEngineError::Api(tepp_api::ApiError::InvalidWirePayload)
    );

    let oversized = vec![score(1, 0.0, 1.0, "2026-07-01T00:00:00Z"); MAX_EVIDENCE_UNITS + 1];
    assert_eq!(
        execute_longitudinal_cwc_run(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            &oversized,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );
}

#[test]
fn execution_refuses_invalid_completion_time() {
    let request = request();
    let accepted = accepted(&request);
    assert_eq!(
        execute_longitudinal_cwc_run(
            &request,
            &accepted,
            SNAPSHOT_ID,
            cutoff(),
            &noiseless_rows(),
            "invalid",
        ),
        Err(AnalysisEngineError::Api(
            tepp_api::ApiError::InvalidWirePayload
        ))
    );
}
