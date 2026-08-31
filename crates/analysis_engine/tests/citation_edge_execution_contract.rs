//! End-to-end contract for cutoff-safe provenance-is-not-transition refusals.

use analysis_engine::{
    AnalysisEngineError, CITATION_EDGE_ARTIFACT_SCHEMA_VERSION,
    CITATION_EDGE_MODEL_CONTRACT_VERSION, CITATION_EDGE_OUTPUT_PROFILE, CitationEdgeDocument,
    execute_citation_edge_run,
};
use citation_edge::ProvenanceKind;
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest, AnalysisRunTerminalState};

fn cutoff() -> KnowledgeCutoff {
    KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff")
}

fn request() -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "citation-edge-idem".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-citation-edge".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        model_contract_version: CITATION_EDGE_MODEL_CONTRACT_VERSION.into(),
        output_profile: CITATION_EDGE_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-citation-edge", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn mixed_documents() -> Vec<CitationEdgeDocument> {
    vec![
        CitationEdgeDocument::new("cite-a", ProvenanceKind::Citation).expect("citation"),
        CitationEdgeDocument::new("rev-b", ProvenanceKind::Revision).expect("revision"),
        CitationEdgeDocument::new("retro-c", ProvenanceKind::RetrospectiveReport)
            .expect("retrospective"),
    ]
}

fn execute(
    request: &AnalysisRunRequest,
    documents: &[CitationEdgeDocument],
) -> Result<analysis_engine::CitationEdgeExecution, AnalysisEngineError> {
    execute_citation_edge_run(
        request,
        &accepted(request),
        "snapshot-citation-edge",
        cutoff(),
        documents,
        "2026-08-02T00:00:00Z",
    )
}

#[test]
fn mixed_provenance_kinds_emit_digest_bound_refusals_without_recovery_metric() {
    let request = request();
    let execution = execute(&request, &mixed_documents()).expect("execution");
    assert_eq!(
        execution.artifact.schema_version,
        CITATION_EDGE_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(execution.artifact.document_count, 3);
    assert_eq!(execution.artifact.citation_count, 1);
    assert_eq!(execution.artifact.translation_count, 0);
    assert_eq!(execution.artifact.revision_count, 1);
    assert_eq!(execution.artifact.retrospective_report_count, 1);
    assert_eq!(execution.artifact.refused_as_transition_count, 3);
    assert_eq!(execution.artifact.distinct_kind_count, 3);
    assert_eq!(
        execution.artifact.inference_status,
        "provenance_is_not_a_state_transition"
    );
    let payload = execution.artifact.to_json().expect("json");
    assert!(!payload.contains("edge_kind_recovery_rate"));
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
        Some(CITATION_EDGE_ARTIFACT_SCHEMA_VERSION)
    );
}

#[test]
fn empty_single_kind_and_duplicate_identities_fail_closed() {
    let request = request();
    assert_eq!(
        execute(&request, &[]),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let citation_only = vec![
        CitationEdgeDocument::new("cite-a", ProvenanceKind::Citation).expect("citation"),
        CitationEdgeDocument::new("cite-b", ProvenanceKind::Citation).expect("citation"),
    ];
    assert_eq!(
        execute(&request, &citation_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        CitationEdgeDocument::new("same", ProvenanceKind::Citation).expect("citation"),
        CitationEdgeDocument::new("same", ProvenanceKind::Revision).expect("revision"),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        CitationEdgeDocument::new("", ProvenanceKind::Citation),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn execution_refuses_snapshot_profile_and_cutoff_mismatch() {
    let request = request();
    let documents = mixed_documents();
    assert_eq!(
        execute_citation_edge_run(
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
        execute_citation_edge_run(
            &mismatched,
            &accepted(&mismatched),
            "snapshot-citation-edge",
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
        "lineage_criterion_v1",
        "composed_fitted_lineage_v1",
        "case_deletion_refit_v1",
        "topic_activity_v1",
    ] {
        let mut reused = request.clone();
        reused.output_profile = profile.into();
        assert_eq!(
            execute_citation_edge_run(
                &reused,
                &accepted(&reused),
                "snapshot-citation-edge",
                cutoff(),
                &documents,
                "2026-08-02T00:00:00Z",
            ),
            Err(AnalysisEngineError::InvalidEvidence)
        );
    }
}
