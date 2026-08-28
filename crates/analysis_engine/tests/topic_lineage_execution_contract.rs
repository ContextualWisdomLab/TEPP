//! End-to-end contract for the completed TRSL topic-lineage artifact.

use analysis_engine::{
    AnalysisEngineError, TOPIC_CONTEXT_POSTERIOR_OUTPUT_PROFILE,
    TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION, TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION,
    TOPIC_LINEAGE_MODEL_CONTRACT_VERSION, TOPIC_LINEAGE_OUTPUT_PROFILE, TopicActivityInterval,
    TopicContextMembership, TopicDocumentRelation, assemble_topic_context_posterior,
    execute_selected_topic_lineage_run, execute_topic_lineage_run,
};
use corpus_split::{CorpusDocument, CorpusSnapshot};
use membership_core::{
    GroupId, MemberId, MembershipAssignment, MembershipNetwork, MembershipRole, MembershipWeight,
};
use model_selection::FittedCandidateKConfig;
use relation_graph::{
    RelationEdge, RelationEndpointId, RelationEvidenceStatus, RelationGraph, RelationKind,
};
use temporal_core::{
    AvailableTime, EventTime, KnowledgeCutoff, TemporalBoundary, TemporalInterval,
    TemporalPrecision,
};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};
use topic_measurement::{
    ReferenceTopicInput, ReferenceTopicModelConfig, SparseMatrix, fit_reference_topic_model,
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

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "topic-lineage-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-topic-lineage".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: TOPIC_LINEAGE_MODEL_CONTRACT_VERSION.into(),
        output_profile: TOPIC_LINEAGE_OUTPUT_PROFILE.into(),
    }
}

fn context_request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        output_profile: TOPIC_CONTEXT_POSTERIOR_OUTPUT_PROFILE.into(),
        ..request()
    }
}

fn context_records(
    ids: &[Uuid],
    topic_ids: &[Uuid],
) -> (
    Vec<TopicActivityInterval>,
    Vec<TopicDocumentRelation>,
    Vec<TopicContextMembership>,
) {
    let activity = topic_ids
        .iter()
        .map(|topic_id| TopicActivityInterval {
            topic_id: topic_id.to_string(),
            state_code: "active".into(),
            valid_from: "2026-07-01T00:00:00Z".into(),
            valid_to: "2026-08-01T00:00:00Z".into(),
        })
        .collect();
    let relations = [(0, 1), (1, 2), (2, 3)]
        .into_iter()
        .enumerate()
        .map(|(index, (source, target))| TopicDocumentRelation {
            source_document_id: ids[source].to_string(),
            target_document_id: ids[target].to_string(),
            relation_kind_code: "event_lineage_precedes".into(),
            event_time: event_time(u8::try_from(target + 1).expect("day")).to_rfc3339(),
            evidence_sha256: format!("{:064x}", index + 1),
            evidence_resource_id: format!("relation-evidence-{index}"),
            provenance_assertion_id: format!("relation-provenance-{index}"),
        })
        .collect();
    let memberships = ids
        .iter()
        .flat_map(|document_id| {
            ["business_unit", "process_unit", "team", "person"]
                .into_iter()
                .map(move |dimension| TopicContextMembership {
                    document_id: document_id.to_string(),
                    dimension_code: dimension.into(),
                    context_id: format!("{dimension}-{document_id}"),
                    weight: 1.0,
                    valid_from: "2026-07-01T00:00:00Z".into(),
                    valid_to: "2026-08-01T00:00:00Z".into(),
                    evidence_sha256: "a".repeat(64),
                    evidence_resource_id: format!("{dimension}-evidence-{document_id}"),
                    provenance_assertion_id: format!("{dimension}-provenance-{document_id}"),
                })
        })
        .collect();
    (activity, relations, memberships)
}

