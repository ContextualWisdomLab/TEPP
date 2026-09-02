//! End-to-end contract for cutoff-safe role-contradiction refusals.

use analysis_engine::{
    AnalysisEngineError, MAX_EVIDENCE_UNITS, ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION,
    ROLE_CONTRADICTION_MODEL_CONTRACT_VERSION, ROLE_CONTRADICTION_OUTPUT_PROFILE,
    RoleContradictionArtifact, RoleContradictionAssignment, execute_role_contradiction_run,
};
use role_contradiction::ContextualRole;
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
        idempotency_key: "role-contradiction-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-role-contradiction".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: ROLE_CONTRADICTION_MODEL_CONTRACT_VERSION.into(),
        output_profile: ROLE_CONTRADICTION_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-role-contradiction",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn assignment(
    assignment_id: &str,
    group_id: &str,
    role: ContextualRole,
    stamp: &str,
) -> RoleContradictionAssignment {
    RoleContradictionAssignment::new(assignment_id, group_id, role, available(stamp))
        .expect("assignment")
}

fn mixed_assignments() -> Vec<RoleContradictionAssignment> {
    vec![
        assignment(
            "customer-mixed",
            "group-mixed",
            ContextualRole::Customer,
            "2026-07-01T00:00:00Z",
        ),
        assignment(
            "competitor-mixed",
            "group-mixed",
            ContextualRole::Competitor,
            "2026-07-02T00:00:00Z",
        ),
        assignment(
            "partner-mixed",
            "group-mixed",
            ContextualRole::Partner,
            "2026-07-03T00:00:00Z",
        ),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    assignments: &[RoleContradictionAssignment],
) -> Result<analysis_engine::RoleContradictionExecution, AnalysisEngineError> {
    execute_role_contradiction_run(
        request,
        &accepted(request),
        "snapshot-role-contradiction",
        cutoff(),
        assignments,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_roles_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_assignments()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.assignment_count, 3);
    assert_eq!(execution.artifact.customer_count, 1);
    assert_eq!(execution.artifact.partner_count, 1);
    assert_eq!(execution.artifact.competitor_count, 1);
    assert_eq!(execution.artifact.refused_as_entity_class_count, 3);
    assert_eq!(execution.artifact.refused_contradictory_pair_count, 1);
    assert_eq!(execution.artifact.compatible_pair_count, 2);
    assert_eq!(
        execution.artifact.inference_status,
        "customer_competitor_cannot_share_group_role_is_not_entity_class"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("identity_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
    assert!(!payload.contains("permanent_entity"));
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
        Some(ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn compact_oversized_artifact_counts_fail_closed() {
    let assignment_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let artifact = RoleContradictionArtifact {
        schema_version: ROLE_CONTRADICTION_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        assignment_count,
        customer_count: 1,
        partner_count: 1,
        competitor_count: assignment_count - 2,
        refused_as_entity_class_count: assignment_count,
        refused_contradictory_pair_count: 1,
        compatible_pair_count: 1,
        inference_status: "customer_competitor_cannot_share_group_role_is_not_entity_class".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");
    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidRoleContradictionArtifact)
    );
    assert_eq!(
        RoleContradictionArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidRoleContradictionArtifact)
    );
}

#[test]
fn future_available_assignments_are_excluded() {
    let request = request();
    let mut with_future = mixed_assignments();
    with_future.push(assignment(
        "future-customer",
        "group-future",
        ContextualRole::Customer,
        "2026-08-02T00:00:00Z",
    ));
    let execution = execute(&request, &with_future).expect("cutoff");
    assert_eq!(execution.artifact.assignment_count, 3);
    assert_eq!(execution.artifact.customer_count, 1);
}

#[test]
fn future_duplicate_identity_cannot_change_a_historical_cutoff_result() {
    let request = request();
    let mut with_future_duplicate = mixed_assignments();
    with_future_duplicate.push(assignment(
        "customer-mixed",
        "group-future",
        ContextualRole::Competitor,
        "2026-08-02T00:00:00Z",
    ));

    let execution = execute(&request, &with_future_duplicate)
        .expect("future-unavailable evidence must not affect the historical run");
    assert_eq!(execution.artifact.assignment_count, 3);
    assert_eq!(execution.artifact.customer_count, 1);
    assert_eq!(execution.artifact.partner_count, 1);
    assert_eq!(execution.artifact.competitor_count, 1);
}

#[test]
fn empty_or_single_class_and_duplicate_fail_closed() {
    let request = request();
    let stamp = "2026-07-01T00:00:00Z";
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let customers_only = vec![
        assignment("customer-a", "group-a", ContextualRole::Customer, stamp),
        assignment("customer-b", "group-b", ContextualRole::Customer, stamp),
        assignment("customer-c", "group-c", ContextualRole::Customer, stamp),
    ];
    assert_eq!(
        execute(&request, &customers_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let no_competitor = vec![
        assignment(
            "customer-a",
            "group-compat",
            ContextualRole::Customer,
            stamp,
        ),
        assignment("partner-b", "group-compat", ContextualRole::Partner, stamp),
        assignment("partner-c", "group-other", ContextualRole::Partner, stamp),
    ];
    assert_eq!(
        execute(&request, &no_competitor),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let no_partner = vec![
        assignment(
            "customer-a",
            "group-conflict",
            ContextualRole::Customer,
            stamp,
        ),
        assignment(
            "competitor-b",
            "group-conflict",
            ContextualRole::Competitor,
            stamp,
        ),
        assignment(
            "competitor-c",
            "group-other",
            ContextualRole::Competitor,
            stamp,
        ),
    ];
    assert_eq!(
        execute(&request, &no_partner),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        assignment("same", "group-mixed", ContextualRole::Customer, stamp),
        assignment("same", "group-mixed", ContextualRole::Competitor, stamp),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        RoleContradictionAssignment::new("", "group-a", ContextualRole::Customer, available(stamp)),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        RoleContradictionAssignment::new(
            "assignment-a",
            "",
            ContextualRole::Customer,
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
        execute_role_contradiction_run(
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
        execute_role_contradiction_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-role-contradiction",
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
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_role_contradiction_run(
                &reused,
                &accepted(&reused),
                "snapshot-role-contradiction",
                cutoff(),
                &assignments,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    let oversized: Vec<RoleContradictionAssignment> = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            assignment(
                &format!("assignment-{index}"),
                "group-mixed",
                ContextualRole::Customer,
                "2026-07-01T00:00:00Z",
            )
        })
        .collect();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
