//! Validation-branch contracts for corpus-background artifacts.

use analysis_engine::{
    AnalysisEngineError, CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION, CorpusBackgroundArtifact,
};

fn artifact() -> CorpusBackgroundArtifact {
    CorpusBackgroundArtifact {
        schema_version: CORPUS_BACKGROUND_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-corpus-background-validation".into(),
        snapshot_id: "snapshot-corpus-background".into(),
        knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
        document_count: 3,
        unique_content_count: 1,
        corpus_background_count: 2,
        refused_as_unique_content_count: 2,
        refused_as_stopword_deletion_count: 2,
        inference_status: "corpus_background_is_not_unique_content_not_stopword_deletion".into(),
    }
}

#[test]
fn independent_refusal_count_tampering_fails_closed() {
    let mut unique_refusal = artifact();
    unique_refusal.refused_as_unique_content_count = 1;
    assert_eq!(
        unique_refusal.to_json(),
        Err(AnalysisEngineError::InvalidCorpusBackgroundArtifact)
    );

    let mut stopword_refusal = artifact();
    stopword_refusal.refused_as_stopword_deletion_count = 1;
    assert_eq!(
        stopword_refusal.to_json(),
        Err(AnalysisEngineError::InvalidCorpusBackgroundArtifact)
    );
}

#[test]
fn overflowing_kind_sum_fails_closed() {
    let mut overflow = artifact();
    overflow.unique_content_count = u64::MAX;
    overflow.corpus_background_count = 1;
    overflow.refused_as_unique_content_count = 1;
    overflow.refused_as_stopword_deletion_count = 1;

    assert_eq!(
        overflow.to_json(),
        Err(AnalysisEngineError::InvalidCorpusBackgroundArtifact)
    );
}
