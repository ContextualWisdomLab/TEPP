//! Trust-boundary contracts for owner-created documents and generic document JSON.

use evidence_core::{
    ContentDigest, DocumentRecord, EvidenceId, SourceArtifact, ValidatedDocumentRecordWire,
    WIRE_SCHEMA_VERSION,
};

#[test]
fn owner_document_binds_the_exact_owner_issued_source_handle() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let document = DocumentRecord::from_text(artifact.id(), "document")
        .expect("trusted source handle must create a document");

    assert_eq!(artifact.id(), document.source_artifact_id());
    assert_eq!(document.source_artifact_id(), artifact.id());
    assert_eq!(
        document.content_digest(),
        ContentDigest::sha256(b"document")
    );
    assert_eq!(document.text(), "document");
}

#[test]
fn caller_selected_document_identity_remains_validated_wire_not_owner_state() {
    let caller_document_id = EvidenceId::new();
    let caller_source_artifact_id = EvidenceId::new();
    let digest = ContentDigest::sha256(b"document");
    let payload = format!(
        "{{\"schema_version\":{WIRE_SCHEMA_VERSION},\"document_id\":\"{caller_document_id}\",\"source_artifact_id\":\"{caller_source_artifact_id}\",\"content_sha256\":\"{digest}\",\"text\":\"document\"}}"
    );

    let validated = ValidatedDocumentRecordWire::from_json(&payload)
        .expect("canonical caller JSON may be structurally valid without becoming owner state");

    assert_eq!(validated.document_id(), caller_document_id);
    assert_eq!(validated.source_artifact_id(), caller_source_artifact_id);
    assert_eq!(validated.content_digest(), digest);
    assert_eq!(validated.text(), "document");
    assert_eq!(validated.byte_length(), 8);
    assert_eq!(validated.scalar_length(), 8);
    assert!(validated.verify_text("document"));
    assert!(!validated.verify_text("different"));
}
