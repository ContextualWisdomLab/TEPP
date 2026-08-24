#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Knowledge cutoff cannot be replaced by event, system, or availability time.
//!
//! Historical eligibility compares availability with a distinct analysis cutoff.
//! Event time, system time, and availability time are not substitutes (ADR 0002).

mod clock;
mod error;

/// Closed vocabulary of clocks that must not be confused with knowledge cutoff.
pub use clock::ClockFamily;
/// Fraction of recovered cutoff flags that match known truth.
pub use clock::eligibility_recovery_rate;
/// Refuse to treat availability time as knowledge cutoff.
pub use clock::refuse_available_time_as_cutoff;
/// Refuse to treat event time as knowledge cutoff.
pub use clock::refuse_event_time_as_cutoff;
/// Refuse to treat system time as knowledge cutoff.
pub use clock::refuse_system_time_as_cutoff;
/// Return whether a stamp is on the knowledge-cutoff clock.
pub use clock::stamp_is_cutoff;
/// Fail-closed cutoff-clock errors.
pub use error::CutoffClockError;
