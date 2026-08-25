#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Later document revisions cannot move backward in system time.
//!
//! A higher revision number is a later assertion about the same document
//! identity. Its system time must strictly increase (ADR 0002/0013).

mod error;
mod revision;

/// Fail-closed revision-order errors.
pub use error::RevisionOrderError;
/// One document revision with a positive revision number and system time.
pub use revision::DocumentRevision;
/// Fraction of recovered order flags that match known truth.
pub use revision::order_recovery_rate;
/// Refuse a later revision whose system time did not increase.
pub use revision::refuse_nonincreasing_system_time;
/// Return whether a later revision has a later system time.
pub use revision::revisions_are_increasing;
