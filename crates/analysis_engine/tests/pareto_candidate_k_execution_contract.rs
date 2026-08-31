//! End-to-end contract for cutoff-safe Pareto candidate-`K` selection.

use analysis_engine::{
    AnalysisEngineError, PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION,
    PARETO_CANDIDATE_K_MODEL_CONTRACT_VERSION, PARETO_CANDIDATE_K_OUTPUT_PROFILE,
    ParetoCandidateKInput, execute_pareto_candidate_k_run,
};
use model_selection::{ModelCandidate, ModelSelectionError};
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "pareto-candidate-k-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-pareto-candidate-k".into(),
        knowledge_cutoff: "2026-02-01T00:00:00Z".into(),
        model_contract_version: PARETO_CANDIDATE_K_MODEL_CONTRACT_VERSION.into(),
        output_profile: PARETO_CANDIDATE_K_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-pareto-candidate-k",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn statistical_front() -> ParetoCandidateKInput {
    ParetoCandidateKInput::new(
        vec![
            ModelCandidate::statistical(2, -30.0, 8.0).expect("k2"),
            ModelCandidate::statistical(4, -30.0, 8.0).expect("k4"),
            ModelCandidate::llm_vote_only(8).expect("llm"),
        ],
        vec![2, 2, 2],
        2,
    )
}

fn execute(
    request: &AnalysisRunRequest,
    input: &ParetoCandidateKInput,
) -> Result<analysis_engine::ParetoCandidateKExecution, AnalysisEngineError> {
    execute_pareto_candidate_k_run(
        request,
        &accepted(request),
        "snapshot-pareto-candidate-k",
        cutoff(),
        input,
        "2026-02-02T00:00:00Z",
    )
}

#[test]
fn pareto_front_selects_smaller_k_and_refuses_llm_vote_as_authority() {
    let request = request();
    let execution = execute(&request, &statistical_front()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.selected_k, 2);
    assert_eq!(execution.artifact.candidate_count, 3);
    assert_eq!(execution.artifact.statistical_count, 2);
    assert_eq!(execution.artifact.truth_k, 2);
    assert!((execution.artifact.selected_k_rmse - 0.0).abs() < f64::EPSILON);
    assert_eq!(
        execution.artifact.inference_status,
        "pareto_statistical_front_not_fitted_schwarz_sampler"
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
        Some(PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn higher_likelihood_wins_and_llm_only_sets_fail_closed() {
    let request = request();
    let higher = ParetoCandidateKInput::new(
        vec![
            ModelCandidate::statistical(2, -30.0, 8.0).expect("k2"),
            ModelCandidate::statistical(8, -20.0, 9.0).expect("k8"),
        ],
        vec![8],
        8,
    );
    let execution = execute(&request, &higher).expect("likelihood");
    assert_eq!(execution.artifact.selected_k, 8);
    assert!((execution.artifact.selected_k_rmse - 0.0).abs() < f64::EPSILON);

    let llm_only = ParetoCandidateKInput::new(
        vec![ModelCandidate::llm_vote_only(3).expect("llm")],
        vec![3],
        3,
    );
    assert_eq!(
        execute(&request, &llm_only),
        Err(AnalysisEngineError::ModelSelection(
            ModelSelectionError::LlmVoteIsNotStatisticalAuthority
        ))
    );
    let empty = ParetoCandidateKInput::new(Vec::new(), vec![2], 2);
    assert_eq!(
        execute(&request, &empty),
        Err(AnalysisEngineError::ModelSelection(
            ModelSelectionError::EmptyCandidateSet
        ))
    );
}

#[test]
fn mismatched_replications_record_positive_rmse() {
    let request = request();
    let mismatched = ParetoCandidateKInput::new(
        vec![ModelCandidate::statistical(2, -30.0, 8.0).expect("k2")],
        vec![4, 4, 4],
        2,
    );
    let execution = execute(&request, &mismatched).expect("rmse");
    assert_eq!(execution.artifact.selected_k, 2);
    assert!((execution.artifact.selected_k_rmse - 2.0).abs() < f64::EPSILON);
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    assert_eq!(
        execute_pareto_candidate_k_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &statistical_front(),
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
            value.output_profile = "fitted_candidate_k_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "joint_posterior_draws_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "trsl_topic_lineage_v1".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request, &statistical_front()),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
