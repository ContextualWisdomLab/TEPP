#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Availability time cannot be replaced by event or system time.
//!
//! Historical eligibility uses availability versus knowledge cutoff. Event
//! time and system time are not substitutes (ADR 0002).

mod clock;
mod error;

/// Closed vocabulary of clocks that must not be confused with availability.
pub use clock::ClockFamily;
/// Fraction of recovered availability flags that match known truth.
pub use clock::eligibility_recovery_rate;
/// Refuse to treat event time as availability time.
pub use clock::refuse_event_time_as_available;
/// Refuse to treat system time as availability time.
pub use clock::refuse_system_time_as_available;
/// Return whether a stamp is on the availability clock.
pub use clock::stamp_is_available;
/// Fail-closed available-clock errors.
pub use error::AvailableClockError;
