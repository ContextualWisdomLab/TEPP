//! Behavioral contracts for immutable artifacts, text records, and exact spans.

use evidence_core::{
    ContentDigest, DocumentRecord, EvidenceError, PageLocation, SourceArtifact, SourceSpan,
};
use std::str::FromStr;

const ALPHA_SHA256: &str = "8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8";
const UNICODE_SHA256: &str = "eaf608e9cf297c54599c2731f265dfbaf14d0f4784b4db8b9c72a1b28527bb8b";

#[test]
fn source_artifact_copies_bytes_and_detects_mutation() {
    let mut original = b"alpha".to_vec();
    let artifact = SourceArtifact::from_bytes(&original).expect("artifact must be valid");
    let digest = artifact.content_digest();

    original[0] = b'X';

    assert_eq!(artifact.content(), b"alpha");
    assert_eq!(artifact.byte_length(), 5);
    assert_eq!(digest.to_string(), ALPHA_SHA256);
    assert_eq!(ContentDigest::from_str(ALPHA_SHA256), Ok(digest));
    assert_eq!(digest.as_bytes().len(), 32);
    assert!(artifact.verify_content(b"alpha"));
    assert!(!artifact.verify_content(&original));
    assert_eq!(artifact.id().as_uuid().get_version_num(), 7);
}

#[test]
fn source_artifact_rejects_empty_and_oversized_content() {
    assert_eq!(
        SourceArtifact::from_bytes(b"").unwrap_err(),
        EvidenceError::EmptySourceArtifact
    );
    assert_eq!(
        SourceArtifact::from_bytes_with_limit(b"four", 3).unwrap_err(),
        EvidenceError::SourceArtifactTooLarge
    );
    assert!(SourceArtifact::from_bytes_with_limit(b"four", 4).is_ok());
}

#[test]
fn content_digest_parsing_is_exact_and_canonical() {
    let uppercase = ALPHA_SHA256.to_uppercase();
    let digest = ContentDigest::from_str(&uppercase).expect("uppercase hexadecimal is valid");
    let non_ascii = "é".repeat(32);
    let invalid_second_nibble = format!("0z{}", "00".repeat(31));
    let invalid_last_pair = format!("{}zz", "00".repeat(31));

    assert_eq!(digest.to_string(), ALPHA_SHA256);
    assert_eq!(
        ContentDigest::from_str("00").unwrap_err(),
        EvidenceError::InvalidContentDigest
    );
    assert_eq!(
        ContentDigest::from_str(&non_ascii).unwrap_err(),
        EvidenceError::InvalidContentDigest
    );
    assert_eq!(
        ContentDigest::from_str(&invalid_second_nibble).unwrap_err(),
        EvidenceError::InvalidContentDigest
    );
    assert_eq!(
        ContentDigest::from_str(&invalid_last_pair).unwrap_err(),
        EvidenceError::InvalidContentDigest
    );
    assert_eq!(
        ContentDigest::from_str("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz")
            .unwrap_err(),
        EvidenceError::InvalidContentDigest
    );
}

#[test]
fn document_record_preserves_unicode_and_verifies_text() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let document =
        DocumentRecord::from_text(artifact.id(), "Aé🧠Z").expect("document must be valid");

    assert_eq!(document.source_artifact_id(), artifact.id());
    assert_eq!(document.text(), "Aé🧠Z");
    assert_eq!(document.byte_length(), 8);
    assert_eq!(document.scalar_length(), 4);
    assert_eq!(document.content_digest().to_string(), UNICODE_SHA256);
    assert!(document.verify_text("Aé🧠Z"));
    assert!(!document.verify_text("AéZ"));
    assert_eq!(document.id().as_uuid().get_version_num(), 7);
}

#[test]
fn document_record_rejects_empty_and_oversized_text() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");

    assert_eq!(
        DocumentRecord::from_text(artifact.id(), "").unwrap_err(),
        EvidenceError::EmptyDocument
    );
    assert_eq!(
        DocumentRecord::from_text_with_limit(artifact.id(), "éé", 3).unwrap_err(),
        EvidenceError::DocumentTooLarge
    );
    assert!(DocumentRecord::from_text_with_limit(artifact.id(), "éé", 4).is_ok());
}

