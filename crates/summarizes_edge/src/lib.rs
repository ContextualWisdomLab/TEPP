#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! A summary is not a state transition and not the source document.
//!
//! Summary provenance may point to earlier event time. It never becomes an
//! input-process-outcome edge and never reuses the source identity
//! (ADR 0003).

mod error;
mod kind;

/// Fail-closed summarizes-edge errors.
pub use error::SummarizesEdgeError;
/// Fraction of recovered summary kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat a summary as the source document identity.
pub use kind::refuse_summary_as_source_identity;
/// Refuse to treat a summary as a forward state transition.
pub use kind::refuse_summary_as_transition;
/// Closed vocabulary of summary-related document identities.
pub use kind::SummarizesKind;
