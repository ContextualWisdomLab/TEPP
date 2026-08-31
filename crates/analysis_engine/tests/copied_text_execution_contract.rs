//! End-to-end contract for cutoff-safe copied-text refusals.

use analysis_engine::{
    AnalysisEngineError, COPIED_TEXT_ARTIFACT_SCHEMA_VERSION, COPIED_TEXT_MODEL_CONTRACT_VERSION,
    COPIED_TEXT_OUTPUT_PROFILE, CopiedTextDocument, execute_copied_text_run,
};
use copied_text::CopiedKind;
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "copied-text-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-copied-text".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: COPIED_TEXT_MODEL_CONTRACT_VERSION.into(),
        output_profile: COPIED_TEXT_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-copied-text", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn mixed_documents() -> Vec<CopiedTextDocument> {
    vec![
        CopiedTextDocument::new("unique-a", CopiedKind::UniqueContent).expect("unique"),
        CopiedTextDocument::new("copied-b", CopiedKind::CopiedText).expect("copied"),
        CopiedTextDocument::new("copied-c", CopiedKind::CopiedText).expect("copied"),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    documents: &[CopiedTextDocument],
) -> Result<analysis_engine::CopiedTextExecution, AnalysisEngineError> {
    execute_copied_text_run(
        request,
        &accepted(request),
        "snapshot-copied-text",
        cutoff(),
        documents,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_copied_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_documents()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        COPIED_TEXT_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.document_count, 3);
    assert_eq!(execution.artifact.unique_content_count, 1);
    assert_eq!(execution.artifact.copied_text_count, 2);
    assert_eq!(execution.artifact.refused_as_unique_content_count, 2);
    assert_eq!(execution.artifact.refused_as_stopword_deletion_count, 2);
    assert_eq!(
        execution.artifact.inference_status,
        "copied_text_is_not_unique_content_not_stopword_deletion"
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
        Some(COPIED_TEXT_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn empty_unique_only_copied_only_and_duplicate_identities_fail_closed() {
    let request = request();
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let unique_only = vec![
        CopiedTextDocument::new("unique-a", CopiedKind::UniqueContent).expect("unique"),
        CopiedTextDocument::new("unique-b", CopiedKind::UniqueContent).expect("unique"),
    ];
    assert_eq!(
        execute(&request, &unique_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let copied_only = vec![
        CopiedTextDocument::new("copied-a", CopiedKind::CopiedText).expect("copied"),
        CopiedTextDocument::new("copied-b", CopiedKind::CopiedText).expect("copied"),
    ];
    assert_eq!(
        execute(&request, &copied_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        CopiedTextDocument::new("same", CopiedKind::UniqueContent).expect("unique"),
        CopiedTextDocument::new("same", CopiedKind::CopiedText).expect("copied"),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        CopiedTextDocument::new("", CopiedKind::UniqueContent),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let documents = mixed_documents();
    assert_eq!(
        execute_copied_text_run(
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
        execute_copied_text_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-copied-text",
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
        "lineage_criterion_v1",
        "composed_fitted_lineage_v1",
        "case_deletion_refit_v1",
        "topic_activity_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_copied_text_run(
                &reused,
                &accepted(&reused),
                "snapshot-copied-text",
                cutoff(),
                &documents,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
