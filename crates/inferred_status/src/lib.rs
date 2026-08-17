#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Inferred relations cannot be promoted to observed evidence or transitions.
//!
//! LLM, reasoner, and heuristic proposals stay inferred until deterministic
//! schema, evidence, and scientific validation promote them (ADR 0003).

mod error;
mod status;

/// Fail-closed inferred-status errors.
pub use error::InferredStatusError;
/// Fraction of recovered evidence statuses that match known truth.
pub use status::identity_recovery_rate;
/// Refuse to treat an inferred relation as observed evidence.
pub use status::refuse_inferred_as_observed;
/// Refuse to treat an inferred relation as a state transition.
pub use status::refuse_inferred_as_transition;
/// Return whether a status is observed evidence.
pub use status::status_is_observed;
/// Closed vocabulary of presence evidence that is not yet a transition.
pub use status::EvidenceStatus;
