//! End-to-end contract for cutoff-safe non-lexical modality refusals.

use analysis_engine::{
    AnalysisEngineError, MODALITY_SOURCE_ARTIFACT_SCHEMA_VERSION,
    MODALITY_SOURCE_MODEL_CONTRACT_VERSION, MODALITY_SOURCE_OUTPUT_PROFILE, ModalitySourceDocument,
    execute_modality_source_run,
};
use modality_source::ModalityKind;
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "modality-source-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-modality-source".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: MODALITY_SOURCE_MODEL_CONTRACT_VERSION.into(),
        output_profile: MODALITY_SOURCE_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-modality-source", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn mixed_documents() -> Vec<ModalitySourceDocument> {
    vec![
        ModalitySourceDocument::new("unique-a", ModalityKind::UniqueContent).expect("unique"),
        ModalitySourceDocument::new("modality-b", ModalityKind::NonLexicalModality)
            .expect("modality"),
        ModalitySourceDocument::new("modality-c", ModalityKind::NonLexicalModality)
            .expect("modality"),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    documents: &[ModalitySourceDocument],
) -> Result<analysis_engine::ModalitySourceExecution, AnalysisEngineError> {
    execute_modality_source_run(
        request,
        &accepted(request),
        "snapshot-modality-source",
        cutoff(),
        documents,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_modality_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_documents()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        MODALITY_SOURCE_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.document_count, 3);
    assert_eq!(execution.artifact.unique_content_count, 1);
    assert_eq!(execution.artifact.non_lexical_modality_count, 2);
    assert_eq!(execution.artifact.refused_as_unique_content_count, 2);
    assert_eq!(execution.artifact.refused_as_stopword_deletion_count, 2);
    assert_eq!(
        execution.artifact.inference_status,
        "non_lexical_modality_is_not_unique_content_not_stopword_deletion"
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
        Some(MODALITY_SOURCE_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn empty_unique_only_modality_only_and_duplicate_identities_fail_closed() {
    let request = request();
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let unique_only = vec![
        ModalitySourceDocument::new("unique-a", ModalityKind::UniqueContent).expect("unique"),
        ModalitySourceDocument::new("unique-b", ModalityKind::UniqueContent).expect("unique"),
    ];
    assert_eq!(
        execute(&request, &unique_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let modality_only = vec![
        ModalitySourceDocument::new("modality-a", ModalityKind::NonLexicalModality)
            .expect("modality"),
        ModalitySourceDocument::new("modality-b", ModalityKind::NonLexicalModality)
            .expect("modality"),
    ];
    assert_eq!(
        execute(&request, &modality_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        ModalitySourceDocument::new("same", ModalityKind::UniqueContent).expect("unique"),
        ModalitySourceDocument::new("same", ModalityKind::NonLexicalModality).expect("modality"),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        ModalitySourceDocument::new("", ModalityKind::UniqueContent),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let documents = mixed_documents();
    assert_eq!(
        execute_modality_source_run(
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
        execute_modality_source_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-modality-source",
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
        "composed_fitted_lineage_v1",
        "case_deletion_refit_v1",
        "topic_activity_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_modality_source_run(
                &reused,
                &accepted(&reused),
                "snapshot-modality-source",
                cutoff(),
                &documents,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