#[test]
fn source_span_preserves_exact_byte_and_scalar_coordinates() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let document =
        DocumentRecord::from_text(artifact.id(), "Aé🧠Z").expect("document must be valid");
    let span = SourceSpan::new(&document, 1, 7, 1, 3, None).expect("span must be exact");

    assert_eq!(span.document_id(), document.id());
    assert_eq!(span.byte_start(), 1);
    assert_eq!(span.byte_end(), 7);
    assert_eq!(span.scalar_start(), 1);
    assert_eq!(span.scalar_end(), 3);
    assert_eq!(span.page_location(), None);
    assert_eq!(span.text(&document), Ok("é🧠"));
}

#[test]
fn source_span_rejects_empty_reversed_and_out_of_bounds_ranges() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let document =
        DocumentRecord::from_text(artifact.id(), "Aé🧠Z").expect("document must be valid");

    assert_eq!(
        SourceSpan::new(&document, 1, 1, 1, 2, None).unwrap_err(),
        EvidenceError::EmptySourceSpan
    );
    assert_eq!(
        SourceSpan::new(&document, 1, 3, 1, 1, None).unwrap_err(),
        EvidenceError::EmptySourceSpan
    );
    assert_eq!(
        SourceSpan::new(&document, 7, 1, 1, 3, None).unwrap_err(),
        EvidenceError::InvalidSourceSpanOrder
    );
    assert_eq!(
        SourceSpan::new(&document, 1, 7, 3, 1, None).unwrap_err(),
        EvidenceError::InvalidSourceSpanOrder
    );
    assert_eq!(
        SourceSpan::new(&document, 1, 9, 1, 4, None).unwrap_err(),
        EvidenceError::ByteRangeOutOfBounds
    );
    assert_eq!(
        SourceSpan::new(&document, 1, 7, 1, 5, None).unwrap_err(),
        EvidenceError::ByteRangeOutOfBounds
    );
}

#[test]
fn source_span_rejects_invalid_boundaries_coordinates_and_documents() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let document =
        DocumentRecord::from_text(artifact.id(), "Aé🧠Z").expect("document must be valid");
    let other = DocumentRecord::from_text(artifact.id(), "other").expect("document must be valid");

    assert_eq!(
        SourceSpan::new(&document, 2, 7, 1, 3, None).unwrap_err(),
        EvidenceError::InvalidUtf8Boundary
    );
    assert_eq!(
        SourceSpan::new(&document, 1, 2, 1, 2, None).unwrap_err(),
        EvidenceError::InvalidUtf8Boundary
    );
    assert_eq!(
        SourceSpan::new(&document, 1, 7, 0, 3, None).unwrap_err(),
        EvidenceError::CharacterRangeMismatch
    );
    assert_eq!(
        SourceSpan::new(&document, 1, 7, 1, 2, None).unwrap_err(),
        EvidenceError::CharacterRangeMismatch
    );

    let span = SourceSpan::new(&document, 1, 7, 1, 3, None).expect("span must be exact");
    assert_eq!(
        span.text(&other).unwrap_err(),
        EvidenceError::SpanDocumentMismatch
    );
}

