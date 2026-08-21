//! Embedded base64 images keep their original location and are not lexical text.

use evidence_core::{
    DocumentRecord, EvidenceError, SourceArtifact, embedded_image_units,
    refuse_base64_image_as_lexical_text,
};

#[test]
fn data_uri_recovers_exact_span_and_media_type() {
    let uri = "data:image/png;base64,iVBORw0KGgo=";
    let text = format!("Before the figure.\n\n{uri}\n\nAfter the figure. data:image/gif y");
    let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
    let document = DocumentRecord::from_text(artifact.id(), &text).expect("document");

    let units = embedded_image_units(&document).expect("units");
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].media_type(), "image/png");
    assert_eq!(
        &document.text()[units[0].span().byte_start()..units[0].span().byte_end()],
        uri
    );
    assert_eq!(
        refuse_base64_image_as_lexical_text(document.text()),
        Err(EvidenceError::EmbeddedImageIsNotLexicalText)
    );
    refuse_base64_image_as_lexical_text("data:image/png").expect("incomplete image");
    refuse_base64_image_as_lexical_text("Before the figure.").expect("plain text");
    refuse_base64_image_as_lexical_text("data:image/gif y").expect("incomplete image marker");
}

#[test]
fn documents_without_images_and_empty_payloads_fail_closed() {
    let text = "No figures in this note.";
    let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
    let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
    assert_eq!(
        embedded_image_units(&document),
        Err(EvidenceError::EmptySourceSpan)
    );

    assert_eq!(
        refuse_base64_image_as_lexical_text(""),
        Err(EvidenceError::InvalidWirePayload)
    );
    let empty_payload = "data:image/png;base64,";
    let empty_artifact = SourceArtifact::from_bytes(empty_payload.as_bytes()).expect("artifact");
    let empty_document =
        DocumentRecord::from_text(empty_artifact.id(), empty_payload).expect("document");
    assert_eq!(
        embedded_image_units(&empty_document),
        Err(EvidenceError::EmptySourceSpan)
    );
}
