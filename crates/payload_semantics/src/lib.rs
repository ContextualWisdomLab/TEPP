#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Untrusted payloads fail closed until scientific semantics validate.
//!
//! Documents, external metadata, serialized records, and LLM outputs stay
//! untrusted as estimator or posterior authority. Passing identity, size,
//! or authorization bounds is not scientific semantics (ADR 0008/0014).

mod error;
mod semantics;

/// Fail-closed payload-semantics errors.
pub use error::PayloadSemanticsError;
/// Closed vocabulary of untrusted inbound payload kinds.
pub use semantics::PayloadKind;
/// Closed vocabulary of claimed scientific roles.
pub use semantics::ScientificRole;
/// Refuse to treat identity, size, or authorization bounds as semantics.
pub use semantics::refuse_bounds_as_semantics;
/// Refuse an untrusted payload that claims an unauthorized scientific role.
pub use semantics::refuse_untrusted_scientific_claim;
/// Fraction of recovered scientific roles that match known truth.
pub use semantics::semantics_recovery_rate;
