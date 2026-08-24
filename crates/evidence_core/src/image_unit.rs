//! Embedded `data:image` units that keep their original source location.

use crate::{DocumentRecord, EvidenceError, SourceSpan};

const DATA_IMAGE_PREFIX: &str = "data:image/";
const BASE64_MARK: &str = ";base64,";

/// Image media types accepted as plausible by [`embedded_image_units`].
///
/// The set is deliberately conservative and tracks widely registered or
/// de facto standard image subtypes; anything else fails closed instead of
/// yielding a bogus embedded-image unit.
const PLAUSIBLE_IMAGE_MEDIA_TYPES: [&str; 14] = [
    "image/apng",
    "image/avif",
    "image/bmp",
    "image/gif",
    "image/heic",
    "image/heif",
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/svg+xml",
    "image/tiff",
    "image/vnd.microsoft.icon",
    "image/webp",
    "image/x-icon",
];

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
/// Only plausible image media types are accepted: a candidate URI whose
/// declared media type is not in [`PLAUSIBLE_IMAGE_MEDIA_TYPES`] fails the
/// whole parse so malformed bodies cannot produce bogus units.
///
/// # Errors
///
/// Returns [`EvidenceError::EmptySourceSpan`] when the document contains no
/// well-formed embedded image URI, and
/// [`EvidenceError::ImplausibleImageMediaType`] when a candidate URI
/// declares an implausible image media type.
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
        let media_type = &text[start + "data:".len()..media_end];
        if media_type.contains(DATA_IMAGE_PREFIX) {
            search_from = after_prefix;
            continue;
        }
        if !is_plausible_image_media_type(media_type) {
            return Err(EvidenceError::ImplausibleImageMediaType);
        }
        let scalar_start = text[..start].chars().count();
        let scalar_end = scalar_start + text[start..payload_end].chars().count();
        let span = SourceSpan::new(document, start, payload_end, scalar_start, scalar_end, None)?;
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
    if contains_base64_image_data_uri(text) {
        return Err(EvidenceError::EmbeddedImageIsNotLexicalText);
    }
    Ok(())
}

fn contains_base64_image_data_uri(text: &str) -> bool {
    let mut search_from = 0usize;
    while let Some(relative) = text[search_from..].find(DATA_IMAGE_PREFIX) {
        let start = search_from + relative;
        let after_prefix = start + DATA_IMAGE_PREFIX.len();
        let Some(mark_rel) = text[after_prefix..].find(BASE64_MARK) else {
            search_from = after_prefix;
            continue;
        };
        let media_end = after_prefix + mark_rel;
        let media_type = &text[start + "data:".len()..media_end];
        if is_image_media_type_token(media_type) {
            return true;
        }
        search_from = after_prefix;
    }
    false
}

fn is_image_media_type_token(media_type: &str) -> bool {
    let Some(subtype) = media_type.strip_prefix("image/") else {
        return false;
    };
    !subtype.is_empty()
        && subtype
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'-'))
}

fn is_base64_payload_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '=')
}

/// Report whether a declared media type is a plausible image media type.
fn is_plausible_image_media_type(media_type: &str) -> bool {
    PLAUSIBLE_IMAGE_MEDIA_TYPES.contains(&media_type)
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

        let empty_text = "data:image/png;base64, following text";
        let empty_artifact = SourceArtifact::from_bytes(empty_text.as_bytes()).expect("artifact");
        let empty_document =
            DocumentRecord::from_text(empty_artifact.id(), empty_text).expect("document");
        assert_eq!(
            embedded_image_units(&empty_document),
            Err(EvidenceError::EmptySourceSpan)
        );
    }

    #[test]
    fn implausible_media_types_fail_closed() {
        let text = "data:image/not-a-type;base64,AAAA";
        let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
        let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
        assert_eq!(
            embedded_image_units(&document),
            Err(EvidenceError::ImplausibleImageMediaType)
        );
        assert_eq!(
            refuse_base64_image_as_lexical_text(text),
            Err(EvidenceError::EmbeddedImageIsNotLexicalText)
        );
    }

    #[test]
    fn common_raster_media_types_are_accepted() {
        let text = "a data:image/png;base64,AAAA b data:image/jpeg;base64,BBBB \
                    c data:image/webp;base64,CCCC d data:image/gif;base64,DDDD e";
        let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
        let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
        let units = embedded_image_units(&document).expect("units");
        let media_types: Vec<&str> = units.iter().map(|unit| unit.media_type()).collect();
        assert_eq!(
            media_types,
            vec!["image/png", "image/jpeg", "image/webp", "image/gif"]
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

    #[test]
    fn malformed_image_prefix_does_not_swallow_later_valid_image() {
        let text = "data:image/gif then data:image/png;base64,AAAA";
        let artifact = SourceArtifact::from_bytes(text.as_bytes()).expect("artifact");
        let document = DocumentRecord::from_text(artifact.id(), text).expect("document");

        let units = embedded_image_units(&document).expect("png");
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].media_type(), "image/png");
        assert_eq!(
            units[0].span().byte_start(),
            text.find("data:image/png").expect("png start")
        );
    }
}
