#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Span-grounded semantic units whose identity is never a language tag.
//!
//! Language metadata may mark a unit unresolved or with a primary ISO 639
//! subtag. Unresolved metadata does not retokenize or move the exact source
//! span. Equivalent Korean and English surfaces remain distinct units until a
//! later concept-alignment layer (ADR 0004 / ADR 0012) is validated.

mod error;
mod profile;
mod unit;

/// Fail-closed semantic-unit validation errors.
pub use error::SemanticError;
/// Language profile metadata, never unit identity.
pub use profile::LanguageProfile;
/// Exact-span identity of one semantic unit.
pub use unit::SemanticIdentity;
/// One exact-span semantic unit.
pub use unit::SemanticUnit;
