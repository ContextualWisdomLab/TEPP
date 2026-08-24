//! Span-grounded semantic units.

use crate::error::SemanticError;
use crate::profile::LanguageProfile;
use evidence_core::{EvidenceId, SourceSpan};

/// Exact-span identity of one semantic unit.
///
/// Identity is `(document_id, byte_start, byte_end)`. Scalar character
/// coordinates are derived deterministically from byte coordinates inside one
/// document encoding, so they add no distinguishing power; page or layout
/// positions are presentation metadata that may vary across renderings and
/// therefore stay outside identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticIdentity {
    document_id: EvidenceId,
    byte_start: usize,
    byte_end: usize,
}

impl SemanticIdentity {
    /// Construct identity from an exact source span.
    #[must_use]
    pub const fn from_span(span: SourceSpan) -> Self {
        Self {
            document_id: span.document_id(),
            byte_start: span.byte_start(),
            byte_end: span.byte_end(),
        }
    }

    /// Return the owning document identifier.
    #[must_use]
    pub const fn document_id(&self) -> EvidenceId {
        self.document_id
    }

    /// Return the inclusive byte start.
    #[must_use]
    pub const fn byte_start(&self) -> usize {
        self.byte_start
    }

    /// Return the exclusive byte end.
    #[must_use]
    pub const fn byte_end(&self) -> usize {
        self.byte_end
    }

    /// Refuse a language tag as semantic-unit identity.
    ///
    /// # Errors
    ///
    /// Always returns [`SemanticError::LanguageIsNotIdentity`].
    pub fn from_language_tag(_tag: &str) -> Result<Self, SemanticError> {
        Err(SemanticError::LanguageIsNotIdentity)
    }
}

/// One exact-span semantic unit with optional language metadata.
#[derive(Clone, Debug)]
pub struct SemanticUnit {
    span: SourceSpan,
    language: LanguageProfile,
}

impl PartialEq for SemanticUnit {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}

impl Eq for SemanticUnit {}

impl SemanticUnit {
    /// Bind a language profile onto an exact source span.
    ///
    /// Unresolved and tagged profiles keep the same byte and scalar bounds.
    /// The profile never becomes [`SemanticIdentity`].
    #[must_use]
    pub fn bind(span: SourceSpan, language: LanguageProfile) -> Self {
        Self { span, language }
    }

    /// Return the exact source span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    /// Return the language profile metadata.
    #[must_use]
    pub const fn language(&self) -> &LanguageProfile {
        &self.language
    }

    /// Return span-grounded identity.
    #[must_use]
    pub const fn identity(&self) -> SemanticIdentity {
        SemanticIdentity::from_span(self.span)
    }

    /// Replace language metadata without moving the span.
    #[must_use]
    pub fn with_language(self, language: LanguageProfile) -> Self {
        Self {
            span: self.span,
            language,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SemanticIdentity, SemanticUnit};
    use crate::error::SemanticError;
    use crate::profile::LanguageProfile;
    use evidence_core::{DocumentRecord, SourceArtifact, SourceSpan};

    fn span_over(text: &str, start: usize, end: usize) -> SourceSpan {
        let artifact = SourceArtifact::from_bytes(b"src").expect("artifact");
        let document = DocumentRecord::from_text(artifact.id(), text).expect("document");
        let scalar_start = text[..start].chars().count();
        let scalar_end = scalar_start + text[start..end].chars().count();
        SourceSpan::new(&document, start, end, scalar_start, scalar_end, None).expect("span")
    }

    #[test]
    fn language_tag_cannot_become_identity() {
        assert_eq!(
            SemanticIdentity::from_language_tag("ko").unwrap_err(),
            SemanticError::LanguageIsNotIdentity
        );
        assert_eq!(
            SemanticIdentity::from_language_tag("en-us").unwrap_err(),
            SemanticError::LanguageIsNotIdentity
        );
    }

    #[test]
    fn unresolved_profile_keeps_supplied_korean_span() {
        let korean = "측정 오차는 RMSE로 보고한다.";
        let start = 0;
        let end = "측정".len();
        let span = span_over(korean, start, end);
        let unresolved = SemanticUnit::bind(span, LanguageProfile::unresolved());
        let tagged = unresolved
            .clone()
            .with_language(LanguageProfile::parse_bcp47("ko").expect("ko"));
        assert_eq!(unresolved.identity(), tagged.identity());
        assert_eq!(unresolved.span().byte_start(), start);
        assert_eq!(unresolved.span().byte_end(), end);
        assert_eq!(tagged.span().byte_start(), start);
        assert_eq!(tagged.span().byte_end(), end);
        assert_ne!(unresolved.language(), tagged.language());
        assert_eq!(unresolved, tagged);
        assert_eq!(unresolved.identity().document_id(), span.document_id());
        assert_eq!(unresolved.identity().byte_start(), start);
        assert_eq!(unresolved.identity().byte_end(), end);
    }
}
