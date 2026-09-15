//! Trust-state contract for source-artifact wire input.

use evidence_core::{ContentDigest, EvidenceId, ValidatedSourceArtifactWire, WIRE_SCHEMA_VERSION};

fn canonical_wire(content: &[u8], caller_selected_id: EvidenceId) -> String {
    let digest = ContentDigest::sha256(content);
    let content_bytes = serde_json::to_string(content).expect("content bytes must serialize");
    format!(
        "{{\"schema_version\":{WIRE_SCHEMA_VERSION},\"artifact_id\":\"{caller_selected_id}\",\"content_sha256\":\"{digest}\",\"content_bytes\":{content_bytes}}}"
    )
}

#[test]
fn caller_selected_identity_remains_validated_wire_not_owner_artifact() {
    let content = b"caller-supplied source artifact";
    let caller_selected_id = EvidenceId::new();
    let payload = canonical_wire(content, caller_selected_id);

    let validated = ValidatedSourceArtifactWire::from_json(&payload)
        .expect("canonical source-artifact wire must validate structurally");

    assert_eq!(validated.artifact_id(), caller_selected_id);
    assert_eq!(validated.content_digest(), ContentDigest::sha256(content));
    assert_eq!(validated.content(), content);
    assert_eq!(validated.byte_length(), content.len());
    assert!(validated.verify_content(content));
    assert!(!validated.verify_content(b"different bytes"));
}

#[test]
fn source_artifact_wire_requires_canonical_json() {
    let content = b"source";
    let caller_selected_id = EvidenceId::new();
    let payload = canonical_wire(content, caller_selected_id);
    let noncanonical = format!(" {payload}");

    assert_eq!(
        ValidatedSourceArtifactWire::from_json(&noncanonical).unwrap_err(),
        evidence_core::EvidenceError::InvalidWirePayload
    );
}
