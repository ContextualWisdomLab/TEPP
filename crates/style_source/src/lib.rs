#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! House-voice style residue is not unique latent content.
//!
//! Style and house-voice stay explicit method/background structure. They
//! are not unique document meaning and are not erased by a stopword list
//! (ADR 0004/0012).

mod error;
mod kind;

/// Fail-closed style-source errors.
pub use error::StyleSourceError;
/// Closed vocabulary of style-related token treatments.
pub use kind::StyleKind;
/// Fraction of recovered style kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat style residue as stopword deletion.
pub use kind::refuse_style_as_stopword_deletion;
/// Refuse to treat style residue as unique latent content.
pub use kind::refuse_style_as_unique_content;
