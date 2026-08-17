#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Report section boilerplate is not unique latent content.
//!
//! Repeated section headings stay explicit method/background structure. They
//! are not unique document meaning and are not erased by a stopword list
//! (ADR 0004/0012).

mod error;
mod kind;

/// Fail-closed section-source errors.
pub use error::SectionSourceError;
/// Closed vocabulary of section-related token treatments.
pub use kind::SectionKind;
/// Fraction of recovered section kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat section boilerplate as stopword deletion.
pub use kind::refuse_section_as_stopword_deletion;
/// Refuse to treat section boilerplate as unique latent content.
pub use kind::refuse_section_as_unique_content;
