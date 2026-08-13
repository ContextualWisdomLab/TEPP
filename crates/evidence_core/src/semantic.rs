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

/// Refuse collapsing a multi-paragraph document into one bag-of-words unit.
///
/// # Errors
///
/// Returns [`EvidenceError::InvalidWirePayload`] for an empty unit set or a
/// zero required count. Returns [`EvidenceError::SemanticUnitBagRefused`] when
/// `units` is shorter than the known paragraph multiplicity.
pub fn refuse_document_bag_of_words(
    units: &[SourceSpan],
    required_paragraph_count: usize,
) -> Result<(), EvidenceError> {
    if units.is_empty() || required_paragraph_count == 0 {
        return Err(EvidenceError::InvalidWirePayload);
    }
    if units.len() < required_paragraph_count {
        return Err(EvidenceError::SemanticUnitBagRefused);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{refuse_document_bag_of_words, semantic_paragraph_units};
    use crate::{DocumentRecord, EvidenceError, SourceArtifact};

    #[test]
    fn trailing_blank_and_zero_required_count_fail_closed() {
        let text = "Only one unit.\n\n   \n\n";
        let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
        let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
        let units = semantic_paragraph_units(&document).expect("trim empty");
        assert_eq!(units.len(), 1);
        assert_eq!(
            refuse_document_bag_of_words(&units, 0),
            Err(EvidenceError::InvalidWirePayload)
        );
    }
}
