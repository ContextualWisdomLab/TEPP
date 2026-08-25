#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Event time cannot be replaced by assertion, system, document, or available time.
//!
//! Event/valid time is when a state was true. Other TEPP clocks are not
//! substitutes for that chronology (ADR 0002).

mod clock;
mod error;

/// Closed vocabulary of clocks that must not be confused with event time.
pub use clock::ClockFamily;
/// Fraction of recovered event flags that match known truth.
pub use clock::identity_recovery_rate;
/// Refuse to treat assertion time as event time.
pub use clock::refuse_assertion_time_as_event;
/// Refuse to treat availability time as event time.
pub use clock::refuse_available_time_as_event;
/// Refuse to treat document time as event time.
pub use clock::refuse_document_time_as_event;
/// Refuse to treat system time as event time.
pub use clock::refuse_system_time_as_event;
/// Return whether a stamp is on the event clock.
pub use clock::stamp_is_event;
/// Fail-closed event-clock errors.
pub use error::EventClockError;
