#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Document analytical rows cannot omit assertion time or document time.
//!
//! Event/valid time, system time, and availability time are not substitutes
//! for assertion time or document time (ADR 0002/0013).

mod clocks;
mod error;

/// Closed vocabulary of document-scoped clock families.
pub use clocks::ClockFamily;
/// One typed instant on a document-scoped clock.
pub use clocks::DocumentClockInstant;
/// One document row with the five document-scoped clocks.
pub use clocks::DocumentClockRow;
/// Fraction of recovered completeness flags that match known truth.
pub use clocks::clock_completeness_recovery_rate;
/// Return whether a document row carries every required clock family.
pub use clocks::clocks_are_complete;
/// Refuse a document row that omits assertion time or document time.
pub use clocks::refuse_omitted_assertion_or_document_time;
/// Fail-closed document-clock errors.
pub use error::DocumentClockError;
