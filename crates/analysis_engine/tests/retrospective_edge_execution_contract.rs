//! End-to-end contract for cutoff-safe retrospective-edge refusals.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION,
    RETROSPECTIVE_EDGE_MODEL_CONTRACT_VERSION, RETROSPECTIVE_EDGE_OUTPUT_PROFILE,
    RetrospectiveEdgeArtifact, RetrospectiveEdgeAssignment, execute_retrospective_edge_run,
};
use retrospective_edge::RetrospectiveKind;
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
        idempotency_key: "retrospective-edge-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-retrospective-edge".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: RETROSPECTIVE_EDGE_MODEL_CONTRACT_VERSION.into(),
        output_profile: RETROSPECTIVE_EDGE_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-retrospective-edge",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn assignment(
    assignment_id: &str,
    kind: RetrospectiveKind,
    stamp: &str,
) -> RetrospectiveEdgeAssignment {
    RetrospectiveEdgeAssignment::new(assignment_id, kind, available(stamp)).expect("assignment")
}

fn mixed_assignments() -> Vec<RetrospectiveEdgeAssignment> {
    vec![
        assignment(
            "retrospective-a",
            RetrospectiveKind::RetrospectiveReport,
            "2026-07-01T00:00:00Z",
        ),
        assignment(
            "forward-b",
            RetrospectiveKind::ForwardReport,
            "2026-07-02T00:00:00Z",
        ),
        assignment(
            "retrospective-c",
            RetrospectiveKind::RetrospectiveReport,
            "2026-07-03T00:00:00Z",
        ),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    assignments: &[RetrospectiveEdgeAssignment],
) -> Result<analysis_engine::RetrospectiveEdgeExecution, AnalysisEngineError> {
    execute_retrospective_edge_run(
        request,
        &accepted(request),
        "snapshot-retrospective-edge",
        cutoff(),
        assignments,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_assignments()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.assignment_count, 3);
    assert_eq!(execution.artifact.retrospective_report_count, 2);
    assert_eq!(execution.artifact.forward_report_count, 1);
    assert_eq!(execution.artifact.refused_as_transition_count, 2);
    assert_eq!(execution.artifact.refused_as_translation_count, 2);
    assert_eq!(execution.artifact.compatible_forward_count, 1);
    assert_eq!(
        execution.artifact.inference_status,
        "retrospective_report_is_not_transition_and_not_translation"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("identity_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
    assert!(!payload.contains("causes"));
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
        Some(RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn compact_oversized_artifact_counts_fail_closed() {
    let assignment_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let artifact = RetrospectiveEdgeArtifact {
        schema_version: RETROSPECTIVE_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        assignment_count,
        retrospective_report_count: 1,
        forward_report_count: assignment_count - 1,
        refused_as_transition_count: 1,
        refused_as_translation_count: 1,
        compatible_forward_count: assignment_count - 1,
        inference_status: "retrospective_report_is_not_transition_and_not_translation".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");
    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidRetrospectiveEdgeArtifact)
    );
    assert_eq!(
        RetrospectiveEdgeArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidRetrospectiveEdgeArtifact)
    );
}

#[test]
fn future_available_assignments_are_excluded() {
    let request = request();
    let mut with_future = mixed_assignments();
    with_future.push(assignment(
        "future-retrospective",
        RetrospectiveKind::RetrospectiveReport,
        "2026-08-02T00:00:00Z",
    ));
    let execution = execute(&request, &with_future).expect("cutoff");
    assert_eq!(execution.artifact.assignment_count, 3);
    assert_eq!(execution.artifact.retrospective_report_count, 2);
}

#[test]
fn empty_or_single_kind_and_duplicate_fail_closed() {
    let request = request();
    let stamp = "2026-07-01T00:00:00Z";
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let retrospectives_only = vec![
        assignment("retro-a", RetrospectiveKind::RetrospectiveReport, stamp),
        assignment("retro-b", RetrospectiveKind::RetrospectiveReport, stamp),
        assignment("retro-c", RetrospectiveKind::RetrospectiveReport, stamp),
    ];
    assert_eq!(
        execute(&request, &retrospectives_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let forwards_only = vec![
        assignment("forward-a", RetrospectiveKind::ForwardReport, stamp),
        assignment("forward-b", RetrospectiveKind::ForwardReport, stamp),
        assignment("forward-c", RetrospectiveKind::ForwardReport, stamp),
    ];
    assert_eq!(
        execute(&request, &forwards_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        assignment("same", RetrospectiveKind::RetrospectiveReport, stamp),
        assignment("same", RetrospectiveKind::ForwardReport, stamp),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        RetrospectiveEdgeAssignment::new(
            "",
            RetrospectiveKind::RetrospectiveReport,
            available(stamp)
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_cutoff_mismatch_and_oversize() {
    let request = request();
    let assignments = mixed_assignments();
    assert_eq!(
        execute_retrospective_edge_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &assignments,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let mut mismatched = request.clone();
    mismatched.knowledge_cutoff = "2026-07-01T00:00:00Z".into();
    assert_eq!(
        execute_retrospective_edge_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-retrospective-edge",
            cutoff(),
            &assignments,
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
        "outcome_order_v1",
        "relation_absence_v1",
        "subevent_containment_v1",
        "episode_membership_v1",
        "inferred_status_v1",
        "role_contradiction_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_retrospective_edge_run(
                &reused,
                &accepted(&reused),
                "snapshot-retrospective-edge",
                cutoff(),
                &assignments,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    let oversized: Vec<RetrospectiveEdgeAssignment> = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            assignment(
                &format!("assignment-{index}"),
                RetrospectiveKind::RetrospectiveReport,
                "2026-07-01T00:00:00Z",
            )
        })
        .collect();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
