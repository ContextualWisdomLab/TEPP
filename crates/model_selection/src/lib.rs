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

/// One candidate `K` with statistical or LLM-only support.
pub use candidate::ModelCandidate;
/// Fail-closed model-selection errors.
pub use error::ModelSelectionError;
/// Seeds, iteration budget, and candidate topic counts for fitted selection.
pub use fitted::FittedCandidateKConfig;
/// Compact reason-bearing outcome for one fitted candidate.
pub use fitted::FittedCandidateOutcome;
/// Winning model and bounded evidence for the fitted selection decision.
pub use fitted::FittedCandidateSelection;
/// Typed selection failure retaining completed candidate diagnostics.
pub use fitted::FittedCandidateSelectionFailure;
/// Fit each candidate `K` and select from the actual statistical diagnostics.
pub use fitted::select_fitted_candidate_k;
/// Fit each candidate `K` and retain the selected converged CPU `f64` model.
pub use fitted::select_fitted_candidate_model;
/// Build a statistical candidate from one actual fitted model.
pub use fitted::statistical_candidate_from_fit;
/// Select the admissible candidate `K` from a Pareto-filtered statistical front.
pub use gate::select_candidate_k;
/// RMSE of selected `K` replications against known truth.
pub use gate::selected_k_root_mean_square_error;
