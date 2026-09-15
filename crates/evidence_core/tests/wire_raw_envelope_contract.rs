//! Raw-wire envelope bounds must be enforced before JSON deserialization allocates payload fields.

use evidence_core::{
    DocumentRecord, EvidenceError, SourceArtifact, ValidatedDocumentRecordWire,
    ValidatedSourceArtifactWire,
};

#[test]
fn source_artifact_raw_wire_limit_precedes_content_vector_validation() {
    let artifact = SourceArtifact::from_bytes(&vec![255_u8; 100]).expect("artifact");
    let serialized = artifact.to_wire_json().expect("wire JSON");

    assert_eq!(
        ValidatedSourceArtifactWire::from_json_with_limit(&serialized, 1).unwrap_err(),
        EvidenceError::InvalidWirePayload
    );
}

#[test]
fn document_raw_wire_limit_precedes_text_validation() {
    let text = "\u{0001}".repeat(100);
    let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
    let document = DocumentRecord::from_text(artifact.id(), &text).expect("document");
    let serialized = document.to_wire_json().expect("wire JSON");

    assert_eq!(
        ValidatedDocumentRecordWire::from_json_with_limit(&serialized, 1).unwrap_err(),
        EvidenceError::InvalidWirePayload
    );
}
