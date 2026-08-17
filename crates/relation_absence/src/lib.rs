#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Unobserved relation pairs are not evidence of no relationship.
//!
//! Observed and inferred statuses stay distinct. Missing pairs remain
//! unobserved and never become negative edges (ADR 0003).

mod error;
mod status;

/// Fail-closed relation-absence errors.
pub use error::RelationAbsenceError;
/// Closed vocabulary of observed, inferred, and unobserved statuses.
pub use status::ObservationStatus;
/// Refuse to treat an unobserved pair as evidence of no relationship.
pub use status::refuse_absence_as_negative;
/// Fraction of recovered observation statuses that match known truth.
pub use status::status_recovery_rate;