#[test]
fn page_location_validates_finite_in_page_bounds() {
    let location = PageLocation::new(2, 100.0, 200.0, 10.0, 20.0, 30.0, 40.0)
        .expect("bounded location must be valid");

    assert_eq!(location.page_number(), 2);
    assert_eq!(location.page_width(), 100.0);
    assert_eq!(location.page_height(), 200.0);
    assert_eq!(location.x(), 10.0);
    assert_eq!(location.y(), 20.0);
    assert_eq!(location.width(), 30.0);
    assert_eq!(location.height(), 40.0);

    assert_eq!(
        PageLocation::new(0, 100.0, 200.0, 0.0, 0.0, 10.0, 10.0).unwrap_err(),
        EvidenceError::InvalidPageNumber
    );
    assert_eq!(
        PageLocation::new(1, f64::NAN, 200.0, 0.0, 0.0, 10.0, 10.0).unwrap_err(),
        EvidenceError::InvalidPageGeometry
    );
    assert_eq!(
        PageLocation::new(1, 0.0, 200.0, 0.0, 0.0, 10.0, 10.0).unwrap_err(),
        EvidenceError::InvalidPageGeometry
    );
    assert_eq!(
        PageLocation::new(1, 100.0, f64::NAN, 0.0, 0.0, 10.0, 10.0).unwrap_err(),
        EvidenceError::InvalidPageGeometry
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 0.0, 0.0, 0.0, 10.0, 10.0).unwrap_err(),
        EvidenceError::InvalidPageGeometry
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, f64::NAN, 0.0, 10.0, 10.0).unwrap_err(),
        EvidenceError::InvalidLayoutBounds
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, -1.0, 0.0, 10.0, 10.0).unwrap_err(),
        EvidenceError::InvalidLayoutBounds
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, 0.0, f64::NAN, 10.0, 10.0).unwrap_err(),
        EvidenceError::InvalidLayoutBounds
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, 0.0, -1.0, 10.0, 10.0).unwrap_err(),
        EvidenceError::InvalidLayoutBounds
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, 0.0, 0.0, f64::NAN, 10.0).unwrap_err(),
        EvidenceError::InvalidLayoutBounds
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, 0.0, 0.0, 0.0, 10.0).unwrap_err(),
        EvidenceError::InvalidLayoutBounds
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, 0.0, 0.0, 10.0, f64::NAN).unwrap_err(),
        EvidenceError::InvalidLayoutBounds
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, 0.0, 0.0, 10.0, 0.0).unwrap_err(),
        EvidenceError::InvalidLayoutBounds
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, 90.0, 0.0, 11.0, 10.0).unwrap_err(),
        EvidenceError::LayoutOutOfBounds
    );
    assert_eq!(
        PageLocation::new(1, 100.0, 200.0, 0.0, 190.0, 10.0, 11.0).unwrap_err(),
        EvidenceError::LayoutOutOfBounds
    );
}

#[test]
fn source_span_carries_validated_page_location() {
    let artifact = SourceArtifact::from_bytes(b"source").expect("artifact must be valid");
    let document =
        DocumentRecord::from_text(artifact.id(), "Aé🧠Z").expect("document must be valid");
    let location = PageLocation::new(1, 100.0, 200.0, 10.0, 20.0, 30.0, 40.0)
        .expect("bounded location must be valid");
    let span = SourceSpan::new(&document, 1, 7, 1, 3, Some(location)).expect("span must be exact");

    assert_eq!(span.page_location(), Some(location));
}

#[test]
fn every_record_validation_error_has_a_stable_message() {
    let cases = [
        (
            EvidenceError::InvalidContentDigest,
            "invalid content digest",
        ),
        (
            EvidenceError::EmptySourceArtifact,
            "source artifact is empty",
        ),
        (
            EvidenceError::SourceArtifactTooLarge,
            "source artifact exceeds the configured byte limit",
        ),
        (EvidenceError::EmptyDocument, "document text is empty"),
        (
            EvidenceError::DocumentTooLarge,
            "document text exceeds the configured byte limit",
        ),
        (EvidenceError::EmptySourceSpan, "source span is empty"),
        (
            EvidenceError::InvalidSourceSpanOrder,
            "source span coordinates are not ordered",
        ),
        (
            EvidenceError::ByteRangeOutOfBounds,
            "source span byte range is out of bounds",
        ),
        (
            EvidenceError::InvalidUtf8Boundary,
            "source span does not use UTF-8 boundaries",
        ),
        (
            EvidenceError::CharacterRangeMismatch,
            "source span character coordinates do not match its byte range",
        ),
        (
            EvidenceError::SpanDocumentMismatch,
            "source span belongs to a different document",
        ),
        (
            EvidenceError::InvalidPageNumber,
            "page number must be positive",
        ),
        (
            EvidenceError::InvalidPageGeometry,
            "page geometry must be finite and positive",
        ),
        (
            EvidenceError::InvalidLayoutBounds,
            "layout bounds must be finite, nonnegative, and nonempty",
        ),
        (
            EvidenceError::LayoutOutOfBounds,
            "layout bounds exceed the page geometry",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
