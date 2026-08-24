#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Assertion time cannot be replaced by event, system, document, or available time.
//!
//! A claim's assertion clock is when the source asserted it. Other TEPP clocks
//! are not substitutes (ADR 0002).

mod clock;
mod error;

/// Closed vocabulary of clocks that must not be confused with assertion time.
pub use clock::ClockFamily;
/// Fraction of recovered assertion flags that match known truth.
pub use clock::identity_recovery_rate;
/// Refuse to treat availability time as assertion time.
pub use clock::refuse_available_time_as_assertion;
/// Refuse to treat document time as assertion time.
pub use clock::refuse_document_time_as_assertion;
/// Refuse to treat event time as assertion time.
pub use clock::refuse_event_time_as_assertion;
/// Refuse to treat system time as assertion time.
pub use clock::refuse_system_time_as_assertion;
/// Return whether a stamp is on the assertion clock.
pub use clock::stamp_is_assertion;
/// Fail-closed assertion-clock errors.
pub use error::AssertionClockError;
