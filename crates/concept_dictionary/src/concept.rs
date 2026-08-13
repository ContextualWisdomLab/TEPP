//! Shared concept identity across language-specific lexical channels.

use crate::error::ConceptError;
use crate::language::LanguageTag;

/// Opaque versioned concept identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConceptId {
    bytes: [u8; 16],
}

impl ConceptId {
    /// Construct a concept identity from sixteen explicit bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    /// Return the identity bytes.
    #[must_use]
    pub const fn as_bytes(self) -> [u8; 16] {
        self.bytes
    }
}

/// Shared concept identity across two language channels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedConceptAlignment {
    concept: ConceptId,
    left_language: LanguageTag,
    right_language: LanguageTag,
}

impl SharedConceptAlignment {
    /// Shared concept identity.
    #[must_use]
    pub const fn concept(self) -> ConceptId {
        self.concept
    }

    /// Left language channel.
    #[must_use]
    pub const fn left_language(self) -> LanguageTag {
        self.left_language
    }

    /// Right language channel.
    #[must_use]
    pub const fn right_language(self) -> LanguageTag {
        self.right_language
    }
}

/// Bind one concept identity to two language channels.
///
/// Native lexical emissions remain language-specific; the concept identity is
/// shared (ADR 0004).
#[must_use]
pub const fn share_concept(
    left_language: LanguageTag,
    right_language: LanguageTag,
    concept: ConceptId,
) -> SharedConceptAlignment {
    SharedConceptAlignment {
        concept,
        left_language,
        right_language,
    }
}

/// Refuse to treat machine translation as measurement equivalence.
///
/// # Errors
///
/// Always returns [`ConceptError::TranslationNotEquivalence`].
pub const fn treat_translation_as_equivalence() -> Result<(), ConceptError> {
    Err(ConceptError::TranslationNotEquivalence)
}

/// Refuse to treat a language-specific lexical form as concept identity.
///
/// # Errors
///
/// Always returns [`ConceptError::LexicalFormNotConcept`].
pub const fn treat_lexical_form_as_concept_identity() -> Result<(), ConceptError> {
    Err(ConceptError::LexicalFormNotConcept)
}

#[cfg(test)]
mod tests {
    use super::{
        ConceptId, share_concept, treat_lexical_form_as_concept_identity,
        treat_translation_as_equivalence,
    };
    use crate::error::ConceptError;
    use crate::language::LanguageTag;

    #[test]
    fn shared_identity_is_not_a_lexical_or_translated_form() {
        let concept = ConceptId::from_bytes([3; 16]);
        let shared = share_concept(LanguageTag::Fra, LanguageTag::Deu, concept);
        assert_eq!(shared.concept().as_bytes(), [3; 16]);
        assert_eq!(
            treat_translation_as_equivalence(),
            Err(ConceptError::TranslationNotEquivalence)
        );
        assert_eq!(
            treat_lexical_form_as_concept_identity(),
            Err(ConceptError::LexicalFormNotConcept)
        );
    }
}
