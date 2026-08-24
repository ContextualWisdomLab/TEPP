//! Errors returned by immutable evidence-domain validation.

use std::fmt;

/// A fail-closed validation error for evidence identifiers and records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EvidenceError {
    /// The supplied identifier was malformed or was not an RFC 9562 `UUIDv7`.
    InvalidEvidenceId,
    /// The supplied content digest was not exactly 32 hexadecimal bytes.
    InvalidContentDigest,
    /// A JSON wire payload was malformed, incomplete, or contained unknown fields.
    InvalidWirePayload,
    /// A JSON wire payload used a schema version this crate does not support.
    UnsupportedWireVersion,
    /// A wire record's declared digest did not match its reconstructed content.
    ContentDigestMismatch,
    /// A source artifact contained no bytes.
    EmptySourceArtifact,
    /// A source artifact exceeded its configured byte limit.
    SourceArtifactTooLarge,
    /// A document contained no text.
    EmptyDocument,
    /// A document exceeded its configured UTF-8 byte limit.
    DocumentTooLarge,
    /// A source span selected no bytes or no Unicode scalar values.
    EmptySourceSpan,
    /// A source span used reversed byte or Unicode-scalar coordinates.
    InvalidSourceSpanOrder,
    /// A source span exceeded the document byte or Unicode-scalar bounds.
    ByteRangeOutOfBounds,
    /// A source span started or ended inside a UTF-8 code point.
    InvalidUtf8Boundary,
    /// A source span's Unicode-scalar coordinates disagreed with its byte range.
    CharacterRangeMismatch,
    /// A source span was applied to a document other than its owner.
    SpanDocumentMismatch,
    /// A page number was zero.
    InvalidPageNumber,
    /// Page dimensions were nonfinite or nonpositive.
    InvalidPageGeometry,
    /// Layout coordinates were nonfinite, negative, or empty.
    InvalidLayoutBounds,
    /// Layout coordinates exceeded the enclosing page.
    LayoutOutOfBounds,
    /// A base64 image data URI was treated as lexical inference text.
    EmbeddedImageIsNotLexicalText,
    /// An embedded image data URI declared an implausible image media type.
    ImplausibleImageMediaType,
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidEvidenceId => "invalid evidence identifier",
            Self::InvalidContentDigest => "invalid content digest",
            Self::InvalidWirePayload => "invalid evidence wire payload",
            Self::UnsupportedWireVersion => "unsupported evidence wire version",
            Self::ContentDigestMismatch => "wire content does not match its declared digest",
            Self::EmptySourceArtifact => "source artifact is empty",
            Self::SourceArtifactTooLarge => "source artifact exceeds the configured byte limit",
            Self::EmptyDocument => "document text is empty",
            Self::DocumentTooLarge => "document text exceeds the configured byte limit",
            Self::EmptySourceSpan => "source span is empty",
            Self::InvalidSourceSpanOrder => "source span coordinates are not ordered",
            Self::ByteRangeOutOfBounds => "source span byte range is out of bounds",
            Self::InvalidUtf8Boundary => "source span does not use UTF-8 boundaries",
            Self::CharacterRangeMismatch => {
                "source span character coordinates do not match its byte range"
            }
            Self::SpanDocumentMismatch => "source span belongs to a different document",
            Self::InvalidPageNumber => "page number must be positive",
            Self::InvalidPageGeometry => "page geometry must be finite and positive",
            Self::InvalidLayoutBounds => "layout bounds must be finite, nonnegative, and nonempty",
            Self::LayoutOutOfBounds => "layout bounds exceed the page geometry",
            Self::EmbeddedImageIsNotLexicalText => "embedded image is not lexical text",
            Self::ImplausibleImageMediaType => "embedded image media type is implausible",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for EvidenceError {}
