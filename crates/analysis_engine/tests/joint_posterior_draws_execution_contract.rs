//! End-to-end contract for cutoff-safe joint posterior Laplace draws.

use analysis_engine::{
    AnalysisEngineError, JOINT_POSTERIOR_DRAWS_ARTIFACT_SCHEMA_VERSION,
    JOINT_POSTERIOR_DRAWS_MODEL_CONTRACT_VERSION, JOINT_POSTERIOR_DRAWS_OUTPUT_PROFILE,
    execute_joint_posterior_draws_run,
};
use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};
use topic_measurement::{
    JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION, ReferenceTopicInput, ReferenceTopicModelConfig,
    SparseMatrix, TopicMeasurementError,
};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-07-{day:02}T00:00:00Z")).expect("event time")
}

fn fixture() -> (
    CorpusSnapshot,
    Vec<Uuid>,
    Vec<EventTime>,
    MembershipNetwork,
    RelationGraph,
) {
    let ids: Vec<_> = (1_u128..=4).map(Uuid::from_u128).collect();
    let times: Vec<_> = (1_u8..=4).map(event_time).collect();
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");
    let available = AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available");
    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();
    for id in &ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &cutoff)
            .expect("eligible");
        memberships
            .insert(
                MembershipAssignment::new(
                    MemberId::from_uuid(*id),
                    GroupId::from_uuid(Uuid::from_u128(100)),
                    MembershipRole::Project,
                    MembershipWeight::full().expect("weight"),
                    event_time(1),
                    event_time(9),
                )
                .expect("membership"),
            )
            .expect("insert");
    }
    let mut relations = RelationGraph::new();
    for (source, target, source_day, target_day) in [(0, 1, 1, 2), (1, 2, 2, 3), (2, 3, 3, 4)] {
        let interval = |day| {
            TemporalInterval::bounded(
                TemporalBoundary::Included(event_time(day)),
                TemporalBoundary::Included(
                    EventTime::parse_rfc3339(&format!("2026-07-{day:02}T12:00:00Z")).expect("end"),
                ),
                TemporalPrecision::Second,
            )
            .expect("interval")
        };
        relations
            .insert(
                RelationEdge::new(
                    RelationKind::TransitionsTo,
                    RelationEndpointId::from_uuid(ids[source]),
                    RelationEndpointId::from_uuid(ids[target]),
                    RelationEvidenceStatus::Observed,
                    interval(source_day),
                    interval(target_day),
                )
                .expect("relation"),
            )
            .expect("insert relation");
    }
    (snapshot, ids, times, memberships, relations)
}

fn separated_input() -> ReferenceTopicInput {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let counts = SparseMatrix::from_csr(
        4,
        4,
        vec![0, 2, 4, 6, 8],
        vec![0, 1, 0, 1, 2, 3, 2, 3],
        vec![90.0, 10.0, 85.0, 15.0, 10.0, 90.0, 15.0, 85.0],
    )
    .expect("counts");
    ReferenceTopicInput::new(
        &snapshot,
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("input")
}

fn recovery_config() -> ReferenceTopicModelConfig {
    ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters")
}

fn topic_ids() -> Vec<Uuid> {
    (1001_u128..=1002).map(Uuid::from_u128).collect()
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "joint-posterior-draws-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-joint-posterior-draws".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: JOINT_POSTERIOR_DRAWS_MODEL_CONTRACT_VERSION.into(),
        output_profile: JOINT_POSTERIOR_DRAWS_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-joint-posterior-draws",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
    draw_count: usize,
) -> Result<analysis_engine::JointPosteriorDrawsExecution, AnalysisEngineError> {
    execute_joint_posterior_draws_run(
        request,
        &accepted(request),
        "snapshot-joint-posterior-draws",
        cutoff(),
        &separated_input(),
        &recovery_config(),
        topic_ids(),
        draw_count,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn fitted_precision_emits_digest_bound_laplace_draws() {
    let request = request();
    let execution = execute(&request, 4).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        JOINT_POSTERIOR_DRAWS_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(
        execution.artifact.algorithm_version,
        JOINT_POSTERIOR_DRAW_ALGORITHM_VERSION
    );
    assert_eq!(execution.artifact.draw_count, 4);
    assert_eq!(execution.artifact.document_count, 4);
    assert_eq!(execution.artifact.topic_count, 2);
    assert_eq!(execution.artifact.draw_set_id.len(), 64);
    assert_eq!(
        execution.artifact.approximation,
        "joint_gauss_newton_laplace"
    );
    assert_eq!(
        execution.artifact.inference_status,
        "joint_gaussian_laplace_plausible_values_not_mcmc"
    );
    assert!(!execution.artifact.to_json().expect("json").contains("rmse"));
    assert!(
        !execution
            .artifact
            .to_json()
            .expect("json")
            .contains("scientific_acceptance")
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
        Some(JOINT_POSTERIOR_DRAWS_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn execution_refuses_zero_draws_and_nonconvergence() {
    let request = request();
    assert_eq!(
        execute(&request, 0),
        Err(AnalysisEngineError::TopicMeasurement(
            TopicMeasurementError::InvalidModelInput
        ))
    );
    let exhausted = ReferenceTopicModelConfig::new(2, vec![1], 2, 1e-12).expect("exhausted");
    assert_eq!(
        execute_joint_posterior_draws_run(
            &request,
            &accepted(&request),
            "snapshot-joint-posterior-draws",
            cutoff(),
            &separated_input(),
            &exhausted,
            topic_ids(),
            4,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::TopicMeasurement(
            TopicMeasurementError::DidNotConverge
        ))
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    assert_eq!(
        execute_joint_posterior_draws_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &separated_input(),
            &recovery_config(),
            topic_ids(),
            4,
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
            value.model_contract_version = "trsl_tm_cpu_f64_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "trsl_topic_lineage_v1".into();
            value
        },
        {
            let mut value = request.clone();
            value.output_profile = "fitted_candidate_k_v1".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request, 4),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
