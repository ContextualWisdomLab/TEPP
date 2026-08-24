#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Non-lexical modality is not unique latent content.
//!
//! Modality channels stay explicit method/background structure. They are
//! not unique document meaning and are not erased by a stopword list
//! (ADR 0004/0012).

mod error;
mod kind;

/// Fail-closed modality-source errors.
pub use error::ModalitySourceError;
/// Closed vocabulary of modality-related token treatments.
pub use kind::ModalityKind;
/// Fraction of recovered modality kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat non-lexical modality as stopword deletion.
pub use kind::refuse_modality_as_stopword_deletion;
/// Refuse to treat non-lexical modality as unique latent content.
pub use kind::refuse_modality_as_unique_content;
