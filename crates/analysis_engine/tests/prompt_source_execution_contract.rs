//! End-to-end contract for cutoff-safe prompt-boilerplate refusals.

use analysis_engine::{
    AnalysisEngineError, PROMPT_SOURCE_ARTIFACT_SCHEMA_VERSION,
    PROMPT_SOURCE_MODEL_CONTRACT_VERSION, PROMPT_SOURCE_OUTPUT_PROFILE, PromptSourceDocument,
    execute_prompt_source_run,
};
use prompt_source::PromptKind;
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "prompt-source-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-prompt-source".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: PROMPT_SOURCE_MODEL_CONTRACT_VERSION.into(),
        output_profile: PROMPT_SOURCE_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-prompt-source", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn mixed_documents() -> Vec<PromptSourceDocument> {
    vec![
        PromptSourceDocument::new("unique-a", PromptKind::UniqueContent).expect("unique"),
        PromptSourceDocument::new("prompt-b", PromptKind::PromptBoilerplate).expect("prompt"),
        PromptSourceDocument::new("prompt-c", PromptKind::PromptBoilerplate).expect("prompt"),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    documents: &[PromptSourceDocument],
) -> Result<analysis_engine::PromptSourceExecution, AnalysisEngineError> {
    execute_prompt_source_run(
        request,
        &accepted(request),
        "snapshot-prompt-source",
        cutoff(),
        documents,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_prompt_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_documents()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        PROMPT_SOURCE_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.document_count, 3);
    assert_eq!(execution.artifact.unique_content_count, 1);
    assert_eq!(execution.artifact.prompt_boilerplate_count, 2);
    assert_eq!(execution.artifact.refused_as_unique_content_count, 2);
    assert_eq!(execution.artifact.refused_as_stopword_deletion_count, 2);
    assert_eq!(
        execution.artifact.inference_status,
        "prompt_boilerplate_is_not_unique_content_not_stopword_deletion"
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
        Some(PROMPT_SOURCE_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn empty_unique_only_prompt_only_and_duplicate_identities_fail_closed() {
    let request = request();
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let unique_only = vec![
        PromptSourceDocument::new("unique-a", PromptKind::UniqueContent).expect("unique"),
        PromptSourceDocument::new("unique-b", PromptKind::UniqueContent).expect("unique"),
    ];
    assert_eq!(
        execute(&request, &unique_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let prompt_only = vec![
        PromptSourceDocument::new("prompt-a", PromptKind::PromptBoilerplate).expect("prompt"),
        PromptSourceDocument::new("prompt-b", PromptKind::PromptBoilerplate).expect("prompt"),
    ];
    assert_eq!(
        execute(&request, &prompt_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        PromptSourceDocument::new("same", PromptKind::UniqueContent).expect("unique"),
        PromptSourceDocument::new("same", PromptKind::PromptBoilerplate).expect("prompt"),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        PromptSourceDocument::new("", PromptKind::UniqueContent),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let documents = mixed_documents();
    assert_eq!(
        execute_prompt_source_run(
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
        execute_prompt_source_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-prompt-source",
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
        "composed_fitted_lineage_v1",
        "case_deletion_refit_v1",
        "topic_activity_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_prompt_source_run(
                &reused,
                &accepted(&reused),
                "snapshot-prompt-source",
                cutoff(),
                &documents,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
