//! End-to-end contract for cutoff-safe fitted candidate-`K` selection.

use analysis_engine::{
    AnalysisEngineError, FITTED_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION,
    FITTED_CANDIDATE_K_MODEL_CONTRACT_VERSION, FITTED_CANDIDATE_K_OUTPUT_PROFILE,
    execute_fitted_candidate_k_run,
};
use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::{FittedCandidateKConfig, ModelSelectionError};
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};
use topic_measurement::{ReferenceTopicInput, SparseMatrix};
use uuid::Uuid;

fn event_time(day: u8) -> EventTime {
    EventTime::parse_rfc3339(&format!("2026-01-{day:02}T00:00:00Z")).expect("event time")
}

fn relation(source: Uuid, target: Uuid, source_day: u8, target_day: u8) -> RelationEdge {
    let interval = |day| {
        TemporalInterval::bounded(
            TemporalBoundary::Included(event_time(day)),
            TemporalBoundary::Included(
                EventTime::parse_rfc3339(&format!("2026-01-{day:02}T12:00:00Z"))
                    .expect("interval end"),
            ),
            TemporalPrecision::Second,
        )
        .expect("bounded interval")
    };
    RelationEdge::new(
        RelationKind::TransitionsTo,
        RelationEndpointId::from_uuid(source),
        RelationEndpointId::from_uuid(target),
        RelationEvidenceStatus::Observed,
        interval(source_day),
        interval(target_day),
    )
    .expect("forward relation")
}

fn separated_topic_input() -> ReferenceTopicInput {
    let document_ids: Vec<_> = (1_u128..=6).map(Uuid::from_u128).collect();
    let times: Vec<_> = (1_u8..=6).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339("2026-01-10T00:00:00Z").expect("available");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff");
    let mut snapshot = CorpusSnapshot::new();
    for id in &document_ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &cutoff)
            .expect("eligible");
    }

    let organization = GroupId::from_uuid(Uuid::from_u128(100));
    let projects = [
        GroupId::from_uuid(Uuid::from_u128(101)),
        GroupId::from_uuid(Uuid::from_u128(102)),
    ];
    let validity_start = event_time(1);
    let validity_end = event_time(9);
    let mut memberships = MembershipNetwork::new();
    for (index, id) in document_ids.iter().enumerate() {
        let member = MemberId::from_uuid(*id);
        memberships
            .insert(
                MembershipAssignment::new(
                    member,
                    organization,
                    MembershipRole::Organization,
                    MembershipWeight::full().expect("full"),
                    validity_start,
                    validity_end,
                )
                .expect("organization membership"),
            )
            .expect("insert organization");
        memberships
            .insert(
                MembershipAssignment::new(
                    member,
                    projects[usize::from(index >= 3)],
                    MembershipRole::Project,
                    MembershipWeight::new(0.75).expect("partial"),
                    validity_start,
                    validity_end,
                )
                .expect("project membership"),
            )
            .expect("insert project");
    }

    let mut relations = RelationGraph::new();
    for (source, target, source_day, target_day) in [
        (0, 1, 1, 2),
        (1, 2, 2, 3),
        (2, 3, 3, 4),
        (3, 4, 4, 5),
        (4, 5, 5, 6),
    ] {
        relations
            .insert(relation(
                document_ids[source],
                document_ids[target],
                source_day,
                target_day,
            ))
            .expect("insert relation");
    }

    let counts = SparseMatrix::from_csr(
        6,
        4,
        vec![0, 2, 4, 6, 8, 10, 12],
        vec![0, 1, 0, 1, 0, 1, 2, 3, 2, 3, 2, 3],
        vec![
            90.0, 10.0, 85.0, 15.0, 80.0, 20.0, 10.0, 90.0, 15.0, 85.0, 20.0, 80.0,
        ],
    )
    .expect("counts");
    ReferenceTopicInput::new(
        &snapshot,
        document_ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("validated input")
}

fn recovery_config() -> FittedCandidateKConfig {
    FittedCandidateKConfig::new(vec![2, 3], vec![7, 11, 19], 2_000, 1e-5)
        .expect("candidate configuration")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-02-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "fitted-candidate-k-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-fitted-candidate-k".into(),
        knowledge_cutoff: "2026-02-01T00:00:00Z".into(),
        model_contract_version: FITTED_CANDIDATE_K_MODEL_CONTRACT_VERSION.into(),
        output_profile: FITTED_CANDIDATE_K_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-fitted-candidate-k",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn execute(
    request: &AnalysisRunRequest,
    method_name: &str,
    llm_votes: &[u32],
) -> Result<analysis_engine::FittedCandidateKExecution, AnalysisEngineError> {
    execute_fitted_candidate_k_run(
        request,
        &accepted(request),
        "snapshot-fitted-candidate-k",
        cutoff(),
        &separated_topic_input(),
        &recovery_config(),
        method_name,
        llm_votes,
        "2026-02-02T00:00:00Z",
    )
}

#[test]
fn separated_topics_select_true_k_and_refuse_llm_vote_as_authority() {
    let request = request();
    let execution = execute(&request, "trsl_tm_reference", &[3]).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        FITTED_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.selected_k, 2);
    assert_eq!(execution.artifact.candidate_count, 2);
    assert_eq!(execution.artifact.evidence_count, 6);
    assert_eq!(execution.artifact.method_name, "trsl_tm_reference");
    assert_eq!(
        execution.artifact.inference_status,
        "fitted_schwarz_candidate_k_not_bayesian_sampler"
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
        Some(FITTED_CANDIDATE_K_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn execution_refuses_lexical_methods_and_failed_fits() {
    let request = request();
    assert_eq!(
        execute(&request, "tf-idf", &[]),
        Err(AnalysisEngineError::ModelSelection(
            ModelSelectionError::LexicalWeightForbidden
        ))
    );
    let exhausted = FittedCandidateKConfig::new(vec![2], vec![1], 2, 1e-12).expect("exhausted");
    assert_eq!(
        execute_fitted_candidate_k_run(
            &request,
            &accepted(&request),
            "snapshot-fitted-candidate-k",
            cutoff(),
            &separated_topic_input(),
            &exhausted,
            "trsl_tm_reference",
            &[],
            "2026-02-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::ModelSelection(
            ModelSelectionError::NoSuccessfulFit
        ))
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    assert_eq!(
        execute_fitted_candidate_k_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &separated_topic_input(),
            &recovery_config(),
            "trsl_tm_reference",
            &[],
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
            value.output_profile = "trsl_topic_lineage_v1".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request, "trsl_tm_reference", &[]),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
