//! End-to-end contract for cutoff-safe subevent-containment refusals.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION,
    SUBEVENT_CONTAINMENT_MODEL_CONTRACT_VERSION, SUBEVENT_CONTAINMENT_OUTPUT_PROFILE,
    SubeventContainmentArtifact, SubeventContainmentAssignment, execute_subevent_containment_run,
};
use subevent_containment::EventInterval;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn interval(start: i64, end: i64) -> EventInterval {
    EventInterval::new(start, end).expect("interval")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "subevent-containment-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-subevent-containment".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: SUBEVENT_CONTAINMENT_MODEL_CONTRACT_VERSION.into(),
        output_profile: SUBEVENT_CONTAINMENT_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-subevent-containment",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn assignment(
    assignment_id: &str,
    parent: EventInterval,
    child: EventInterval,
    stamp: &str,
) -> SubeventContainmentAssignment {
    SubeventContainmentAssignment::new(assignment_id, parent, child, available(stamp))
        .expect("assignment")
}

fn mixed_assignments() -> Vec<SubeventContainmentAssignment> {
    let parent = interval(10, 40);
    vec![
        assignment(
            "contained-a",
            parent,
            interval(15, 30),
            "2026-07-01T00:00:00Z",
        ),
        assignment("escaped-b", parent, interval(0, 20), "2026-07-02T00:00:00Z"),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    assignments: &[SubeventContainmentAssignment],
) -> Result<analysis_engine::SubeventContainmentExecution, AnalysisEngineError> {
    execute_subevent_containment_run(
        request,
        &accepted(request),
        "snapshot-subevent-containment",
        cutoff(),
        assignments,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_intervals_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_assignments()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.assignment_count, 2);
    assert_eq!(execution.artifact.contained_count, 1);
    assert_eq!(execution.artifact.escaped_count, 1);
    assert_eq!(execution.artifact.refused_as_escape_count, 1);
    assert_eq!(
        execution.artifact.inference_status,
        "subevent_interval_cannot_escape_parent_interval"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("identity_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
    assert!(!payload.contains("episode_membership"));
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
        Some(SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn compact_oversized_artifact_counts_fail_closed() {
    let assignment_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let escaped_count = assignment_count - 1;
    let artifact = SubeventContainmentArtifact {
        schema_version: SUBEVENT_CONTAINMENT_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        assignment_count,
        contained_count: 1,
        escaped_count,
        refused_as_escape_count: escaped_count,
        inference_status: "subevent_interval_cannot_escape_parent_interval".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");
    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidSubeventContainmentArtifact)
    );
    assert_eq!(
        SubeventContainmentArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidSubeventContainmentArtifact)
    );
}

#[test]
fn future_available_assignments_are_excluded() {
    let request = request();
    let parent = interval(10, 40);
    let mut with_future = mixed_assignments();
    with_future.push(assignment(
        "future-c",
        parent,
        interval(15, 18),
        "2026-08-02T00:00:00Z",
    ));
    let execution = execute(&request, &with_future).expect("cutoff");
    assert_eq!(execution.artifact.assignment_count, 2);
    assert_eq!(execution.artifact.contained_count, 1);
}

#[test]
fn empty_or_single_class_and_duplicate_fail_closed() {
    let request = request();
    let parent = interval(10, 40);
    let stamp = "2026-07-01T00:00:00Z";
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let contained_only = vec![
        assignment("contained-a", parent, interval(15, 18), stamp),
        assignment("contained-b", parent, interval(20, 25), stamp),
    ];
    assert_eq!(
        execute(&request, &contained_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let escaped_only = vec![
        assignment("escaped-a", parent, interval(0, 5), stamp),
        assignment("escaped-b", parent, interval(40, 50), stamp),
    ];
    assert_eq!(
        execute(&request, &escaped_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        assignment("same", parent, interval(15, 18), stamp),
        assignment("same", parent, interval(0, 5), stamp),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        SubeventContainmentAssignment::new("", parent, interval(15, 18), available(stamp)),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_cutoff_mismatch_and_oversize() {
    let request = request();
    let assignments = mixed_assignments();
    assert_eq!(
        execute_subevent_containment_run(
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
        execute_subevent_containment_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-subevent-containment",
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
        "episode_membership_v1",
        "inferred_status_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_subevent_containment_run(
                &reused,
                &accepted(&reused),
                "snapshot-subevent-containment",
                cutoff(),
                &assignments,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    let parent = interval(10, 40);
    let oversized: Vec<SubeventContainmentAssignment> = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            assignment(
                &format!("assignment-{index}"),
                parent,
                interval(15, 18),
                "2026-07-01T00:00:00Z",
            )
        })
        .collect();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
