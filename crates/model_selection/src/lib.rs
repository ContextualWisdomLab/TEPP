#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Selected-K RMSE casts small finite topic counts to `f64`.
#![allow(clippy::cast_precision_loss)]
//! Statistical and Pareto candidate-`K` gates for TRSL-TM model selection.
//!
//! Model selection fits each candidate `K` with the CPU `f64` reference
//! estimator and scores the actual mixture likelihood and parameter count
//! before any blinded LLM review. An LLM vote may recommend among
//! statistically admissible candidates but never defines the numerical
//! optimum (ADR 0012).

mod candidate;
mod error;
mod fitted;
mod gate;
mod rolling_origin_predictive;

/// One candidate `K` with statistical or LLM-only support.
pub use candidate::ModelCandidate;
/// Fail-closed model-selection errors.
pub use error::ModelSelectionError;
/// Seeds, iteration budget, and candidate topic counts for fitted selection.
pub use fitted::FittedCandidateKConfig;
/// Fit each candidate `K` and select from the actual statistical diagnostics.
pub use fitted::select_fitted_candidate_k;
/// Build a statistical candidate from one owner-issued reference fit.
pub use fitted::statistical_candidate_from_fit;
/// Monte Carlo recovery summary that retains failed-replication denominators.
pub use gate::SelectedKRecoverySummary;
/// Select the admissible candidate `K` from a Pareto-filtered statistical front.
pub use gate::select_candidate_k;
/// Summarize selected-`K` recovery with bias, RMSE, failures, and Monte Carlo error.
pub use gate::selected_k_recovery_summary;
/// RMSE of selected `K` replications against known truth.
pub use gate::selected_k_root_mean_square_error;
/// Score one admitted rolling-origin evaluation partition under a fixed training fit.
pub use rolling_origin_predictive::rolling_origin_prevalence_mean_predictive_log_likelihood;
/// Select candidate `K` from one admitted rolling-origin predictive partition.
pub use rolling_origin_predictive::select_rolling_origin_predictive_candidate_k;
