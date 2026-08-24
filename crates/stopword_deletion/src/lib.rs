#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Default stopword deletion is not a valid method for repeated report language.
//!
//! A global stopword list cannot erase boilerplate. Repeated template, section,
//! copied-text, style, modality, and corpus-background wording stays explicit
//! method/background structure (ADR 0004/0012).

mod error;
mod kind;

/// Fail-closed stopword-deletion errors.
pub use error::StopwordDeletionError;
/// Closed vocabulary of deletion versus explicit method-source treatments.
pub use kind::DeletionKind;
/// Fraction of recovered deletion kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat a default stopword list as a valid deletion method.
pub use kind::refuse_default_stopword_deletion;
