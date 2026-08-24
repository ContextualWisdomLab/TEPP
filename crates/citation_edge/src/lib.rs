#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Citation and retrospective edges cannot become reverse state transitions.
//!
//! Provenance edges may point to earlier event time. They never become
//! input-process-outcome transitions (ADR 0002/0003).

mod error;
mod kind;

/// Fail-closed citation-edge errors.
pub use error::CitationEdgeError;
/// Closed vocabulary of provenance edges that are not state transitions.
pub use kind::ProvenanceKind;
/// Fraction of recovered provenance kinds that match known truth.
pub use kind::edge_kind_recovery_rate;
/// Refuse to treat a provenance edge as a forward state transition.
pub use kind::refuse_provenance_as_transition;
