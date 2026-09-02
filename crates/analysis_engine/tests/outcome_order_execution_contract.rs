//! End-to-end contract for cutoff-safe input-process-outcome order refusals.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION,
    OUTCOME_ORDER_MODEL_CONTRACT_VERSION, OUTCOME_ORDER_OUTPUT_PROFILE, OutcomeOrderArtifact,
    OutcomeOrderEdge, execute_outcome_order_run,
};
use outcome_order::OutcomeKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "outcome-order-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-outcome-order".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: OUTCOME_ORDER_MODEL_CONTRACT_VERSION.into(),
        output_profile: OUTCOME_ORDER_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-outcome-order", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn edge(
    edge_id: &str,
    kind: OutcomeKind,
    source_rank: u64,
    target_rank: u64,
    stamp: &str,
) -> OutcomeOrderEdge {
    OutcomeOrderEdge::new(edge_id, kind, source_rank, target_rank, available(stamp)).expect("edge")
}

fn mixed_edges() -> Vec<OutcomeOrderEdge> {
    vec![
        edge(
            "input-a",
            OutcomeKind::InputTo,
            1,
            2,
            "2026-07-01T00:00:00Z",
        ),
        edge(
            "process-b",
            OutcomeKind::ProcessTo,
            2,
            3,
            "2026-07-02T00:00:00Z",
        ),
        edge(
            "outcome-c",
            OutcomeKind::OutcomeOf,
            9,
            1,
            "2026-07-03T00:00:00Z",
        ),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    edges: &[OutcomeOrderEdge],
) -> Result<analysis_engine::OutcomeOrderExecution, AnalysisEngineError> {
    execute_outcome_order_run(
        request,
        &accepted(request),
        "snapshot-outcome-order",
        cutoff(),
        edges,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_ipo_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_edges()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.edge_count, 3);
    assert_eq!(execution.artifact.input_to_count, 1);
    assert_eq!(execution.artifact.process_to_count, 1);
    assert_eq!(execution.artifact.outcome_of_count, 1);
    assert_eq!(execution.artifact.refused_as_transition_count, 1);
    assert_eq!(
        execution.artifact.inference_status,
        "input_process_forward_outcome_of_is_not_transition"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("kind_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
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
        Some(OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn compact_oversized_artifact_counts_fail_closed() {
    let edge_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let outcome_of_count = edge_count - 2;
    let artifact = OutcomeOrderArtifact {
        schema_version: OUTCOME_ORDER_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        edge_count,
        input_to_count: 1,
        process_to_count: 1,
        outcome_of_count,
        refused_as_transition_count: outcome_of_count,
        inference_status: "input_process_forward_outcome_of_is_not_transition".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");
    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidOutcomeOrderArtifact)
    );
    assert_eq!(
        OutcomeOrderArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidOutcomeOrderArtifact)
    );
}

#[test]
fn future_available_edges_are_excluded() {
    let request = request();
    let mut with_future = mixed_edges();
    with_future.push(edge(
        "future-d",
        OutcomeKind::InputTo,
        4,
        5,
        "2026-08-02T00:00:00Z",
    ));
    let execution = execute(&request, &with_future).expect("cutoff");
    assert_eq!(execution.artifact.edge_count, 3);
    assert_eq!(execution.artifact.input_to_count, 1);
}

#[test]
fn empty_or_single_class_reverse_uncertain_and_duplicate_fail_closed() {
    let request = request();
    let stamp = "2026-07-01T00:00:00Z";
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let input_only = vec![
        edge("input-a", OutcomeKind::InputTo, 1, 2, stamp),
        edge("input-b", OutcomeKind::InputTo, 3, 4, stamp),
        edge("input-c", OutcomeKind::InputTo, 5, 6, stamp),
    ];
    assert_eq!(
        execute(&request, &input_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let transitions_only = vec![
        edge("input-a", OutcomeKind::InputTo, 1, 2, stamp),
        edge("process-b", OutcomeKind::ProcessTo, 2, 3, stamp),
    ];
    assert_eq!(
        execute(&request, &transitions_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let provenance_only = vec![
        edge("outcome-a", OutcomeKind::OutcomeOf, 9, 1, stamp),
        edge("outcome-b", OutcomeKind::OutcomeOf, 8, 2, stamp),
        edge("outcome-c", OutcomeKind::OutcomeOf, 7, 3, stamp),
    ];
    assert_eq!(
        execute(&request, &provenance_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let reverse = vec![
        edge("input-a", OutcomeKind::InputTo, 4, 1, stamp),
        edge("process-b", OutcomeKind::ProcessTo, 2, 3, stamp),
        edge("outcome-c", OutcomeKind::OutcomeOf, 9, 1, stamp),
    ];
    assert_eq!(
        execute(&request, &reverse),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let contemporaneous = vec![
        edge("input-a", OutcomeKind::InputTo, 2, 2, stamp),
        edge("process-b", OutcomeKind::ProcessTo, 2, 3, stamp),
        edge("outcome-c", OutcomeKind::OutcomeOf, 9, 1, stamp),
    ];
    assert_eq!(
        execute(&request, &contemporaneous),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        edge("same", OutcomeKind::InputTo, 1, 2, stamp),
        edge("same", OutcomeKind::OutcomeOf, 9, 1, stamp),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        OutcomeOrderEdge::new("", OutcomeKind::InputTo, 1, 2, available(stamp)),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_cutoff_mismatch_and_oversize() {
    let request = request();
    let edges = mixed_edges();
    assert_eq!(
        execute_outcome_order_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &edges,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let mut mismatched = request.clone();
    mismatched.knowledge_cutoff = "2026-07-01T00:00:00Z".into();
    assert_eq!(
        execute_outcome_order_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-outcome-order",
            cutoff(),
            &edges,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    for profile in [
        "trsl_topic_lineage_v1",
        "fitted_candidate_k_v1",
        "pareto_candidate_k_v1",
        "joint_posterior_draws_v1",
        "method_effects_v1",
        "copy_identity_v1",
        "style_source_v1",
        "prompt_source_v1",
        "modality_source_v1",
        "corpus_background_v1",
        "citation_edge_v1",
        "copied_text_v1",
        "lineage_criterion_v1",
        "composed_fitted_lineage_v1",
        "case_deletion_refit_v1",
        "topic_activity_v1",
        "location_membership_v1",
        "topic_context_posterior_v1",
        "membership_posterior_icc_v1",
        "membership_target_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_outcome_order_run(
                &reused,
                &accepted(&reused),
                "snapshot-outcome-order",
                cutoff(),
                &edges,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    let oversized: Vec<OutcomeOrderEdge> = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            edge(
                &format!("edge-{index}"),
                OutcomeKind::InputTo,
                u64::try_from(index).expect("index"),
                u64::try_from(index).expect("index") + 1,
                "2026-07-01T00:00:00Z",
            )
        })
        .collect();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
