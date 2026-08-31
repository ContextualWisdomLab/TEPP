//! End-to-end contract for fitted candidate-`K` composed with topic lineage.

use analysis_engine::{
    AnalysisEngineError, COMPOSED_FITTED_LINEAGE_ARTIFACT_SCHEMA_VERSION,
    COMPOSED_FITTED_LINEAGE_MODEL_CONTRACT_VERSION, COMPOSED_FITTED_LINEAGE_OUTPUT_PROFILE,
    ComposedFittedLineageInput, TOPIC_LINEAGE_MODEL_CONTRACT_VERSION, TOPIC_LINEAGE_OUTPUT_PROFILE,
    execute_composed_fitted_lineage_run, execute_topic_lineage_run,
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
    EventTime::parse_rfc3339(&format!("2026-07-{day:02}T00:00:00Z")).expect("event time")
}

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn fixture() -> ReferenceTopicInput {
    fixture_available_at("2026-07-01T00:00:00Z", cutoff())
}

fn fixture_available_at(
    available_at: &str,
    admission_cutoff: KnowledgeCutoff,
) -> ReferenceTopicInput {
    let ids: Vec<_> = (1_u128..=4).map(Uuid::from_u128).collect();
    let times: Vec<_> = (1_u8..=4).map(event_time).collect();
    let available = AvailableTime::parse_rfc3339(available_at).expect("available");
    let mut snapshot = CorpusSnapshot::new();
    let mut memberships = MembershipNetwork::new();
    for id in &ids {
        snapshot
            .insert_if_eligible(CorpusDocument::new(*id, available), &admission_cutoff)
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

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "composed-fitted-lineage-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-composed-fitted-lineage".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: COMPOSED_FITTED_LINEAGE_MODEL_CONTRACT_VERSION.into(),
        output_profile: COMPOSED_FITTED_LINEAGE_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-composed-fitted-lineage",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn selection() -> FittedCandidateKConfig {
    FittedCandidateKConfig::new(vec![2, 3], vec![7, 11], 2_000, 1e-5).expect("selection")
}

fn composition<'a>(
    input: &'a ReferenceTopicInput,
    selection: &'a FittedCandidateKConfig,
    method_name: &'a str,
    llm_votes: &'a [u32],
) -> ComposedFittedLineageInput<'a> {
    ComposedFittedLineageInput::new(input, selection, method_name, llm_votes)
}

fn execute(
    request: &AnalysisRunRequest,
) -> Result<analysis_engine::ComposedFittedLineageExecution, AnalysisEngineError> {
    let input = fixture();
    let selection = selection();
    execute_composed_fitted_lineage_run(
        request,
        &accepted(request),
        "snapshot-composed-fitted-lineage",
        cutoff(),
        &composition(&input, &selection, "trsl_tm_reference", &[3]),
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn fitted_selection_then_lineage_emits_digest_bound_composition() {
    let request = request();
    let execution = execute(&request).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        COMPOSED_FITTED_LINEAGE_ARTIFACT_SCHEMA_VERSION
    );
    assert!(execution.artifact.selected_k >= 2);
    assert_eq!(
        execution.artifact.lineage_topic_count,
        execution.artifact.selected_k
    );
    assert_eq!(execution.artifact.candidate_count, 2);
    assert_eq!(execution.artifact.evidence_count, 4);
    assert!(execution.artifact.lineage_edge_count >= 1);
    assert_eq!(execution.artifact.lineage_artifact_sha256.len(), 64);
    assert_eq!(
        execution.artifact.inference_status,
        "fitted_k_composed_lineage_not_bayesian_sampler"
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
        Some(COMPOSED_FITTED_LINEAGE_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn selected_fit_preserves_hyperparameters_and_cutoff_provenance() {
    let request = request();
    let accepted = accepted(&request);
    let input = fixture();
    let selection = selection()
        .with_hyperparameters(1.7, 0.4, 0.03, 0.08, 0.12)
        .expect("non-default selection");
    let execution = execute_composed_fitted_lineage_run(
        &request,
        &accepted,
        "snapshot-composed-fitted-lineage",
        cutoff(),
        &composition(&input, &selection, "trsl_tm_reference", &[]),
        "2026-08-02T00:00:00Z",
    )
    .expect("composition");

    let selected_k = u32::try_from(execution.artifact.selected_k).expect("selected K");
    let mut lineage_request = request.clone();
    lineage_request.model_contract_version = TOPIC_LINEAGE_MODEL_CONTRACT_VERSION.into();
    lineage_request.output_profile = TOPIC_LINEAGE_OUTPUT_PROFILE.into();
    let direct = execute_topic_lineage_run(
        &lineage_request,
        &accepted,
        "snapshot-composed-fitted-lineage",
        cutoff(),
        &input,
        &selection
            .reference_config(selected_k)
            .expect("exact config"),
        "2026-08-02T00:00:00Z",
    )
    .expect("direct lineage");
    assert_eq!(
        execution.artifact.lineage_artifact_sha256,
        direct.artifact.sha256().expect("lineage digest")
    );

    let late_input = fixture_available_at(
        "2026-08-02T00:00:00Z",
        KnowledgeCutoff::parse_rfc3339("2026-08-03T00:00:00Z").expect("later cutoff"),
    );
    assert_eq!(
        execute_composed_fitted_lineage_run(
            &request,
            &accepted,
            "snapshot-composed-fitted-lineage",
            cutoff(),
            &composition(&late_input, &selection, "trsl_tm_reference", &[]),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn lexical_method_and_empty_candidates_fail_closed() {
    let request = request();
    let input = fixture();
    let selection = selection();
    assert_eq!(
        execute_composed_fitted_lineage_run(
            &request,
            &accepted(&request),
            "snapshot-composed-fitted-lineage",
            cutoff(),
            &composition(&input, &selection, "tfidf", &[]),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::ModelSelection(
            ModelSelectionError::LexicalWeightForbidden
        ))
    );
    assert_eq!(
        execute_composed_fitted_lineage_run(
            &request,
            &accepted(&request),
            "snapshot-composed-fitted-lineage",
            cutoff(),
            &composition(&input, &selection, "", &[]),
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let input = fixture();
    let selection = selection();
    assert_eq!(
        execute_composed_fitted_lineage_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &composition(&input, &selection, "trsl_tm_reference", &[]),
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
            value.output_profile = "joint_posterior_draws_v1".into();
            value
        },
    ] {
        assert_eq!(
            execute(&invalid_request),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
