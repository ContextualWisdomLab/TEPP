#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Copied-text residue is not unique latent content.
//!
//! Copied and boilerplate passages stay explicit method/background structure.
//! They are not unique document meaning and are not erased by a stopword list
//! (ADR 0004/0012).

mod error;
mod kind;

/// Fail-closed copied-text errors.
pub use error::CopiedTextError;
/// Closed vocabulary of copied-text token treatments.
pub use kind::CopiedKind;
/// Fraction of recovered copied-text kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat copied-text residue as stopword deletion.
pub use kind::refuse_copied_text_as_stopword_deletion;
/// Refuse to treat copied-text residue as unique latent content.
pub use kind::refuse_copied_text_as_unique_content;
