#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Event-time lags that refuse equal system-time spacing.
//!
//! DSEM and other longitudinal estimators must space observations on event
//! time. Equal system-time sampling cannot stand in for irregular event lags
//! (ADR 0002/0005).

mod error;
mod observation;

/// Fail-closed irregular-time errors.
pub use error::IrregularTimeError;
/// Dual-clock observation.
pub use observation::ClockedObservation;
/// Consecutive event-time lags.
pub use observation::event_lag_seconds;
/// RMSE of recovered lags against known truth.
pub use observation::lag_root_mean_square_error;
/// Refuse to treat equal system spacing as event spacing.
pub use observation::refuse_equal_system_spacing_as_event_spacing;
