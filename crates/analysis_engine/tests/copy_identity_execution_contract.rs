//! End-to-end contract for cutoff-safe template-copy identity refusals.

use analysis_engine::{
    AnalysisEngineError, COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION,
    COPY_IDENTITY_MODEL_CONTRACT_VERSION, COPY_IDENTITY_OUTPUT_PROFILE, CopyIdentityDocument,
    execute_copy_identity_run,
};
use copy_identity::CopyKind;
use temporal_core::{AvailableTime, KnowledgeCutoff};
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "copy-identity-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-copy-identity".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: COPY_IDENTITY_MODEL_CONTRACT_VERSION.into(),
        output_profile: COPY_IDENTITY_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-copy-identity", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn document(id: &str, kind: CopyKind) -> CopyIdentityDocument {
    CopyIdentityDocument::new(
        id,
        kind,
        AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available"),
    )
    .expect("document")
}

fn mixed_documents() -> Vec<CopyIdentityDocument> {
    vec![
        document("source-a", CopyKind::SourceDocument),
        document("copy-b", CopyKind::TemplateCopy),
        document("copy-c", CopyKind::TemplateCopy),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    documents: &[CopyIdentityDocument],
) -> Result<analysis_engine::CopyIdentityExecution, AnalysisEngineError> {
    execute_copy_identity_run(
        request,
        &accepted(request),
        "snapshot-copy-identity",
        cutoff(),
        documents,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_copy_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_documents()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.document_count, 3);
    assert_eq!(execution.artifact.source_document_count, 1);
    assert_eq!(execution.artifact.template_copy_count, 2);
    assert_eq!(execution.artifact.refused_as_source_count, 2);
    assert_eq!(execution.artifact.refused_as_transition_count, 2);
    assert_eq!(
        execution.artifact.inference_status,
        "template_copy_is_not_source_identity_not_transition"
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
        Some(COPY_IDENTITY_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn empty_source_only_copy_only_and_duplicate_identities_fail_closed() {
    let request = request();
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let sources_only = vec![
        document("source-a", CopyKind::SourceDocument),
        document("source-b", CopyKind::SourceDocument),
    ];
    assert_eq!(
        execute(&request, &sources_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let copies_only = vec![
        document("copy-a", CopyKind::TemplateCopy),
        document("copy-b", CopyKind::TemplateCopy),
    ];
    assert_eq!(
        execute(&request, &copies_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        document("same", CopyKind::SourceDocument),
        document("same", CopyKind::TemplateCopy),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        CopyIdentityDocument::new(
            "",
            CopyKind::SourceDocument,
            AvailableTime::parse_rfc3339("2026-07-01T00:00:00Z").expect("available"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn future_available_documents_fail_closed() {
    let request = request();
    let mut documents = mixed_documents();
    documents.push(
        CopyIdentityDocument::new(
            "future-copy",
            CopyKind::TemplateCopy,
            AvailableTime::parse_rfc3339("2026-08-02T00:00:00Z").expect("available"),
        )
        .expect("document"),
    );
    assert_eq!(
        execute(&request, &documents),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let documents = mixed_documents();
    assert_eq!(
        execute_copy_identity_run(
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
        execute_copy_identity_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-copy-identity",
            cutoff(),
            &documents,
            "2026-08-02T00:00:00Z",
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let mut model_mismatch = request.clone();
    model_mismatch.model_contract_version = "other-model".into();
    assert_eq!(
        execute_copy_identity_run(
            &model_mismatch,
            &accepted(&model_mismatch),
            "snapshot-copy-identity",
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
        "composed_fitted_lineage_v1",
        "case_deletion_refit_v1",
        "topic_activity_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_copy_identity_run(
                &reused,
                &accepted(&reused),
                "snapshot-copy-identity",
                cutoff(),
                &documents,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
