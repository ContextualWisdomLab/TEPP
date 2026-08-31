//! End-to-end contract for cutoff-safe independent TDT link-criterion fitting.

use analysis_engine::{
    AnalysisEngineError, LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION,
    LINEAGE_CRITERION_MODEL_CONTRACT_VERSION, LINEAGE_CRITERION_OUTPUT_PROFILE,
    LineageCriterionInput, LineageCriterionObservation, execute_lineage_criterion_run,
};
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn observation(pair_id: &str, successes: u32, trials: u32) -> LineageCriterionObservation {
    LineageCriterionObservation {
        pair_id: pair_id.into(),
        successes,
        trials,
        predecessor_event_time_draws: vec!["2026-01-01T00:00:00Z".into(); 32],
        successor_event_time_draws: vec!["2026-01-02T00:00:00Z".into(); 32],
    }
}

fn observations() -> Vec<LineageCriterionObservation> {
    vec![
        observation("pair-a", 1_000, 10_000),
        observation("pair-b", 5_000, 10_000),
    ]
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "lineage-criterion-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-lineage-criterion".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: LINEAGE_CRITERION_MODEL_CONTRACT_VERSION.into(),
        output_profile: LINEAGE_CRITERION_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-lineage-criterion",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
) -> Result<analysis_engine::LineageCriterionExecution, AnalysisEngineError> {
    let observations = observations();
    execute_lineage_criterion_run(
        request,
        &accepted(request),
        "snapshot-lineage-criterion",
        cutoff(),
        &LineageCriterionInput::new(&observations, 32),
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn identified_pairs_emit_digest_bound_counts_without_inferring_dates() {
    let request = request();
    let execution = execute(&request).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.pair_count, 2);
    assert_eq!(execution.artifact.draw_count, 32);
    assert_eq!(
        execution.artifact.inference_status,
        "independent_tdt_criterion_not_date_from_record_order"
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
        Some(LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn invalid_observations_and_criterion_refusal_fail_closed() {
    let request = request();
    assert_eq!(
        execute_lineage_criterion_run(
            &request,
            &accepted(&request),
            "snapshot-lineage-criterion",
            cutoff(),
            &LineageCriterionInput::new(&[], 32),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let observations = observations();
    assert_eq!(
        execute_lineage_criterion_run(
            &request,
            &accepted(&request),
            "snapshot-lineage-criterion",
            cutoff(),
            &LineageCriterionInput::new(&observations, 0),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let invalid = vec![observation("pair-a", 3, 2)];
    assert_eq!(
        execute_lineage_criterion_run(
            &request,
            &accepted(&request),
            "snapshot-lineage-criterion",
            cutoff(),
            &LineageCriterionInput::new(&invalid, 32),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::LineageCriterionFitFailure)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let observations = observations();
    assert_eq!(
        execute_lineage_criterion_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &LineageCriterionInput::new(&observations, 32),
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
            value.output_profile = "case_deletion_refit_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "composed_fitted_lineage_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "fitted_candidate_k_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "pareto_candidate_k_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "trsl_topic_lineage_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "method_effects_v1".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
