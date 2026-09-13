//! End-to-end contract for cutoff-safe provenance-is-not-transition refusals.

use analysis_engine::{
    AnalysisEngineError, CITATION_EDGE_ARTIFACT_SCHEMA_VERSION,
    CITATION_EDGE_MODEL_CONTRACT_VERSION, CITATION_EDGE_OUTPUT_PROFILE, CitationEdgeDocument,
    MAX_EVIDENCE_UNITS, execute_citation_edge_run,
};
use citation_edge::ProvenanceKind;
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

fn document(document_id: &str, kind: ProvenanceKind, available_time: &str) -> CitationEdgeDocument {
    CitationEdgeDocument::new(
        document_id,
        kind,
        "snapshot-citation-edge",
        available(available_time),
    )
    .expect("document")
}

fn mixed_documents() -> Vec<CitationEdgeDocument> {
    vec![
        document(
            "cite-a",
            ProvenanceKind::Citation,
            "2026-07-31T22:00:00Z",
        ),
        document(
            "rev-b",
            ProvenanceKind::Revision,
            "2026-07-31T23:00:00Z",
        ),
        document(
            "retro-c",
            ProvenanceKind::RetrospectiveReport,
            "2026-08-01T00:00:00Z",
        ),
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
fn equivalent_rfc3339_cutoff_spellings_bind_the_same_instant() {
    let mut equivalent = request();
    equivalent.knowledge_cutoff = "2026-08-01T01:00:00+01:00".into();
    let execution = execute_citation_edge_run(
        &equivalent,
        &accepted(&equivalent),
        "snapshot-citation-edge",
        cutoff(),
        &mixed_documents(),
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent cutoff instant");
    assert_eq!(execution.artifact.knowledge_cutoff, cutoff().to_rfc3339());
}

#[test]
fn terminal_summary_keeps_validation_status_separate_from_domain_inference() {
    let request = request();
    let execution = execute(&request, &mixed_documents()).expect("execution");
    let summary = execution
        .terminal_result
        .summary
        .as_ref()
        .expect("succeeded summary");
    assert_eq!(summary.validation_status, "validated");
    assert_ne!(summary.validation_status, execution.artifact.inference_status);
}

#[test]
fn future_unavailable_duplicate_cannot_change_historical_replay() {
    let request = request();
    let baseline = execute(&request, &mixed_documents()).expect("baseline");
    let mut with_future = vec![document(
        "cite-a",
        ProvenanceKind::Citation,
        "2026-08-01T00:00:01Z",
    )];
    with_future.extend(mixed_documents());
    let replay = execute(&request, &with_future).expect("historical replay");
    assert_eq!(replay.artifact, baseline.artifact);
    assert_eq!(replay.terminal_result, baseline.terminal_result);
}

#[test]
fn cross_snapshot_document_fails_before_aggregation() {
    let request = request();
    let mut documents = mixed_documents();
    documents.push(
        CitationEdgeDocument::new(
            "other-snapshot-edge",
            ProvenanceKind::Citation,
            "other-snapshot",
            available("2026-07-31T23:30:00Z"),
        )
        .expect("cross-snapshot document"),
    );
    assert_eq!(
        execute(&request, &documents),
        Err(AnalysisEngineError::InvalidEvidence)
    );
}

#[test]
fn raw_census_bound_precedes_identity_allocation_and_duplicate_checks() {
    let request = request();
    let repeated = document(
        "repeated",
        ProvenanceKind::Citation,
        "2026-07-31T22:00:00Z",
    );
    let documents = vec![repeated; MAX_EVIDENCE_UNITS + 1];
    assert_eq!(
        execute(&request, &documents),
        Err(AnalysisEngineError::LimitExceeded)
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
        document(
            "cite-a",
            ProvenanceKind::Citation,
            "2026-07-31T22:00:00Z",
        ),
        document(
            "cite-b",
            ProvenanceKind::Citation,
            "2026-07-31T23:00:00Z",
        ),
    ];
    assert_eq!(
        execute(&request, &citation_only),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    let duplicates = vec![
        document(
            "same",
            ProvenanceKind::Citation,
            "2026-07-31T22:00:00Z",
        ),
        document(
            "same",
            ProvenanceKind::Revision,
            "2026-07-31T23:00:00Z",
        ),
    ];
    assert_eq!(
        execute(&request, &duplicates),
        Err(AnalysisEngineError::DuplicateEvidence)
    );
    assert_eq!(
        CitationEdgeDocument::new(
            "",
            ProvenanceKind::Citation,
            "snapshot-citation-edge",
            available("2026-07-31T22:00:00Z"),
        ),
        Err(AnalysisEngineError::InvalidEvidence)
    );
    assert_eq!(
        CitationEdgeDocument::new(
            "cite-a",
            ProvenanceKind::Citation,
            "",
            available("2026-07-31T22:00:00Z"),
        ),
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
