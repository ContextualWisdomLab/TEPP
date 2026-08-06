//! Versioned, fail-closed JSON wire contracts for immutable evidence records.

use evidence_core::{
    DocumentRecord, EvidenceError, PageLocation, SourceArtifact, SourceSpan,
    WIRE_SCHEMA_VERSION,
};
use serde_json::{Value, json};

fn artifact_and_document(text: &str) -> (SourceArtifact, DocumentRecord) {
    let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact must be valid");
    let document = DocumentRecord::from_text(artifact.id(), text).expect("document must be valid");
    (artifact, document)
}

fn replace_field(serialized: &str, field: &str, replacement: Value) -> String {
    let mut value: Value = serde_json::from_str(serialized).expect("wire JSON must parse");
    value[field] = replacement;
    serde_json::to_string(&value).expect("tampered JSON must serialize")
}

#[test]
fn source_artifact_wire_round_trip_preserves_identity_digest_and_bytes() {
    let artifact = SourceArtifact::from_bytes(b"\x00source\xff").expect("artifact must be valid");
    let serialized = artifact.to_wire_json().expect("artifact must serialize");
    let value: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    let restored = SourceArtifact::from_wire_json(&serialized).expect("wire record must validate");

    assert_eq!(value["schema_version"], json!(WIRE_SCHEMA_VERSION));
    assert_eq!(value["artifact_id"], json!(artifact.id().to_string()));
    assert_eq!(
        value["content_sha256"],
        json!(artifact.content_digest().to_string())
    );
    assert_eq!(value["content_bytes"], json!([0, 115, 111, 117, 114, 99, 101, 255]));
    assert_eq!(restored, artifact);
}

#[test]
fn source_artifact_wire_rejects_unknown_version_fields_and_digest_mismatch() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let serialized = artifact.to_wire_json().expect("artifact must serialize");

    let unsupported = replace_field(&serialized, "schema_version", json!(2));
    assert_eq!(
        SourceArtifact::from_wire_json(&unsupported).unwrap_err(),
        EvidenceError::UnsupportedWireVersion
    );

    let mut unknown: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    unknown["unexpected"] = json!(true);
    assert_eq!(
        SourceArtifact::from_wire_json(&unknown.to_string()).unwrap_err(),
        EvidenceError::InvalidWirePayload
    );

    let mismatch = replace_field(
        &serialized,
        "content_sha256",
        json!("00".repeat(32)),
    );
    assert_eq!(
        SourceArtifact::from_wire_json(&mismatch).unwrap_err(),
        EvidenceError::ContentDigestMismatch
    );

    assert_eq!(
        SourceArtifact::from_wire_json("not JSON").unwrap_err(),
        EvidenceError::InvalidWirePayload
    );
}

#[test]
fn source_artifact_wire_reapplies_content_limits() {
    let artifact = SourceArtifact::from_bytes(b"four").expect("artifact must be valid");
    let serialized = artifact.to_wire_json().expect("artifact must serialize");

    assert_eq!(
        SourceArtifact::from_wire_json_with_limit(&serialized, 3).unwrap_err(),
        EvidenceError::SourceArtifactTooLarge
    );
    assert_eq!(
        SourceArtifact::from_wire_json_with_limit(&serialized, 4)
            .expect("boundary size must be valid"),
        artifact
    );
}

#[test]
fn document_wire_round_trip_preserves_identity_source_digest_and_unicode() {
    let (_, document) = artifact_and_document("Aé🧠Z");
    let serialized = document.to_wire_json().expect("document must serialize");
    let value: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    let restored = DocumentRecord::from_wire_json(&serialized).expect("wire record must validate");

    assert_eq!(value["schema_version"], json!(WIRE_SCHEMA_VERSION));
    assert_eq!(value["document_id"], json!(document.id().to_string()));
    assert_eq!(
        value["source_artifact_id"],
        json!(document.source_artifact_id().to_string())
    );
    assert_eq!(
        value["content_sha256"],
        json!(document.content_digest().to_string())
    );
    assert_eq!(value["text"], json!("Aé🧠Z"));
    assert_eq!(restored, document);
}

#[test]
fn document_wire_rejects_unknown_version_fields_digest_mismatch_and_limits() {
    let (_, document) = artifact_and_document("four");
    let serialized = document.to_wire_json().expect("document must serialize");

    let unsupported = replace_field(&serialized, "schema_version", json!(9));
    assert_eq!(
        DocumentRecord::from_wire_json(&unsupported).unwrap_err(),
        EvidenceError::UnsupportedWireVersion
    );

    let mut unknown: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    unknown["private_scalar_length"] = json!(4);
    assert_eq!(
        DocumentRecord::from_wire_json(&unknown.to_string()).unwrap_err(),
        EvidenceError::InvalidWirePayload
    );

    let mismatch = replace_field(
        &serialized,
        "content_sha256",
        json!("00".repeat(32)),
    );
    assert_eq!(
        DocumentRecord::from_wire_json(&mismatch).unwrap_err(),
        EvidenceError::ContentDigestMismatch
    );
    assert_eq!(
        DocumentRecord::from_wire_json_with_limit(&serialized, 3).unwrap_err(),
        EvidenceError::DocumentTooLarge
    );
    assert_eq!(
        DocumentRecord::from_wire_json_with_limit(&serialized, 4)
            .expect("boundary size must be valid"),
        document
    );
    assert_eq!(
        DocumentRecord::from_wire_json("[").unwrap_err(),
        EvidenceError::InvalidWirePayload
    );
}

