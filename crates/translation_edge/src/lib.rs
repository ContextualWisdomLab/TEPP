#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Translation, same-language copy, and revision are not state transitions.
//!
//! Provenance may point to earlier event time. A same primary language tag
//! cannot be classified as a translation (ADR 0002/0003).

mod error;
mod kind;

/// Fail-closed translation-edge errors.
pub use error::TranslationEdgeError;
/// Closed vocabulary of translation-related provenance that is not a transition.
pub use kind::TranslationKind;
/// Fraction of recovered provenance kinds that match known truth.
pub use kind::edge_kind_recovery_rate;
/// Refuse to treat a same-language pair as a translation.
pub use kind::refuse_same_language_as_translation;
/// Refuse to treat a translation-related edge as a forward state transition.
pub use kind::refuse_translation_as_transition;
