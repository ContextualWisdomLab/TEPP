#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Posterior-aware psychometric input gates for ESEM/DSEM.
//!
//! Raw topic proportions are not Euclidean indicators. This crate classifies
//! constructs, admits only log-ratio or logistic-normal coordinates, aggregates
//! plausible-value loadings on a CPU `f64` path, and refuses causal language
//! from temporal precedence, document linkage, event tracking, or prediction.

mod causality;
mod construct;
mod error;
mod indicator;
mod loading;
mod plausible;

/// A heuristic that is not causal identification.
pub use causality::CausalHeuristic;
/// Refuse a causal-effect claim from a non-identifying heuristic.
pub use causality::claim_causal_effect;
/// Higher-order construct class.
pub use construct::ConstructClass;
/// Permit latent-mean comparison only with invariance evidence.
pub use construct::compare_latent_means;
/// Refuse fit-driven reinterpretation as reflective.
pub use construct::interpret_as_reflective;
/// Fail-closed psychometric errors.
pub use error::PsychometricError;
/// Indicator coordinate kind.
pub use indicator::IndicatorKind;
/// Pearson correlation on valid coordinates.
pub use indicator::pearson_correlation;
/// Refuse raw topic proportions as psychometric indicators.
pub use indicator::require_valid_indicator;
/// Ordinary least-squares slope.
pub use loading::ordinary_least_squares_slope;
/// Recover one reflective loading.
pub use loading::recover_reflective_loading;
/// Arithmetic mean of plausible-value draws.
pub use plausible::plausible_value_mean;
/// Average OLS loadings across posterior indicator draws.
pub use plausible::recover_loading_from_plausible_values;