#[test]
fn source_span_wire_round_trip_revalidates_exact_coordinates_and_page_location() {
    let (_, document) = artifact_and_document("Aé🧠Z");
    let location = PageLocation::new(2, 100.0, 200.0, 10.0, 20.0, 30.0, 40.0)
        .expect("page location must be valid");
    let span = SourceSpan::new(&document, 1, 7, 1, 3, Some(location))
        .expect("source span must be valid");
    let serialized = span.to_wire_json().expect("span must serialize");
    let value: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    let restored =
        SourceSpan::from_wire_json(&serialized, &document).expect("wire span must validate");

    assert_eq!(value["schema_version"], json!(WIRE_SCHEMA_VERSION));
    assert_eq!(value["document_id"], json!(document.id().to_string()));
    assert_eq!(value["byte_start"], json!(1));
    assert_eq!(value["byte_end"], json!(7));
    assert_eq!(value["scalar_start"], json!(1));
    assert_eq!(value["scalar_end"], json!(3));
    assert_eq!(value["page_location"]["page_number"], json!(2));
    assert_eq!(restored, span);
    assert_eq!(restored.text(&document), Ok("é🧠"));
}

#[test]
fn source_span_wire_rejects_wrong_document_unknown_fields_versions_and_invalid_ranges() {
    let (_, document) = artifact_and_document("Aé🧠Z");
    let (_, other) = artifact_and_document("other");
    let span = SourceSpan::new(&document, 1, 7, 1, 3, None).expect("span must be valid");
    let serialized = span.to_wire_json().expect("span must serialize");

    assert_eq!(
        SourceSpan::from_wire_json(&serialized, &other).unwrap_err(),
        EvidenceError::SpanDocumentMismatch
    );

    let unsupported = replace_field(&serialized, "schema_version", json!(0));
    assert_eq!(
        SourceSpan::from_wire_json(&unsupported, &document).unwrap_err(),
        EvidenceError::UnsupportedWireVersion
    );

    let mut unknown: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    unknown["unexpected"] = json!(true);
    assert_eq!(
        SourceSpan::from_wire_json(&unknown.to_string(), &document).unwrap_err(),
        EvidenceError::InvalidWirePayload
    );

    let invalid_boundary = replace_field(&serialized, "byte_start", json!(2));
    assert_eq!(
        SourceSpan::from_wire_json(&invalid_boundary, &document).unwrap_err(),
        EvidenceError::InvalidUtf8Boundary
    );

    assert_eq!(
        SourceSpan::from_wire_json("{}", &document).unwrap_err(),
        EvidenceError::InvalidWirePayload
    );
}

#[test]
fn page_location_wire_rejects_unknown_nested_fields_and_invalid_geometry() {
    let (_, document) = artifact_and_document("page");
    let location = PageLocation::new(1, 100.0, 100.0, 0.0, 0.0, 10.0, 10.0)
        .expect("page location must be valid");
    let span = SourceSpan::new(&document, 0, 4, 0, 4, Some(location))
        .expect("span must be valid");
    let serialized = span.to_wire_json().expect("span must serialize");
    let mut unknown: Value = serde_json::from_str(&serialized).expect("wire JSON must parse");
    unknown["page_location"]["rotation"] = json!(90);

    assert_eq!(
        SourceSpan::from_wire_json(&unknown.to_string(), &document).unwrap_err(),
        EvidenceError::InvalidWirePayload
    );

    let mut out_of_bounds: Value =
        serde_json::from_str(&serialized).expect("wire JSON must parse");
    out_of_bounds["page_location"]["width"] = json!(101.0);
    assert_eq!(
        SourceSpan::from_wire_json(&out_of_bounds.to_string(), &document).unwrap_err(),
        EvidenceError::LayoutOutOfBounds
    );
}

#[test]
fn generated_unicode_spans_round_trip_and_invalid_coordinates_fail_closed() {
    let corpus = ["ascii", "Aé🧠Z", "한글과日本語", "e\u{301}lan"];

    for text in corpus {
        let (_, document) = artifact_and_document(text);
        let mut boundaries = document
            .text()
            .char_indices()
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        boundaries.push(document.byte_length());

        for start_index in 0..boundaries.len() - 1 {
            for end_index in start_index + 1..boundaries.len() {
                let byte_start = boundaries[start_index];
                let byte_end = boundaries[end_index];
                let span = SourceSpan::new(
                    &document,
                    byte_start,
                    byte_end,
                    start_index,
                    end_index,
                    None,
                )
                .expect("generated valid span must validate");
                let serialized = span.to_wire_json().expect("span must serialize");
                let restored = SourceSpan::from_wire_json(&serialized, &document)
                    .expect("generated wire span must validate");

                assert_eq!(restored, span);
                assert_eq!(
                    restored.text(&document).expect("text must be available"),
                    &text[byte_start..byte_end]
                );

                let mismatched = replace_field(
                    &serialized,
                    "scalar_start",
                    json!(start_index.saturating_add(1)),
                );
                assert!(matches!(
                    SourceSpan::from_wire_json(&mismatched, &document),
                    Err(EvidenceError::EmptySourceSpan)
                        | Err(EvidenceError::InvalidSourceSpanOrder)
                        | Err(EvidenceError::CharacterRangeMismatch)
                ));
            }
        }

        for byte_index in 1..document.byte_length() {
            if document.text().is_char_boundary(byte_index) {
                continue;
            }
            assert_eq!(
                SourceSpan::new(&document, byte_index, document.byte_length(), 0, 1, None)
                    .unwrap_err(),
                EvidenceError::InvalidUtf8Boundary
            );
        }
    }
}
