//! End-to-end contract for cutoff-safe corpus-background refusals.

use analysis_engine::{
    AnalysisEngineError, CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION,
    CORPUS_BACKGROUND_MODEL_CONTRACT_VERSION, CORPUS_BACKGROUND_OUTPUT_PROFILE,
    CorpusBackgroundDocument, execute_corpus_background_run,
};
use corpus_background::CorpusBackgroundKind;
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "corpus-background-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-corpus-background".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: CORPUS_BACKGROUND_MODEL_CONTRACT_VERSION.into(),
        output_profile: CORPUS_BACKGROUND_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new(
        "run-corpus-background",
        "accepted",
        &request.idempotency_key,
    )
    .expect("accepted")
}

fn mixed_documents() -> Vec<CorpusBackgroundDocument> {
    vec![
        CorpusBackgroundDocument::new("unique-a", CorpusBackgroundKind::UniqueContent)
            .expect("unique"),
        CorpusBackgroundDocument::new("bg-b", CorpusBackgroundKind::CorpusBackground)
            .expect("background"),
        CorpusBackgroundDocument::new("bg-c", CorpusBackgroundKind::CorpusBackground)
            .expect("background"),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    documents: &[CorpusBackgroundDocument],
) -> Result<analysis_engine::CorpusBackgroundExecution, AnalysisEngineError> {
    execute_corpus_background_run(
        request,
        &accepted(request),
        "snapshot-corpus-background",
        cutoff(),
        documents,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_background_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_documents()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.document_count, 3);
    assert_eq!(execution.artifact.unique_content_count, 1);
    assert_eq!(execution.artifact.corpus_background_count, 2);
    assert_eq!(execution.artifact.refused_as_unique_content_count, 2);
    assert_eq!(execution.artifact.refused_as_stopword_deletion_count, 2);
    assert_eq!(
        execution.artifact.inference_status,
        "corpus_background_is_not_unique_content_not_stopword_deletion"
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
        Some(CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn empty_unique_only_background_only_and_duplicate_identities_fail_closed() {
    let request = request();
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let unique_only = vec![
        CorpusBackgroundDocument::new("unique-a", CorpusBackgroundKind::UniqueContent)
            .expect("unique"),
        CorpusBackgroundDocument::new("unique-b", CorpusBackgroundKind::UniqueContent)
            .expect("unique"),
    ];
    assert_eq!(
        execute(&request, &unique_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let background_only = vec![
        CorpusBackgroundDocument::new("bg-a", CorpusBackgroundKind::CorpusBackground)
            .expect("background"),
        CorpusBackgroundDocument::new("bg-b", CorpusBackgroundKind::CorpusBackground)
            .expect("background"),
    ];
    assert_eq!(
        execute(&request, &background_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        CorpusBackgroundDocument::new("same", CorpusBackgroundKind::UniqueContent).expect("unique"),
        CorpusBackgroundDocument::new("same", CorpusBackgroundKind::CorpusBackground)
            .expect("background"),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        CorpusBackgroundDocument::new("", CorpusBackgroundKind::UniqueContent),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let documents = mixed_documents();
    assert_eq!(
        execute_corpus_background_run(
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
        execute_corpus_background_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-corpus-background",
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
        "composed_fitted_lineage_v1",
        "case_deletion_refit_v1",
        "topic_activity_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_corpus_background_run(
                &reused,
                &accepted(&reused),
                "snapshot-corpus-background",
                cutoff(),
                &documents,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
