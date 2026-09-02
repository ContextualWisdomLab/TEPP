//! End-to-end contract for cutoff-safe membership-target refusals.

use analysis_engine::{
    execute_membership_target_run, AnalysisEngineError, MembershipTargetArtifact,
    MembershipTargetDocument, MAX_EVIDENCE_UNITS, MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION,
    MEMBERSHIP_TARGET_MODEL_CONTRACT_VERSION, MEMBERSHIP_TARGET_OUTPUT_PROFILE,
};
use membership_target::MembershipTargetKind;
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "membership-target-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-membership-target".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: MEMBERSHIP_TARGET_MODEL_CONTRACT_VERSION.into(),
        output_profile: MEMBERSHIP_TARGET_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-membership-target",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn mixed_documents() -> Vec<MembershipTargetDocument> {
    vec![
        MembershipTargetDocument::new("lang-a", MembershipTargetKind::Language).expect("language"),
        MembershipTargetDocument::new("ep-b", MembershipTargetKind::Episode).expect("episode"),
        MembershipTargetDocument::new("tmpl-c", MembershipTargetKind::Template).expect("template"),
        MembershipTargetDocument::new("dept-d", MembershipTargetKind::Department)
            .expect("department"),
        MembershipTargetDocument::new("pool-e", MembershipTargetKind::OpportunityPool)
            .expect("opportunity"),
        MembershipTargetDocument::new("ent-f", MembershipTargetKind::Entity).expect("entity"),
        MembershipTargetDocument::new("proj-g", MembershipTargetKind::Project).expect("project"),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    documents: &[MembershipTargetDocument],
) -> Result<analysis_engine::MembershipTargetExecution, AnalysisEngineError> {
    execute_membership_target_run(
        request,
        &accepted(request),
        "snapshot-membership-target",
        cutoff(),
        documents,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_target_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_documents()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.document_count, 7);
    assert_eq!(execution.artifact.language_count, 1);
    assert_eq!(execution.artifact.episode_count, 1);
    assert_eq!(execution.artifact.template_count, 1);
    assert_eq!(execution.artifact.department_count, 1);
    assert_eq!(execution.artifact.opportunity_pool_count, 1);
    assert_eq!(execution.artifact.entity_count, 1);
    assert_eq!(execution.artifact.project_count, 1);
    assert_eq!(execution.artifact.refused_as_entity_count, 5);
    assert_eq!(execution.artifact.refused_as_project_count, 5);
    assert_eq!(
        execution.artifact.inference_status,
        "language_episode_template_department_opportunity_pool_are_not_entities"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("identity_recovery_rate"));
    assert!(!payload.contains("scientific_acceptance"));
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
        Some(MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn compact_oversized_artifact_counts_fail_closed() {
    let document_count = MAX_EVIDENCE_UNITS as u64 + 1;
    let language_count = document_count - 2;
    let artifact = MembershipTargetArtifact {
        schema_version: MEMBERSHIP_TARGET_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-compact-oversize".into(),
        snapshot_id: "snapshot-compact-oversize".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        document_count,
        language_count,
        episode_count: 0,
        template_count: 0,
        department_count: 0,
        opportunity_pool_count: 0,
        entity_count: 1,
        project_count: 1,
        refused_as_entity_count: language_count,
        refused_as_project_count: language_count,
        inference_status: "language_episode_template_department_opportunity_pool_are_not_entities".into(),
    };
    let raw_payload = serde_json::to_string(&artifact).expect("raw json");
    assert_eq!(
        artifact.to_json(),
        Err(AnalysisEngineError::InvalidMembershipTargetArtifact)
    );
    assert_eq!(
        MembershipTargetArtifact::from_json(&raw_payload),
        Err(AnalysisEngineError::InvalidMembershipTargetArtifact)
    );
}

#[test]
fn empty_single_class_and_duplicate_identities_fail_closed() {
    let request = request();
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let language_only = vec![
        MembershipTargetDocument::new("lang-a", MembershipTargetKind::Language).expect("language"),
        MembershipTargetDocument::new("lang-b", MembershipTargetKind::Language).expect("language"),
    ];
    assert_eq!(
        execute(&request, &language_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let entity_only = vec![
        MembershipTargetDocument::new("ent-a", MembershipTargetKind::Entity).expect("entity"),
        MembershipTargetDocument::new("ent-b", MembershipTargetKind::Entity).expect("entity"),
    ];
    assert_eq!(
        execute(&request, &entity_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let project_only = vec![
        MembershipTargetDocument::new("proj-a", MembershipTargetKind::Project).expect("project"),
        MembershipTargetDocument::new("proj-b", MembershipTargetKind::Project).expect("project"),
    ];
    assert_eq!(
        execute(&request, &project_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let typed_only = vec![
        MembershipTargetDocument::new("lang-a", MembershipTargetKind::Language).expect("language"),
        MembershipTargetDocument::new("ep-b", MembershipTargetKind::Episode).expect("episode"),
    ];
    assert_eq!(
        execute(&request, &typed_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let persistence_only = vec![
        MembershipTargetDocument::new("ent-a", MembershipTargetKind::Entity).expect("entity"),
        MembershipTargetDocument::new("proj-b", MembershipTargetKind::Project).expect("project"),
    ];
    assert_eq!(
        execute(&request, &persistence_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        MembershipTargetDocument::new("same", MembershipTargetKind::Language).expect("language"),
        MembershipTargetDocument::new("same", MembershipTargetKind::Entity).expect("entity"),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        MembershipTargetDocument::new("", MembershipTargetKind::Language),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_cutoff_mismatch_and_oversize() {
    let request = request();
    let documents = mixed_documents();
    assert_eq!(
        execute_membership_target_run(
            &request,
            &accepted(&request),
            "other-snapshot",
            cutoff(),
            &documents,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::SnapshotMismatch)
    );
    let mut mismatched = request.clone();
    mismatched.knowledge_cutoff = "2026-07-01T00:00:00Z".into();
    assert_eq!(
        execute_membership_target_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-membership-target",
            cutoff(),
            &documents,
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
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_membership_target_run(
                &reused,
                &accepted(&reused),
                "snapshot-membership-target",
                cutoff(),
                &documents,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
    let oversized: Vec<MembershipTargetDocument> = (0..=MAX_EVIDENCE_UNITS)
        .map(|index| {
            let kind = if index == MAX_EVIDENCE_UNITS {
                MembershipTargetKind::Entity
            } else {
                MembershipTargetKind::Language
            };
            MembershipTargetDocument::new(format!("document-{index}"), kind).expect("document")
        })
        .collect();
    assert_eq!(
        execute(&request, &oversized),
        Err(AnalysisEngineError::LimitExceeded)
    );
}
