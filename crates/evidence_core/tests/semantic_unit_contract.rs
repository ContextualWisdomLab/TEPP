//! Paragraph units keep exact source spans for later meaning search.

use evidence_core::{
    DocumentRecord, EvidenceError, SourceArtifact, refuse_document_bag_of_words,
    semantic_paragraph_units,
};

#[test]
fn two_paragraphs_recover_exact_spans_and_refuse_bag_of_words() {
    let text = "Q3 pipeline slipped after the Acme renewal stalled.\n\nLegal hold remains on the Acme folder.";
    let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
    let document = DocumentRecord::from_text(artifact.id(), text).expect("document");

    let units = semantic_paragraph_units(&document).expect("units");
    assert_eq!(units.len(), 2);
    assert_eq!(
        &document.text()[units[0].byte_start()..units[0].byte_end()],
        "Q3 pipeline slipped after the Acme renewal stalled."
    );
    assert_eq!(
        &document.text()[units[1].byte_start()..units[1].byte_end()],
        "Legal hold remains on the Acme folder."
    );
    refuse_document_bag_of_words(&document, &units).expect("keep canonical units");
    assert_eq!(
        refuse_document_bag_of_words(&document, &units[..1]),
        Err(EvidenceError::SemanticUnitBagRefused)
    );
}

#[test]
fn single_paragraph_rejects_empty_or_unrelated_spans() {
    let text = "One paragraph only.";
    let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
    let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
    let units = semantic_paragraph_units(&document).expect("one");
    assert_eq!(units.len(), 1);
    refuse_document_bag_of_words(&document, &units).expect("single canonical unit");
    assert_eq!(
        refuse_document_bag_of_words(&document, &[]),
        Err(EvidenceError::InvalidWirePayload)
    );

    let other_text = "Another paragraph.";
    let other_artifact = SourceArtifact::from_bytes(other_text.as_bytes()).expect("other artifact");
    let other_document =
        DocumentRecord::from_text(other_artifact.id(), other_text).expect("other document");
    let unrelated_units = semantic_paragraph_units(&other_document).expect("other units");
    assert_eq!(
        refuse_document_bag_of_words(&document, &unrelated_units),
        Err(EvidenceError::InvalidWirePayload)
    );
}
