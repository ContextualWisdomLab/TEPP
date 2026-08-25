#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! CPU `f64` ESEM loading recovery and event-time DSEM lag gates.
//!
//! Raw topic proportions are not Euclidean indicators. This crate recovers
//! exploratory cross-loadings from admitted log-ratio or logistic-normal
//! coordinates, refuses reverse event-time lagged paths, and refuses to let a
//! global fit statistic reclassify a formative or network construct as
//! reflective (ADR 0005). It does not replace `psychometric_core` input gates.

mod error;
mod fit;

/// Fail-closed psychometric-fit errors.
pub use error::PsychometricFitError;
/// Higher-order construct class.
pub use fit::FitConstructClass;
/// Coordinate system admitted into an ESEM/DSEM fit.
pub use fit::FitCoordinateKind;
/// Admit only log-ratio or logistic-normal coordinates into a fit.
pub use fit::admit_fit_coordinates;
/// Interpret a classified construct as reflective after a fit.
pub use fit::interpret_fit_as_reflective;
/// Root-mean-square error between known-truth and recovered loadings.
pub use fit::loading_recovery_rmse;
/// Recover a DSEM lagged path only when the predictor precedes the outcome.
pub use fit::recover_dsem_lagged_path;
/// Recover an ESEM loading matrix by OLS of each indicator on every factor.
pub use fit::recover_esem_loadings;
