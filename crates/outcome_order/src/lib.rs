#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Input-process-outcome edges never move backward in event time.
//!
//! `input_to` and `process_to` are forward transitions. `outcome_of` is
//! provenance (the inverse of `produces`) and may point at an earlier
//! producer without becoming a reverse state transition (ADR 0002/0003).

mod error;
mod kind;

/// Fail-closed input-process-outcome order errors.
pub use error::OutcomeOrderError;
/// Closed vocabulary of input, process, and outcome-of edges.
pub use kind::OutcomeKind;
/// Fraction of recovered IPO kinds that match known truth.
pub use kind::kind_recovery_rate;
/// Refuse to treat `outcome_of` as a forward state transition.
pub use kind::refuse_outcome_of_as_transition;
/// Refuse reverse event-time order on input and process transitions.
pub use kind::refuse_reverse_ipo_order;
