//! End-to-end contract for cutoff-safe Pareto candidate-`K` selection.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION,
    PARETO_CANDIDATE_K_MODEL_CONTRACT_VERSION, PARETO_CANDIDATE_K_OUTPUT_PROFILE,
    ParetoCandidateKInput, execute_pareto_candidate_k_run,
};
use model_selection::{ModelCandidate, ModelSelectionError};
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState, ApiError};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff")
}

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
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
        "snapshot-pareto-candidate-k",
        cutoff(),
        vec![
            available("2026-01-10T00:00:00Z"),
            available("2026-01-11T00:00:00Z"),
            available("2026-01-12T00:00:00Z"),
            available("2026-01-13T00:00:00Z"),
            available("2026-01-14T00:00:00Z"),
        ],
        vec![
            ModelCandidate::statistical(2, -30.0, 8.0).expect("k2"),
            ModelCandidate::statistical(4, -30.0, 8.0).expect("k4"),
            ModelCandidate::llm_vote_only(8).expect("llm"),
        ],
        vec![2, 2, 2],
        2,
    )
    .expect("front")
}

fn input(
    candidates: Vec<ModelCandidate>,
    selected_replications: Vec<u32>,
    truth_k: u32,
) -> ParetoCandidateKInput {
    ParetoCandidateKInput::new(
        "snapshot-pareto-candidate-k",
        cutoff(),
        vec![available("2026-01-10T00:00:00Z")],
        candidates,
        selected_replications,
        truth_k,
    )
    .expect("input")
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
fn pareto_front_selects_smaller_k_and_reports_source_evidence_count() {
    let request = request();
    let execution = execute(&request, &statistical_front()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version(),
        PARETO_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.selected_k(), 2);
    assert_eq!(execution.artifact.candidate_count(), 3);
    assert_eq!(execution.artifact.statistical_count(), 2);
    assert_eq!(execution.artifact.truth_k(), 2);
    assert!((execution.artifact.selected_k_rmse() - 0.0).abs() < f64::EPSILON);
    assert_eq!(
        execution.artifact.inference_status(),
        "pareto_statistical_front_not_fitted_schwarz_sampler"
    );
    assert_eq!(
        execution.terminal_result.run_state,
        AnalysisRunTerminalState::Succeeded
    );
    let summary = execution.terminal_result.summary.as_ref().expect("summary");
    assert_eq!(summary.evidence_count, 5);
    assert_ne!(summary.evidence_count, execution.artifact.candidate_count());
    assert_eq!(summary.validation_status, "validated");
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
    let higher = input(
        vec![
            ModelCandidate::statistical(2, -30.0, 8.0).expect("k2"),
            ModelCandidate::statistical(8, -20.0, 9.0).expect("k8"),
        ],
        vec![8],
        8,
    );
    let execution = execute(&request, &higher).expect("likelihood");
    assert_eq!(execution.artifact.selected_k(), 8);
    assert!((execution.artifact.selected_k_rmse() - 0.0).abs() < f64::EPSILON);

    let llm_only = input(
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
    let empty = input(Vec::new(), vec![2], 2);
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
    let mismatched = input(
        vec![ModelCandidate::statistical(2, -30.0, 8.0).expect("k2")],
        vec![4, 4, 4],
        2,
    );
    let execution = execute(&request, &mismatched).expect("rmse");
    assert_eq!(execution.artifact.selected_k(), 2);
    assert!((execution.artifact.selected_k_rmse() - 2.0).abs() < f64::EPSILON);
}

#[test]
fn provenance_rejects_future_evidence_and_resource_exhaustion_before_selection() {
    let candidate = ModelCandidate::statistical(2, -30.0, 8.0).expect("candidate");
    assert_eq!(
        ParetoCandidateKInput::new(
            "snapshot-pareto-candidate-k",
            cutoff(),
            vec![available("2026-02-01T00:00:01Z")],
            vec![candidate],
            vec![2],
            2,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        ParetoCandidateKInput::new(
            "snapshot-pareto-candidate-k",
            cutoff(),
            vec![available("2026-01-10T00:00:00Z")],
            vec![candidate; 257],
            vec![2],
            2,
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );
    assert_eq!(
        ParetoCandidateKInput::new(
            "snapshot-pareto-candidate-k",
            cutoff(),
            vec![available("2026-01-10T00:00:00Z")],
            vec![candidate],
            vec![2; MAX_EVIDENCE_UNITS + 1],
            2,
        ),
        Err(AnalysisEngineError::LimitExceeded)
    );
}

#[test]
fn equivalent_cutoff_instants_bind_and_provenance_snapshot_is_enforced() {
    let mut equivalent = request();
    equivalent.knowledge_cutoff = "2026-01-31T19:00:00-05:00".into();
    assert!(execute(&equivalent, &statistical_front()).is_ok());

    let wrong_snapshot = ParetoCandidateKInput::new(
        "other-snapshot",
        cutoff(),
        vec![available("2026-01-10T00:00:00Z")],
        vec![ModelCandidate::statistical(2, -30.0, 8.0).expect("candidate")],
        vec![2],
        2,
    )
    .expect("input");
    assert_eq!(
        execute(&request(), &wrong_snapshot),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
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

#[test]
fn invalid_completed_at_fails_terminal_result_construction() {
    let request = request();
    assert_eq!(
        execute_pareto_candidate_k_run(
            &request,
            &accepted(&request),
            "snapshot-pareto-candidate-k",
            cutoff(),
            &statistical_front(),
            "not-a-timestamp",
        ),
        Err(AnalysisEngineError::Api(ApiError::InvalidWirePayload))
    );
}
