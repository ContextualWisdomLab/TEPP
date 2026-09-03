//! End-to-end contract for cutoff-safe episode-membership refusals.

use analysis_engine::{
    AnalysisEngineError, EPISODE_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION,
    EPISODE_MEMBERSHIP_MODEL_CONTRACT_VERSION, EPISODE_MEMBERSHIP_OUTPUT_PROFILE,
    EpisodeMembershipArtifact, EpisodeMembershipAssignment, MAX_EVIDENCE_UNITS,
    execute_episode_membership_run,
};
use episode_membership::EventWindow;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn available(stamp: &str) -> AvailableTime {
    AvailableTime::parse_rfc3339(stamp).expect("available")
}

fn window(start: i64, end: i64) -> EventWindow {
    EventWindow::new(start, end).expect("window")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "episode-membership-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-episode-membership".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: EPISODE_MEMBERSHIP_MODEL_CONTRACT_VERSION.into(),
        output_profile: EPISODE_MEMBERSHIP_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-episode-membership",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn assignment(
    assignment_id: &str,
    membership: EventWindow,
    episode: EventWindow,
    stamp: &str,
) -> EpisodeMembershipAssignment {
    EpisodeMembershipAssignment::new(assignment_id, membership, episode, available(stamp))
        .expect("assignment")
}

fn mixed_assignments() -> Vec<EpisodeMembershipAssignment> {
    let episode = window(10, 20);
    vec![
        assignment(
            "contained-a",
            window(11, 19),
            episode,
            "2026-07-01T00:00:00Z",
        ),
        assignment("escaped-b", window(9, 15), episode, "2026-07-02T00:00:00Z"),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    assignments: &[EpisodeMembershipAssignment],
) -> Result<analysis_engine::EpisodeMembershipExecution, AnalysisEngineError> {
    execute_episode_membership_run(
        request,
        &accepted(request),
        "snapshot-episode-membership",
        cutoff(),
        assignments,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_windows_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_assignments()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        EPISODE_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.assignment_count, 2);
    assert_eq!(execution.artifact.contained_count, 1);
    assert_eq!(execution.artifact.escaped_count, 1);
    assert_eq!(execution.artifact.refused_as_escape_count, 1);
    assert_eq!(
        execution.artifact.inference_status,
        "membership_window_cannot_escape_episode_interval"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("identity_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
    assert!(!payload.contains("subevent"));
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
        Some(EPISODE_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn equivalent_rfc3339_cutoff_offsets_are_the_same_instant() {
    let mut offset_request = request();
    offset_request.knowledge_cutoff = "2026-08-01T09:00:00+09:00".into();
    let execution = execute(&offset_request, &mixed_assignments()).expect("equivalent cutoff");
    assert_eq!(
        execution.artifact.knowledge_cutoff,
        "2026-08-01T00:00:00Z"
    );
}

#[test]
fn compact_oversized_artifact_counts_fail_closed() {
    let assignment_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let escaped_count = assignment_count - 1;
    let artifact = EpisodeMembershipArtifact {
        schema_version: EPISODE_MEMBERSHIP_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        assignment_count,
        contained_count: 1,
        escaped_count,
        refused_as_escape_count: escaped_count,
        inference_status: "membership_window_cannot_escape_episode_interval".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");
    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidEpisodeMembershipArtifact)
    );
    assert_eq!(
        EpisodeMembershipArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidEpisodeMembershipArtifact)
    );
}

#[test]
fn future_available_assignments_are_excluded() {
    let request = request();
    let episode = window(10, 20);
    let mut with_future = mixed_assignments();
    with_future.push(assignment(
        "future-c",
        window(11, 12),
        episode,
        "2026-08-02T00:00:00Z",
    ));
    let execution = execute(&request, &with_future).expect("cutoff");
    assert_eq!(execution.artifact.assignment_count, 2);
    assert_eq!(execution.artifact.contained_count, 1);
}

#[test]
fn empty_or_single_class_and_duplicate_fail_closed() {
    let request = request();
    let episode = window(10, 20);
    let stamp = "2026-07-01T00:00:00Z";
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let contained_only = vec![
        assignment("contained-a", window(11, 12), episode, stamp),
        assignment("contained-b", window(13, 14), episode, stamp),
    ];
    assert_eq!(
        execute(&request, &contained_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let escaped_only = vec![
        assignment("escaped-a", window(1, 5), episode, stamp),
        assignment("escaped-b", window(21, 25), episode, stamp),
    ];
    assert_eq!(
        execute(&request, &escaped_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        assignment("same", window(11, 12), episode, stamp),
        assignment("same", window(1, 5), episode, stamp),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        EpisodeMembershipAssignment::new("", window(11, 12), episode, available(stamp)),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_cutoff_mismatch_and_oversize() {
    let request = request();
    let assignments = mixed_assignments();
    assert_eq!(
        execute_episode_membership_run(
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
        execute_episode_membership_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-episode-membership",
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
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_episode_membership_run(
                &reused,
                &accepted(&reused),
                "snapshot-episode-membership",
                cutoff(),
                &assignments,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    let episode = window(10, 20);
    let oversized: Vec<EpisodeMembershipAssignment> = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            assignment(
                &format!("assignment-{index}"),
                window(11, 12),
                episode,
                "2026-07-01T00:00:00Z",
            )
        })
        .collect();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
