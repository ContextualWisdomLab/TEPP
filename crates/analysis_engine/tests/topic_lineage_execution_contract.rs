//! End-to-end contract for the completed TRSL topic-lineage artifact.

use analysis_engine::{
    AnalysisEngineError, TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION,
    TOPIC_LINEAGE_MODEL_CONTRACT_VERSION, TOPIC_LINEAGE_OUTPUT_PROFILE, execute_topic_lineage_run,
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
use topic_measurement::{ReferenceTopicInput, SparseMatrix};
use uuid::Uuid;

const CONFIG_JSON: &str = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[7,11],\"maximum_iterations\":2000,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.5,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";

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
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("input");
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
        CONFIG_JSON,
        "2026-08-02T00:00:00Z",
    )
    .expect("execution");

    assert_eq!(
        execution.artifact.schema_version,
        TOPIC_LINEAGE_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.model_contract_version, TOPIC_LINEAGE_MODEL_CONTRACT_VERSION);
    assert_eq!(execution.artifact.method_configuration_json, CONFIG_JSON);
    assert_eq!(execution.artifact.estimator_backend, "cpu_f64_reference");
    assert_eq!(execution.artifact.posterior_approximation, "diagonal_laplace");
    assert_eq!(execution.artifact.connected_post_count, 4);
    assert_eq!(execution.artifact.lineage_count, 2);
    assert_eq!(execution.artifact.sequence_edges.len(), 2);
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
}

#[test]
fn execution_refuses_binding_and_nonconvergence_without_an_artifact() {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let counts = SparseMatrix::from_csr(4, 2, vec![0, 1, 2, 3, 4], vec![0, 0, 1, 1], vec![1.0; 4])
        .expect("counts");
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("input");
    let request = request();
    let accepted =
        AnalysisRunAccepted::new("run-topic-lineage", "accepted", &request.idempotency_key)
            .expect("accepted");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");
    let nonconvergent_config = "{\"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[1],\"maximum_iterations\":2,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.25,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}";

    assert_eq!(
        execute_topic_lineage_run(
            &request,
            &accepted,
            "other-snapshot",
            cutoff,
            &input,
            nonconvergent_config,
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
            execute_topic_lineage_run(
                &invalid_request,
                &accepted,
                "snapshot-topic-lineage",
                cutoff,
                &input,
                nonconvergent_config,
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
            nonconvergent_config,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::TopicMeasurement(
            topic_measurement::TopicMeasurementError::DidNotConverge
        ))
    );
}

#[test]
fn malformed_or_noncanonical_configuration_fails_before_estimation() {
    let (snapshot, ids, times, memberships, relations) = fixture();
    let counts = SparseMatrix::from_csr(4, 2, vec![0, 1, 2, 3, 4], vec![0, 0, 1, 1], vec![1.0; 4])
        .expect("counts");
    let input = ReferenceTopicInput::new(
        &snapshot,
        ids,
        &counts,
        &times,
        None,
        &memberships,
        &relations,
    )
    .expect("input");
    let request = request();
    let accepted =
        AnalysisRunAccepted::new("run-topic-lineage", "accepted", &request.idempotency_key)
            .expect("accepted");
    let cutoff = KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff");

    for invalid_config in [
        "{}",
        "{\"configuration_schema_version\":\"floating\",\"topic_count\":2,\"seeds\":[1],\"maximum_iterations\":2,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.25,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}",
        "{ \"configuration_schema_version\":\"tepp.trsl_topic_lineage.reference_config.v1\",\"topic_count\":2,\"seeds\":[1],\"maximum_iterations\":2,\"tolerance\":0.001,\"prior_variance\":1.0,\"relation_strength\":0.25,\"ridge\":0.01,\"topic_smoothing\":0.05,\"step_size\":0.2}",
    ] {
        assert_eq!(
            execute_topic_lineage_run(
                &request,
                &accepted,
                "snapshot-topic-lineage",
                cutoff,
                &input,
                invalid_config,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
