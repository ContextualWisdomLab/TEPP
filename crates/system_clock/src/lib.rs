#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! System time cannot be replaced by the other TEPP clocks.
//!
//! System/record time is when TEPP recorded a change. Event, assertion,
//! document, available, and cutoff times are not substitutes (ADR 0002).

mod clock;
mod error;

/// Closed vocabulary of clocks that must not be confused with system time.
pub use clock::ClockFamily;
/// Fraction of recovered system-clock flags that match known truth.
pub use clock::identity_recovery_rate;
/// Refuse to treat assertion time as system time.
pub use clock::refuse_assertion_time_as_system;
/// Refuse to treat availability time as system time.
pub use clock::refuse_available_time_as_system;
/// Refuse to treat knowledge-cutoff time as system time.
pub use clock::refuse_cutoff_time_as_system;
/// Refuse to treat document time as system time.
pub use clock::refuse_document_time_as_system;
/// Refuse to treat event time as system time.
pub use clock::refuse_event_time_as_system;
/// Return whether a stamp is on the system clock.
pub use clock::stamp_is_system;
/// Fail-closed system-clock errors.
pub use error::SystemClockError;
