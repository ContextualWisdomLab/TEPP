//! Historical-cutoff contracts for the corpus-background analysis profile.

use analysis_engine::{
    CORPUS_BACKGROUND_MODEL_CONTRACT_VERSION, CORPUS_BACKGROUND_OUTPUT_PROFILE,
    CorpusBackgroundDocument, execute_corpus_background_run,
};
use corpus_background::CorpusBackgroundKind;
use temporal_core::KnowledgeCutoff;
use tepp_api::{AnalysisRunAccepted, AnalysisRunRequest};

fn request(cutoff: &str) -> AnalysisRunRequest {
    AnalysisRunRequest {
        contract_version: 1,
        idempotency_key: "corpus-background-cutoff".into(),
        tenant_workspace_id: "tenant-workspace".into(),
        snapshot_id: "snapshot-corpus-background".into(),
        knowledge_cutoff: cutoff.into(),
        model_contract_version: CORPUS_BACKGROUND_MODEL_CONTRACT_VERSION.into(),
        output_profile: CORPUS_BACKGROUND_OUTPUT_PROFILE.into(),
    }
}

fn accepted(request: &AnalysisRunRequest) -> AnalysisRunAccepted {
    AnalysisRunAccepted::new("run-corpus-background-cutoff", "accepted", &request.idempotency_key)
        .expect("accepted")
}

fn documents() -> Vec<CorpusBackgroundDocument> {
    vec![
        CorpusBackgroundDocument::new("unique-a", CorpusBackgroundKind::UniqueContent)
            .expect("unique"),
        CorpusBackgroundDocument::new("background-b", CorpusBackgroundKind::CorpusBackground)
            .expect("background"),
    ]
}

#[test]
fn equivalent_rfc3339_cutoff_spellings_bind_to_the_same_instant() {
    let request = request("2026-08-01T01:00:00+01:00");
    let execution = execute_corpus_background_run(
        &request,
        &accepted(&request),
        "snapshot-corpus-background",
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        &documents(),
        "2026-08-02T00:00:00Z",
    )
    .expect("equivalent cutoff instant must be accepted");

    assert_eq!(execution.artifact.knowledge_cutoff, "2026-08-01T00:00:00Z");
}

#[test]
fn terminal_validation_status_is_not_the_domain_inference_claim() {
    let request = request("2026-08-01T00:00:00Z");
    let execution = execute_corpus_background_run(
        &request,
        &accepted(&request),
        "snapshot-corpus-background",
        KnowledgeCutoff::parse_rfc3339("2026-08-01T00:00:00Z").expect("cutoff"),
        &documents(),
        "2026-08-02T00:00:00Z",
    )
    .expect("execution");

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
        execution.artifact.inference_status,
        "corpus_background_is_not_unique_content_not_stopword_deletion"
    );
}
