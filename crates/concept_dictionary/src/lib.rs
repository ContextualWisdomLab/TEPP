#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Shared multilingual concept identity and language-profile validity gates.
//!
//! Equivalent meanings share one concept identity. Language-specific lexical
//! emissions remain native. Architecture support is not validated
//! interpretation. Semantic units require exact source spans, and unknown
//! meaning stays unresolved.

mod alignment;
mod concept;
mod error;
mod language;
mod preprocessing;
mod span;

/// Invariance evidence required for a claimed comparison.
pub use alignment::InvarianceLevel;
/// Permit cross-language means only with invariance evidence.
pub use alignment::compare_cross_language_means;
/// CPU `f64` concept-coordinate RMSE.
pub use alignment::concept_coordinate_rmse;
/// Shared concept identity.
pub use concept::ConceptId;
/// Shared-concept alignment across languages.
pub use concept::SharedConceptAlignment;
/// Bind one concept to two language channels.
pub use concept::share_concept;
/// Refuse lexical form as concept identity.
pub use concept::treat_lexical_form_as_concept_identity;
/// Refuse translation as measurement equivalence.
pub use concept::treat_translation_as_equivalence;
/// Fail-closed concept-dictionary errors.
pub use error::ConceptError;
/// Language profile with explicit validity status.
pub use language::LanguageProfile;
/// BCP 47 language tag.
pub use language::LanguageTag;
/// Language-profile validity status.
pub use language::ProfileStatus;
/// Permit comparative interpretation only for validated profiles.
pub use language::claim_comparative_interpretation;
/// Inferential weight kind.
pub use preprocessing::InferentialWeightKind;
/// Admit only statistical/posterior inferential weights.
pub use preprocessing::admit_inferential_weight;
/// Refuse default stopword deletion.
pub use preprocessing::apply_default_stopword_deletion;
/// Semantic unit grounded in an exact span.
pub use span::SemanticUnit;
/// Exact source span.
pub use span::SourceSpan;
/// Bind a semantic unit to an exact source span.
pub use span::bind_semantic_unit;
/// Refuse forcing unknown meaning into a known concept.
pub use span::force_unknown_into_known;
/// Validate a half-open Unicode-scalar span.
pub use span::source_span;
