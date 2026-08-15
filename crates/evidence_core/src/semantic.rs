//! Paragraph-scale semantic units with exact source spans.

use crate::{DocumentRecord, EvidenceError, SourceSpan};

/// Split a document into paragraph units with validated exact spans.
///
/// Units are separated by a blank line (`\n\n`). Whitespace-only segments are
/// skipped. This is the meaning-search unit for later embedding; it is not a
/// bag-of-words document vector.
///
/// # Errors
///
/// Returns [`EvidenceError::EmptySourceSpan`] when no non-empty paragraph
/// remains after splitting.
pub fn semantic_paragraph_units(
    document: &DocumentRecord,
) -> Result<Vec<SourceSpan>, EvidenceError> {
    let text = document.text();
    let mut units = Vec::new();
    let mut byte_cursor = 0usize;
    let mut scalar_cursor = 0usize;
    for segment in text.split("\n\n") {
        let byte_start = byte_cursor;
        let byte_end = byte_start + segment.len();
        let scalar_start = scalar_cursor;
        let scalar_end = scalar_start + segment.chars().count();
        byte_cursor = byte_end + 2;
        scalar_cursor = scalar_end + 2;
        if segment.trim().is_empty() {
            continue;
        }
        units.push(SourceSpan::new(
            document,
            byte_start,
            byte_end,
            scalar_start,
            scalar_end,
            None,
        )?);
    }
    if units.is_empty() {
        return Err(EvidenceError::EmptySourceSpan);
    }
    Ok(units)
}

/// Require the canonical paragraph spans instead of one collapsed document bag.
///
/// The expected paragraph multiplicity is derived from `document`; callers
/// cannot weaken the guard by supplying their own count. The supplied spans
/// must also equal the canonical exact spans, preventing unrelated spans from
/// satisfying the count alone.
///
/// # Errors
///
/// Returns [`EvidenceError::InvalidWirePayload`] for an empty or noncanonical
/// span set. Returns [`EvidenceError::SemanticUnitBagRefused`] when `units`
/// contains fewer spans than the document's canonical paragraph set.
pub fn refuse_document_bag_of_words(
    document: &DocumentRecord,
    units: &[SourceSpan],
) -> Result<(), EvidenceError> {
    if units.is_empty() {
        return Err(EvidenceError::InvalidWirePayload);
    }
    let expected = semantic_paragraph_units(document)?;
    if units.len() < expected.len() {
        return Err(EvidenceError::SemanticUnitBagRefused);
    }
    if units != expected.as_slice() {
        return Err(EvidenceError::InvalidWirePayload);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{refuse_document_bag_of_words, semantic_paragraph_units};
    use crate::{DocumentRecord, EvidenceError, SourceArtifact};

    #[test]
    fn trailing_blank_and_empty_unit_set_fail_closed() {
        let text = "Only one unit.\n\n   \n\n";
        let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
        let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
        let units = semantic_paragraph_units(&document).expect("trim empty");
        assert_eq!(units.len(), 1);
        refuse_document_bag_of_words(&document, &units).expect("canonical unit");
        assert_eq!(
            refuse_document_bag_of_words(&document, &[]),
            Err(EvidenceError::InvalidWirePayload)
        );
    }
}
