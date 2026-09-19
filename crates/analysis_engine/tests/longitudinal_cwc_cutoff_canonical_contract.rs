use analysis_engine::{
    AnalysisEngineError, LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION, LongitudinalCwcArtifact,
};

fn artifact(knowledge_cutoff: &str) -> LongitudinalCwcArtifact {
    LongitudinalCwcArtifact {
        schema_version: LONGITUDINAL_CWC_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: "run-cutoff-canonical".into(),
        snapshot_id: "snapshot-cutoff-canonical".into(),
        knowledge_cutoff: knowledge_cutoff.into(),
        admitted_evidence_sha256:
            "0000000000000000000000000000000000000000000000000000000000000000".into(),
        row_count: 4,
        cluster_count: 2,
        within_slope: 0.5,
        between_slope: 2.0,
        contextual_effect: 1.5,
        inference_status: "composed_cwc_slopes_not_causal".into(),
    }
}

#[test]
fn semantically_equivalent_noncanonical_cutoff_text_fails_closed() {
    let canonical = artifact("2026-08-01T00:00:00Z");
    assert!(canonical.to_json().is_ok());

    let offset_spelling = artifact("2026-08-01T09:00:00+09:00");
    assert_eq!(
        offset_spelling.to_json(),
        Err(AnalysisEngineError::InvalidLongitudinalCwcArtifact)
    );
}
