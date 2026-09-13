//! Worst-case escaping contract for the bounded citation-edge artifact wire.

use analysis_engine::{
    CITATION_EDGE_ARTIFACT_BYTE_LIMIT, CITATION_EDGE_ARTIFACT_SCHEMA_VERSION,
    CitationEdgeArtifact, MAX_ANALYSIS_IDENTIFIER_BYTES, MAX_EVIDENCE_UNITS,
};

#[test]
fn maximum_valid_identifier_escaping_stays_below_input_wire_limit() {
    let maximum_count = u64::try_from(MAX_EVIDENCE_UNITS).expect("bounded census");
    let escaped_identifier = "\\".repeat(MAX_ANALYSIS_IDENTIFIER_BYTES);
    let artifact = CitationEdgeArtifact {
        schema_version: CITATION_EDGE_ARTIFACT_SCHEMA_VERSION.into(),
        run_id: escaped_identifier.clone(),
        snapshot_id: escaped_identifier,
        knowledge_cutoff: "2026-08-01T00:00:00.123456789+14:00".into(),
        document_count: maximum_count,
        citation_count: maximum_count - 1,
        translation_count: 0,
        revision_count: 1,
        retrospective_report_count: 0,
        refused_as_transition_count: maximum_count,
        distinct_kind_count: 2,
        inference_status: "provenance_is_not_a_state_transition".into(),
    };

    let payload = artifact.to_json().expect("worst escaping artifact");
    assert!(payload.len() < CITATION_EDGE_ARTIFACT_BYTE_LIMIT);
    assert_eq!(CitationEdgeArtifact::from_json(&payload), Ok(artifact));
}
