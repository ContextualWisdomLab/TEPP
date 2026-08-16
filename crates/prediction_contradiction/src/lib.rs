#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Predicted intervals that contradict observations stay hypothetical.
//!
//! A CHRONOS-style forecast cannot be promoted to an observed event when its
//! event-time interval is disjoint from later-observed evidence (ADR 0016).

mod error;
mod interval;

/// Fail-closed prediction-contradiction errors.
pub use error::PredictionContradictionError;
/// One half-open event-time interval.
pub use interval::ClosedEventInterval;
/// Fraction of recovered contradiction flags that match known truth.
pub use interval::contradiction_recovery_rate;
/// Return whether two half-open intervals are disjoint.
pub use interval::intervals_contradict;
/// Refuse to promote a contradicting prediction to observed fact.
pub use interval::refuse_promotion_when_contradict;
