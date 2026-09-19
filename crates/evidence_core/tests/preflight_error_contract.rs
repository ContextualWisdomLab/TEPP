//! Public contract for allocation-safe wire preflight error semantics.

use evidence_core::{
    DocumentRecord, EvidenceError, SourceArtifact, ValidatedDocumentRecordWire,
    ValidatedSourceArtifactWire,
};
use serde_json::{Value, json};

#[test]
fn source_preflight_preserves_too_large_semantics_before_vec_allocation() {
    let artifact = SourceArtifact::from_bytes(b"four").expect("artifact must be valid");
    let wire = artifact.to_wire_json().expect("artifact must serialize");

    assert_eq!(
        ValidatedSourceArtifactWire::from_json_with_limit(&wire, 3).unwrap_err(),
        EvidenceError::SourceArtifactTooLarge
    );
}

#[test]
fn document_preflight_preserves_too_large_semantics_before_string_allocation() {
    let artifact = SourceArtifact::from_bytes(b"four").expect("artifact must be valid");
    let document =
        DocumentRecord::from_text(artifact.id(), "four").expect("document must be valid");
    let wire = document.to_wire_json().expect("document must serialize");

    assert_eq!(
        ValidatedDocumentRecordWire::from_json_with_limit(&wire, 3).unwrap_err(),
        EvidenceError::DocumentTooLarge
    );
}

#[test]
fn source_preflight_validates_the_bounded_wire_before_classifying_overflow() {
    let artifact = SourceArtifact::from_bytes(b"four").expect("artifact must be valid");
    let wire = artifact.to_wire_json().expect("artifact must serialize");
    let mut payload: Value = serde_json::from_str(&wire).expect("wire must parse");
    payload["content_bytes"] = json!([0, 1, 256]);
    let malformed = serde_json::to_string(&payload).expect("tampered wire must serialize");

    assert_eq!(
        ValidatedSourceArtifactWire::from_json_with_limit(&malformed, 1).unwrap_err(),
        EvidenceError::InvalidWirePayload
    );
}
