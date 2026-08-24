#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Selected-K RMSE casts small finite topic counts to `f64`.
#![allow(clippy::cast_precision_loss)]
//! Statistical and Pareto candidate-`K` gates for TRSL-TM model selection.
//!
//! Model selection uses held-out log-likelihood and complexity before any
//! blinded LLM review. An LLM vote may recommend among statistically
//! admissible candidates but never defines the numerical optimum (ADR 0012).

mod candidate;
mod error;
mod gate;

/// One candidate `K` with statistical or LLM-only support.
pub use candidate::ModelCandidate;
/// Fail-closed model-selection errors.
pub use error::ModelSelectionError;
/// Select the admissible candidate `K` from a Pareto-filtered statistical front.
pub use gate::select_candidate_k;
/// RMSE of selected `K` replications against known truth.
pub use gate::selected_k_root_mean_square_error;
