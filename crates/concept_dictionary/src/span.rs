//! Exact source spans required by semantic units.

use crate::concept::ConceptId;
use crate::error::ConceptError;
use crate::language::LanguageTag;

/// Half-open Unicode-scalar source span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceSpan {
    start_scalar: usize,
    end_scalar: usize,
}

impl SourceSpan {
    /// Inclusive start scalar.
    #[must_use]
    pub const fn start_scalar(self) -> usize {
        self.start_scalar
    }

    /// Exclusive end scalar.
    #[must_use]
    pub const fn end_scalar(self) -> usize {
        self.end_scalar
    }
}

/// Semantic unit bound to an exact source span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticUnit {
    language: LanguageTag,
    span: SourceSpan,
    concept: Option<ConceptId>,
}

impl SemanticUnit {
    /// Language of the source emission.
    #[must_use]
    pub const fn language(self) -> LanguageTag {
        self.language
    }

    /// Exact source span.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }

    /// Bound concept, if known.
    #[must_use]
    pub const fn concept(self) -> Option<ConceptId> {
        self.concept
    }
}

/// Validate a half-open Unicode-scalar source span.
///
/// # Errors
///
/// Returns [`ConceptError::InvalidSourceSpan`] when the range is empty or
/// reversed.
pub const fn source_span(
    start_scalar: usize,
    end_scalar: usize,
) -> Result<SourceSpan, ConceptError> {
    if start_scalar >= end_scalar {
        Err(ConceptError::InvalidSourceSpan)
    } else {
        Ok(SourceSpan {
            start_scalar,
            end_scalar,
        })
    }
}

/// Bind a semantic unit to an exact source span.
///
/// Unknown meaning may remain unresolved. A missing span is never accepted.
///
/// # Errors
///
/// Returns [`ConceptError::MissingSourceSpan`] when no span is supplied.
pub const fn bind_semantic_unit(
    language: LanguageTag,
    span: Option<SourceSpan>,
    concept: Option<ConceptId>,
) -> Result<SemanticUnit, ConceptError> {
    match span {
        Some(span) => Ok(SemanticUnit {
            language,
            span,
            concept,
        }),
        None => Err(ConceptError::MissingSourceSpan),
    }
}

/// Refuse to force an unknown meaning into a known concept.
///
/// # Errors
///
/// Returns [`ConceptError::ForcedConceptAssignment`] when the unit has no
/// concept binding.
pub const fn force_unknown_into_known(
    unit: &SemanticUnit,
    _known: ConceptId,
) -> Result<(), ConceptError> {
    match unit.concept() {
        Some(_) => Ok(()),
        None => Err(ConceptError::ForcedConceptAssignment),
    }
}

#[cfg(test)]
mod tests {
    use super::{bind_semantic_unit, force_unknown_into_known, source_span};
    use crate::concept::ConceptId;
    use crate::error::ConceptError;
    use crate::language::LanguageTag;

    #[test]
    fn empty_span_and_unknown_force_fail_closed() {
        assert_eq!(source_span(1, 1), Err(ConceptError::InvalidSourceSpan));
        let span = source_span(1, 3).expect("span");
        let unknown = bind_semantic_unit(LanguageTag::Vie, Some(span), None).expect("unknown");
        assert_eq!(
            force_unknown_into_known(&unknown, ConceptId::from_bytes([9; 16])),
            Err(ConceptError::ForcedConceptAssignment)
        );
    }
}
