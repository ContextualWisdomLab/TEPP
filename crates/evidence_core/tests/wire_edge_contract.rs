//! Edge-case contracts for strict evidence JSON reconstruction.

use evidence_core::{DocumentRecord, EvidenceError, SourceArtifact, SourceSpan};
use serde_json::{Value, json};

fn replace_field(serialized: &str, field: &str, replacement: Value) -> String {
    let mut value: Value = serde_json::from_str(serialized).expect("wire JSON must parse");
    value[field] = replacement;
    serde_json::to_string(&value).expect("tampered JSON must serialize")
}

#[test]
fn artifact_wire_rejects_invalid_identifiers_digests_and_empty_content() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let serialized = artifact.to_wire_json().expect("artifact must serialize");

    let malformed_identifier = replace_field(&serialized, "artifact_id", json!("not-a-uuid"));
    assert_eq!(
        SourceArtifact::from_wire_json(&malformed_identifier).unwrap_err(),
        EvidenceError::InvalidEvidenceId
    );

    let wrong_uuid_version = replace_field(
        &serialized,
        "artifact_id",
        json!("550e8400-e29b-41d4-a716-446655440000"),
    );
    assert_eq!(
        SourceArtifact::from_wire_json(&wrong_uuid_version).unwrap_err(),
        EvidenceError::InvalidEvidenceId
    );

    let malformed_digest = replace_field(&serialized, "content_sha256", json!("00"));
    assert_eq!(
        SourceArtifact::from_wire_json(&malformed_digest).unwrap_err(),
        EvidenceError::InvalidContentDigest
    );

    let empty_content = replace_field(&serialized, "content_bytes", json!([]));
    assert_eq!(
        SourceArtifact::from_wire_json(&empty_content).unwrap_err(),
        EvidenceError::EmptySourceArtifact
    );

    let invalid_byte = replace_field(&serialized, "content_bytes", json!([256]));
    assert_eq!(
        SourceArtifact::from_wire_json(&invalid_byte).unwrap_err(),
        EvidenceError::InvalidWirePayload
    );
}

#[test]
fn document_wire_rejects_invalid_identifiers_digests_and_empty_text() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let document =
        DocumentRecord::from_text(artifact.id(), "document").expect("document must be valid");
    let serialized = document.to_wire_json().expect("document must serialize");

    let malformed_document = replace_field(&serialized, "document_id", json!("invalid"));
    assert_eq!(
        DocumentRecord::from_wire_json(&malformed_document).unwrap_err(),
        EvidenceError::InvalidEvidenceId
    );

    let malformed_source = replace_field(&serialized, "source_artifact_id", json!("invalid"));
    assert_eq!(
        DocumentRecord::from_wire_json(&malformed_source).unwrap_err(),
        EvidenceError::InvalidEvidenceId
    );

    let malformed_digest = replace_field(&serialized, "content_sha256", json!("invalid"));
    assert_eq!(
        DocumentRecord::from_wire_json(&malformed_digest).unwrap_err(),
        EvidenceError::InvalidContentDigest
    );

    let empty_text = replace_field(&serialized, "text", json!(""));
    assert_eq!(
        DocumentRecord::from_wire_json(&empty_text).unwrap_err(),
        EvidenceError::EmptyDocument
    );
}

#[test]
fn source_span_wire_rejects_malformed_document_identifier() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let document =
        DocumentRecord::from_text(artifact.id(), "document").expect("document must be valid");
    let span = SourceSpan::new(&document, 0, 8, 0, 8, None).expect("span must be valid");
    let serialized = span.to_wire_json().expect("span must serialize");
    let malformed = replace_field(&serialized, "document_id", json!("invalid"));

    assert_eq!(
        SourceSpan::from_wire_json(&malformed, &document).unwrap_err(),
        EvidenceError::InvalidEvidenceId
    );
}

#[test]
fn wire_validation_errors_have_stable_redacted_messages() {
    let cases = [
        (
            EvidenceError::InvalidWirePayload,
            "invalid evidence wire payload",
        ),
        (
            EvidenceError::UnsupportedWireVersion,
            "unsupported evidence wire version",
        ),
        (
            EvidenceError::ContentDigestMismatch,
            "wire content does not match its declared digest",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
