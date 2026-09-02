//! Semantic identifier contract for immutable evidence records.

use evidence_core::{DocumentRecord, SourceArtifact};

#[test]
fn evidence_records_expose_semantic_identifiers_with_legacy_aliases() {
    let source_artifact = SourceArtifact::from_bytes(b"source artifact")
        .expect("source artifact must be valid");
    let document_record = DocumentRecord::from_text(
        source_artifact.source_artifact_id(),
        "document record",
    )
    .expect("document record must be valid");

    assert_eq!(source_artifact.source_artifact_id(), source_artifact.id());
    assert_eq!(document_record.document_record_id(), document_record.id());
    assert_eq!(
        document_record.source_artifact_id(),
        source_artifact.source_artifact_id()
    );

    let artifact_wire = source_artifact
        .to_wire_json()
        .expect("source artifact wire JSON must serialize");
    let document_wire = document_record
        .to_wire_json()
        .expect("document record wire JSON must serialize");

    assert!(artifact_wire.contains("\"artifact_id\""));
    assert!(document_wire.contains("\"document_id\""));
    assert!(document_wire.contains("\"source_artifact_id\""));
}
