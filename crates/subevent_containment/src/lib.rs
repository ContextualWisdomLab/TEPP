#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Subevent intervals must stay inside the parent event interval.
//!
//! A subevent is part of a versioned event instance. Its event-time interval
//! cannot start before or end after the parent (ADR 0003).

mod error;
mod interval;

/// Fail-closed subevent-containment errors.
pub use error::SubeventContainmentError;
/// One half-open event-time interval.
pub use interval::EventInterval;
/// Fraction of recovered containment flags that match known truth.
pub use interval::containment_recovery_rate;
/// Return whether a child interval lies entirely inside a parent interval.
pub use interval::interval_contains;
/// Refuse to attach a subevent that escapes the parent interval.
pub use interval::refuse_escaped_subevent;