#[test]
fn fitted_topics_emit_digest_bound_predecessor_successor_counts() {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let counts = SparseMatrix::from_csr(
        4,
        4,
        vec![0, 2, 4, 6, 8],
        vec![0, 1, 0, 1, 2, 3, 2, 3],
        vec![90.0, 10.0, 85.0, 15.0, 10.0, 90.0, 15.0, 85.0],
    )
    .expect("counts");
    let input = ReferenceTopicInput::new_bound(
        &snapshot,
        "snapshot-topic-lineage",
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        ids.clone(),
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("input");
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let request = request();
    let accepted =
        AnalysisRunAccepted::new("run-topic-lineage", "accepted", &request.idempotency_key)
            .expect("accepted");
    let execution = execute_topic_lineage_run(
        &request,
        &accepted,
        "snapshot-topic-lineage",
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        &input,
        &config,
        "2026-08-02T00:00:00Z",
    )
    .expect("execution");

    assert_eq!(
        execution.artifact.schema_version,
        TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.connected_post_count, 4);
    assert_eq!(execution.artifact.lineage_count, 2);
    assert_eq!(execution.artifact.topic_count, 2);
    assert_eq!(execution.artifact.sequence_edges.len(), 2);
    assert_eq!(execution.artifact.source_snapshot_sha256.len(), 64);
    assert_eq!(execution.artifact.model_input_sha256.len(), 64);
    assert_eq!(
        execution.artifact.model_contract_version,
        TOPIC_LINEAGE_MODEL_CONTRACT_VERSION
    );
    assert_eq!(
        execution.artifact.fit_manifest.method_code,
        "fixed_k_reference_v1"
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
        Some(TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION)
    );
    assert!(execution.artifact.to_json().is_ok());

    let selection = FittedCandidateKConfig::new(vec![5, 2], vec![7, 11], 2_000, 1e-5)
        .expect("selection config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("selection hyperparameters");
    let selected = execute_selected_topic_lineage_run(
        &request,
        &accepted,
        "snapshot-topic-lineage",
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        &input,
        &selection,
        "trsl_tm_reference",
        &[3],
        "2026-08-02T00:00:00Z",
    )
    .expect("selected execution");
    assert_eq!(selected.artifact.topic_count, 2);
    assert_eq!(
        selected.artifact.fit_manifest.method_code,
        "trsl_tm_reference_bic_v1"
    );
    assert_eq!(selected.artifact.fit_manifest.candidate_outcomes.len(), 2);
    assert_eq!(selected.artifact.fit_manifest.llm_recommendations, vec![3]);
}

#[test]
#[allow(clippy::too_many_lines)]
fn complete_topic_context_artifact_retains_event_time_branches_and_draws() {
    let (snapshot, ids, times, source_memberships, source_relations) = fixture();
    let counts = SparseMatrix::from_csr(
        4,
        4,
        vec![0, 2, 4, 6, 8],
        vec![0, 1, 0, 1, 2, 3, 2, 3],
        vec![90.0, 10.0, 85.0, 15.0, 10.0, 90.0, 15.0, 85.0],
    )
    .expect("counts");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");
    let input = ReferenceTopicInput::new_bound(
        &snapshot,
        "snapshot-topic-lineage",
        cutoff,
        ids.clone(),
        &counts,
        &times,
        None,
        &source_memberships,
        &source_relations,
    )
    .expect("input");
    let config = ReferenceTopicModelConfig::new(2, vec![7, 11], 2_000, 1e-5)
        .expect("config")
        .with_hyperparameters(1.0, 0.5, 0.01, 0.05, 0.2)
        .expect("hyperparameters");
    let model = fit_reference_topic_model(&input, &config).expect("fit");
    let topic_ids = vec![Uuid::from_u128(101), Uuid::from_u128(102)];
    let (activity, relations, memberships) = context_records(&ids, &topic_ids);
    let request = context_request();
    let accepted =
        AnalysisRunAccepted::new("run-topic-context", "accepted", &request.idempotency_key)
            .expect("accepted");
    assert_eq!(
        assemble_topic_context_posterior(
            &request,
            &accepted,
            "foreign-snapshot",
            cutoff,
            &input,
            &model,
            &config,
            topic_ids.clone(),
            activity.clone(),
            vec![],
            relations.clone(),
            memberships.clone(),
            19,
            3,
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let invalid_profile_request = AnalysisRunRequest {
        output_profile: TOPIC_LINEAGE_OUTPUT_PROFILE.into(),
        ..context_request()
    };
    let invalid_profile_accepted = AnalysisRunAccepted::new(
        "run-topic-context-invalid-profile",
        "accepted",
        &invalid_profile_request.idempotency_key,
    )
    .expect("accepted invalid profile request");
    assert_eq!(
        assemble_topic_context_posterior(
            &invalid_profile_request,
            &invalid_profile_accepted,
            "snapshot-topic-lineage",
            cutoff,
            &input,
            &model,
            &config,
            topic_ids.clone(),
            activity.clone(),
            vec![],
            relations.clone(),
            memberships.clone(),
            19,
            3,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let assert_invalid_binding =
        |invalid_request: &AnalysisRunRequest,
         invalid_accepted: &AnalysisRunAccepted,
         invalid_snapshot: &str,
         invalid_cutoff: KnowledgeCutoff| {
            assert_eq!(
                assemble_topic_context_posterior(
                    invalid_request,
                    invalid_accepted,
                    invalid_snapshot,
                    invalid_cutoff,
                    &input,
                    &model,
                    &config,
                    topic_ids.clone(),
                    activity.clone(),
                    vec![],
                    relations.clone(),
                    memberships.clone(),
                    19,
                    3,
                ),
                Err(AnalysisEngineError::InvalidEvidence)
            );
        };
    let wrong_cutoff_request = AnalysisRunRequest {
        knowledge_cutoff: "2026-07-31T00:00:00Z".into(),
        ..context_request()
    };
    let wrong_cutoff_accepted = AnalysisRunAccepted::new(
        "run-topic-context-wrong-cutoff",
        "accepted",
        &wrong_cutoff_request.idempotency_key,
    )
    .expect("accepted wrong-cutoff request");
    assert_invalid_binding(
        &wrong_cutoff_request,
        &wrong_cutoff_accepted,
        "snapshot-topic-lineage",
        cutoff,
    );
    let wrong_model_request = AnalysisRunRequest {
        model_contract_version: "foreign-model-v1".into(),
        ..context_request()
    };
    let wrong_model_accepted = AnalysisRunAccepted::new(
        "run-topic-context-wrong-model",
        "accepted",
        &wrong_model_request.idempotency_key,
    )
    .expect("accepted wrong-model request");
    assert_invalid_binding(
        &wrong_model_request,
        &wrong_model_accepted,
        "snapshot-topic-lineage",
        cutoff,
    );
    let foreign_snapshot_request = AnalysisRunRequest {
        snapshot_id: "foreign-snapshot".into(),
        ..context_request()
    };
    let foreign_snapshot_accepted = AnalysisRunAccepted::new(
        "run-topic-context-foreign-snapshot",
        "accepted",
        &foreign_snapshot_request.idempotency_key,
    )
    .expect("accepted foreign-snapshot request");
    assert_invalid_binding(
        &foreign_snapshot_request,
        &foreign_snapshot_accepted,
        "foreign-snapshot",
        cutoff,
    );
    let foreign_cutoff =
        KnowledgeCutoff::parse_rfc3339("2026-07-31T00:00:00Z").expect("foreign cutoff");
    let foreign_cutoff_request = AnalysisRunRequest {
        knowledge_cutoff: foreign_cutoff.to_rfc3339(),
        ..context_request()
    };
    let foreign_cutoff_accepted = AnalysisRunAccepted::new(
        "run-topic-context-foreign-cutoff",
        "accepted",
        &foreign_cutoff_request.idempotency_key,
    )
    .expect("accepted foreign-cutoff request");
    assert_invalid_binding(
        &foreign_cutoff_request,
        &foreign_cutoff_accepted,
        "snapshot-topic-lineage",
        foreign_cutoff,
    );
    let mut divergent_relations = relations.clone();
    divergent_relations[1].source_document_id = ids[0].to_string();
    assert_eq!(
        assemble_topic_context_posterior(
            &request,
            &accepted,
            "snapshot-topic-lineage",
            cutoff,
            &input,
            &model,
            &config,
            topic_ids.clone(),
            activity.clone(),
            vec![],
            divergent_relations,
            memberships.clone(),
            19,
            3,
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let artifact = assemble_topic_context_posterior(
        &request,
        &accepted,
        "snapshot-topic-lineage",
        cutoff,
        &input,
        &model,
        &config,
        topic_ids,
        activity,
        vec![],
        relations,
        memberships,
        19,
        3,
    )
    .expect("artifact");

    assert_eq!(
        artifact.schema_version,
        TOPIC_CONTEXT_POSTERIOR_SCHEMA_VERSION
    );
    assert_eq!(artifact.posterior_draw_count, 3);
    assert_eq!(artifact.plausible_values.len(), 12);
    assert_eq!(artifact.document_relations.len(), 3);
    assert_eq!(
        artifact
            .document_relations
            .iter()
            .filter(|relation| relation.source_document_id == ids[0].to_string())
            .count(),
        1
    );
    assert_eq!(artifact.posterior_draw_set_id.len(), 64);
    let json = artifact.to_json().expect("json");
    assert_eq!(
        analysis_engine::TopicContextPosteriorArtifact::from_json(&json)
            .expect("round trip")
            .to_json()
            .expect("canonical round trip"),
        json
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn execution_refuses_binding_and_nonconvergence_without_an_artifact() {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let counts = SparseMatrix::from_csr(4, 2, vec![0, 1, 2, 3, 4], vec![0, 0, 1, 1], vec![1.0; 4])
        .expect("counts");
    let unbound_input = ReferenceTopicInput::new(
        &snapshot,
        ids.clone(),
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("input");
    let input = ReferenceTopicInput::new_bound(
        &snapshot,
        "snapshot-topic-lineage",
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        ids.clone(),
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("bound input");
    let request = request();
    let accepted =
        AnalysisRunAccepted::new("run-topic-lineage", "accepted", &request.idempotency_key)
            .expect("accepted");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");
    let config = ReferenceTopicModelConfig::new(2, vec![1], 2, 1e-12).expect("config");

    assert_eq!(
        execute_topic_lineage_run(
            &request,
            &accepted,
            "snapshot-topic-lineage",
            cutoff,
            &unbound_input,
            &config,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );

    assert_eq!(
        execute_topic_lineage_run(
            &request,
            &accepted,
            "other-snapshot",
            cutoff,
            &input,
            &config,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let other_snapshot_input = ReferenceTopicInput::new_bound(
        &snapshot,
        "other-binding",
        cutoff,
        ids.clone(),
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("other snapshot binding");
    assert_eq!(
        execute_topic_lineage_run(
            &request,
            &accepted,
            "snapshot-topic-lineage",
            cutoff,
            &other_snapshot_input,
            &config,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let other_cutoff_input = ReferenceTopicInput::new_bound(
        &snapshot,
        "snapshot-topic-lineage",
        KnowledgeCutoff::parse_rfc3339("2026-08-02T00:00:00Z").expect("other cutoff"),
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("other cutoff binding");
    assert_eq!(
        execute_topic_lineage_run(
            &request,
            &accepted,
            "snapshot-topic-lineage",
            cutoff,
            &other_cutoff_input,
            &config,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
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
            execute_topic_lineage_run(
                &invalid_request,
                &accepted,
                "snapshot-topic-lineage",
                cutoff,
                &input,
                &config,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    assert_eq!(
        execute_topic_lineage_run(
            &request,
            &accepted,
            "snapshot-topic-lineage",
            cutoff,
            &input,
            &config,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::TopicMeasurement(
            topic_measurement::TopicMeasurementError::DidNotConverge
        ))
    );

    let failed_selection =
        FittedCandidateKConfig::new(vec![5], vec![1], 10, 1e-6).expect("selection config");
    let error = execute_selected_topic_lineage_run(
        &request,
        &accepted,
        "snapshot-topic-lineage",
        cutoff,
        &input,
        &failed_selection,
        "trsl_tm_reference",
        &[],
        "2026-08-02T00:00:00Z",
    )
    .expect_err("failed fitted selection");
    assert_eq!(
        error.to_string(),
        "no fitted candidate produced a finite diagnostic"
    );
    let AnalysisEngineError::FittedModelSelection(receipt) = error else {
        panic!("expected reason-bearing fitted selection failure");
    };
    assert_eq!(receipt.candidate_outcomes().len(), 1);
    assert_eq!(
        receipt.candidate_outcomes()[0].failure(),
        Some(topic_measurement::TopicMeasurementError::InvalidModelInput)
    );
}
