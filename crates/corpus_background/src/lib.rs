#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Corpus-background wording is not unique latent content.
//!
//! Corpus-level background language stays explicit method/background
//! structure. It is not unique document meaning and is not erased by a
//! stopword list (ADR 0004/0012).

mod error;
mod kind;

/// Fail-closed corpus-background errors.
pub use error::CorpusBackgroundError;
/// Closed vocabulary of corpus-background token treatments.
pub use kind::CorpusBackgroundKind;
/// Fraction of recovered corpus-background kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat corpus-background wording as stopword deletion.
pub use kind::refuse_corpus_background_as_stopword_deletion;
/// Refuse to treat corpus-background wording as unique latent content.
pub use kind::refuse_corpus_background_as_unique_content;
