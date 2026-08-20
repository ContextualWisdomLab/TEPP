//! Embedded `data:image` units that keep their original source location.

use crate::{DocumentRecord, EvidenceError, SourceSpan};

const DATA_IMAGE_PREFIX: &str = "data:image/";
const BASE64_MARK: &str = ";base64,";

/// One embedded image located in a document body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmbeddedImageUnit<'document> {
    span: SourceSpan,
    media_type: &'document str,
}

impl<'document> EmbeddedImageUnit<'document> {
    /// Exact source span of the data URI, including the `data:image/` prefix.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }

    /// Declared image media type (`image/png`, `image/jpeg`, …).
    #[must_use]
    pub const fn media_type(self) -> &'document str {
        self.media_type
    }
}

/// Locate `data:image/<type>;base64,...` units and retain their original spans.
///
/// # Errors
///
/// Returns [`EvidenceError::EmptySourceSpan`] when the document contains no
/// well-formed embedded image URI.
///
/// # Panics
///
/// Panics only if the coordinates derived from the validated document violate
/// the source-span invariant, which indicates an internal implementation bug.
pub fn embedded_image_units(
    document: &DocumentRecord,
) -> Result<Vec<EmbeddedImageUnit<'_>>, EvidenceError> {
    let text = document.text();
    let mut units = Vec::new();
    let mut search_from = 0usize;
    while let Some(relative) = text[search_from..].find(DATA_IMAGE_PREFIX) {
        let start = search_from + relative;
        let after_prefix = start + DATA_IMAGE_PREFIX.len();
        let Some(mark_rel) = text[after_prefix..].find(BASE64_MARK) else {
            search_from = after_prefix;
            continue;
        };
        let media_end = after_prefix + mark_rel;
        let payload_start = media_end + BASE64_MARK.len();
        let payload_end = payload_start
            + text[payload_start..]
                .find(|ch: char| !is_base64_payload_char(ch))
                .unwrap_or(text.len() - payload_start);
        if payload_end == payload_start {
            search_from = payload_start;
            continue;
        }
        // The fixed `data:image/` prefix already guarantees this media-type
        // boundary; retaining a second prefix guard would create unreachable
        // coverage obligations.
        let media_type = &text[start + "data:".len()..media_end];
        let scalar_start = text[..start].chars().count();
        let scalar_end = scalar_start + text[start..payload_end].chars().count();
        // These coordinates are derived from this validated document's own
        // UTF-8 boundaries and scalar counts, so SourceSpan validation cannot
        // reject them without an internal invariant violation.
        let span = SourceSpan::new(document, start, payload_end, scalar_start, scalar_end, None)
            .expect("derived embedded-image coordinates must form a valid source span");
        units.push(EmbeddedImageUnit { span, media_type });
        search_from = payload_end;
    }
    if units.is_empty() {
        return Err(EvidenceError::EmptySourceSpan);
    }
    Ok(units)
}

/// Refuse using a document body that still contains an embedded image as
/// lexical inference text.
///
/// # Errors
///
/// Returns [`EvidenceError::InvalidWirePayload`] for empty input and
/// [`EvidenceError::EmbeddedImageIsNotLexicalText`] when a `data:image`
/// base64 URI is present.
pub fn refuse_base64_image_as_lexical_text(text: &str) -> Result<(), EvidenceError> {
    if text.is_empty() {
        return Err(EvidenceError::InvalidWirePayload);
    }
    if text.contains(DATA_IMAGE_PREFIX) && text.contains(BASE64_MARK) {
        return Err(EvidenceError::EmbeddedImageIsNotLexicalText);
    }
    Ok(())
}

fn is_base64_payload_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '=')
}

#[cfg(test)]
mod tests {
    use super::{embedded_image_units, refuse_base64_image_as_lexical_text};
    use crate::{DocumentRecord, EvidenceError, SourceArtifact};

    #[test]
    fn jpeg_uri_and_incomplete_prefix_are_classified() {
        let text = "x data:image/jpeg;base64,/9j/4AA= y data:image/gif y";
        let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
        let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
        let units = embedded_image_units(&document).expect("jpeg");
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].media_type(), "image/jpeg");
        refuse_base64_image_as_lexical_text("plain note").expect("plain");
        refuse_base64_image_as_lexical_text("data:image/png").expect("incomplete image");
        assert_eq!(
            refuse_base64_image_as_lexical_text("data:image/png;base64,AAAA"),
            Err(EvidenceError::EmbeddedImageIsNotLexicalText)
        );
    }

    #[test]
    fn empty_payload_is_not_an_image_unit() {
        let text = "data:image/png;base64,";
        let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
        let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
        assert_eq!(
            embedded_image_units(&document),
            Err(EvidenceError::EmptySourceSpan)
        );
    }
}
