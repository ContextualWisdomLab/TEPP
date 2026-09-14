//! End-to-end contract for cutoff-safe independent TDT link-criterion fitting.

use analysis_engine::{
    AnalysisEngineError, LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION,
    LINEAGE_CRITERION_MODEL_CONTRACT_VERSION, LINEAGE_CRITERION_OUTPUT_PROFILE,
    LineageCriterionInput, LineageCriterionObservation, execute_lineage_criterion_run,
};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

const SNAPSHOT_ID: &str = "snapshot-lineage-criterion";

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn available_time(value: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(value).expect("available time")
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

fn visible_snapshot_ids(len: usize) -> Vec<String> {
    vec![SNAPSHOT_ID.to_owned(); len]
}

fn visible_available_times(len: usize) -> Vec<AvailableTime> {
    vec![available_time("2026-07-01T00:00:00Z"); len]
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "lineage-criterion-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: SNAPSHOT_ID.into(),
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

fn execute_with_provenance(
    request: &AnalysisRunRequest,
    observations: &[LineageCriterionObservation],
    snapshot_ids: &[String],
    available_times: &[AvailableTime],
    draw_count: usize,
) -> Result<analysis_engine::LineageCriterionExecution, AnalysisEngineError> {
    execute_lineage_criterion_run(
        request,
        &accepted(request),
        SNAPSHOT_ID,
        cutoff(),
        &LineageCriterionInput::new(observations, snapshot_ids, available_times, draw_count),
        "2026-08-02T00:00:00Z",
    )
}

fn execute_with_observations(
    request: &AnalysisRunRequest,
    observations: &[LineageCriterionObservation],
) -> Result<analysis_engine::LineageCriterionExecution, AnalysisEngineError> {
    let snapshot_ids = visible_snapshot_ids(observations.len());
    let available_times = visible_available_times(observations.len());
    execute_with_provenance(
        request,
        observations,
        &snapshot_ids,
        &available_times,
        32,
    )
}

fn execute(
    request: &AnalysisRunRequest,
) -> Result<analysis_engine::LineageCriterionExecution, AnalysisEngineError> {
    execute_with_observations(request, &observations())
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
        Some(LINEAGE_CRITERION_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn equivalent_cutoff_spellings_bind_the_same_instant() {
    let canonical_request = request();
    let baseline = execute(&canonical_request).expect("canonical cutoff");
    let mut offset_request = canonical_request;
    offset_request.knowledge_cutoff = "2026-08-01T01:00:00+01:00".into();
    let equivalent = execute(&offset_request).expect("equivalent instant");
    assert_eq!(equivalent.artifact, baseline.artifact);
    assert_eq!(equivalent.terminal_result.summary, baseline.terminal_result.summary);
}

#[test]
fn future_duplicate_and_malformed_evidence_cannot_change_historical_replay() {
    let request = request();
    let baseline_observations = observations();
    let baseline = execute_with_observations(&request, &baseline_observations).expect("baseline");

    let mut replay_observations = baseline_observations;
    let mut future_duplicate = observation("pair-a", 10_000, 10_000);
    future_duplicate.predecessor_event_time_draws[0] = "future-malformed-time".into();
    replay_observations.push(future_duplicate);
    let replay_snapshot_ids = visible_snapshot_ids(replay_observations.len());
    let mut replay_available_times = visible_available_times(replay_observations.len());
    *replay_available_times.last_mut().expect("future availability") =
        available_time("2026-08-02T00:00:00Z");

    let replay = execute_with_provenance(
        &request,
        &replay_observations,
        &replay_snapshot_ids,
        &replay_available_times,
        32,
    )
    .expect("future evidence must be censored before validation");
    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result.summary, baseline.terminal_result.summary);
}

#[test]
fn cross_snapshot_evidence_and_misaligned_provenance_fail_closed() {
    let request = request();
    let observations = observations();
    let available_times = visible_available_times(observations.len());
    let mut wrong_snapshot_ids = visible_snapshot_ids(observations.len());
    wrong_snapshot_ids[1] = "snapshot-other".into();
    assert_eq!(
        execute_with_provenance(
            &request,
            &observations,
            &wrong_snapshot_ids,
            &available_times,
            32,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let short_snapshot_ids = vec![SNAPSHOT_ID.to_owned()];
    assert_eq!(
        execute_with_provenance(
            &request,
            &observations,
            &short_snapshot_ids,
            &available_times,
            32,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn malformed_visible_event_time_draws_fail_closed() {
    let request = request();
    let mut invalid_predecessor = observations();
    invalid_predecessor[0].predecessor_event_time_draws[0] = "not-an-event-time".into();
    assert_eq!(
        execute_with_observations(&request, &invalid_predecessor),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    let mut invalid_successor = observations();
    invalid_successor[0].successor_event_time_draws[0] = "also-not-an-event-time".into();
    assert_eq!(
        execute_with_observations(&request, &invalid_successor),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn oversized_draw_budget_fails_before_posterior_materialization() {
    let request = request();
    let observations = vec![observation("pair-a", 1, 2)];
    let snapshot_ids = visible_snapshot_ids(1);
    let available_times = visible_available_times(1);
    assert_eq!(
        execute_with_provenance(
            &request,
            &observations,
            &snapshot_ids,
            &available_times,
            1_000_001,
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );
}

#[test]
fn invalid_observations_and_criterion_refusal_fail_closed() {
    let request = request();
    assert_eq!(
        execute_with_provenance(&request, &[], &[], &[], 32),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let observations = observations();
    let snapshot_ids = visible_snapshot_ids(observations.len());
    let available_times = visible_available_times(observations.len());
    assert_eq!(
        execute_with_provenance(
            &request,
            &observations,
            &snapshot_ids,
            &available_times,
            0,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let invalid = vec![observation("pair-a", 3, 2)];
    let snapshot_ids = visible_snapshot_ids(1);
    let available_times = visible_available_times(1);
    assert_eq!(
        execute_with_provenance(
            &request,
            &invalid,
            &snapshot_ids,
            &available_times,
            32,
        ),
        Err(AnalysisEngineError::LineageCriterionFitFailure)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let observations = observations();
    let snapshot_ids = visible_snapshot_ids(observations.len());
    let available_times = visible_available_times(observations.len());
    assert_eq!(
        execute_lineage_criterion_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &LineageCriterionInput::new(
                &observations,
                &snapshot_ids,
                &available_times,
                32,
            ),
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
