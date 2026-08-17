#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Association, temporal precedence, and document links are not causal language.
//!
//! Identified experimental, quasi-experimental, and defensible observational
//! designs remain distinct from mere association (ADR 0003/0005).

mod claim;
mod error;

/// Closed vocabulary of association versus identified causal claims.
pub use claim::ClaimKind;
/// Fraction of recovered claim kinds that match known truth.
pub use claim::claim_kind_recovery_rate;
/// Refuse to treat an unidentified claim as causal language.
pub use claim::refuse_unidentified_as_causal;
/// Fail-closed causal-language errors.
pub use error::CausalLanguageError;
