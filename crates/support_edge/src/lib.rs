#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Support, contradiction, summary, and `outcome_of` are not state transitions.
//!
//! Evidential and inverse-production provenance may point to earlier event
//! time. They never become input-process-outcome transitions (ADR 0002/0003).

mod error;
mod kind;

/// Fail-closed support-edge errors.
pub use error::SupportEdgeError;
/// Closed vocabulary of evidential edges that are not state transitions.
pub use kind::EvidenceKind;
/// Fraction of recovered evidential kinds that match known truth.
pub use kind::edge_kind_recovery_rate;
/// Refuse to treat an evidential edge as a forward state transition.
pub use kind::refuse_evidence_as_transition;
