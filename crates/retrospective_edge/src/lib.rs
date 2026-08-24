#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Retrospective reporting is not a transition and not a translation.
//!
//! A later report may point at earlier event time. It never becomes an
//! input-process-outcome edge or a translation (ADR 0002/0003).

mod error;
mod kind;

/// Fail-closed retrospective-edge errors.
pub use error::RetrospectiveEdgeError;
/// Closed vocabulary of reporting edges that may point at earlier event time.
pub use kind::RetrospectiveKind;
/// Fraction of recovered reporting kinds that match known truth.
pub use kind::identity_recovery_rate;
/// Refuse to treat a retrospective report as a forward state transition.
pub use kind::refuse_retrospective_as_transition;
/// Refuse to treat a retrospective report as a translation.
pub use kind::refuse_retrospective_as_translation;
