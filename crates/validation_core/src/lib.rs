#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Recovery metrics intentionally cast small finite sample sizes to `f64`.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
//! Recovery, calibration, graph, and Monte Carlo validation metrics.
//!
//! TEPP scientific acceptance requires realistic synthetic truth recovery:
//! parameter match counts, RMSE, bias, interval coverage with Wilson bounds,
//! temporal-order accuracy, relation precision/recall, and SE-aware Monte Carlo
//! acceptance gates. Metrics are pure `f64` CPU reference implementations.

mod bias;
mod coverage;
mod error;
mod graph_metrics;
mod input;
mod matching;
mod monte_carlo;
mod report;
mod rmse;
mod temporal_order;

/// Standard error of mean signed bias.
pub use bias::bias_standard_error;
/// Mean signed bias.
pub use bias::mean_bias;
/// Empirical interval coverage.
pub use coverage::interval_coverage;
/// Wilson bounds for coverage proportions.
pub use coverage::wilson_coverage_interval;
/// Fail-closed validation errors.
pub use error::ValidationError;
/// Undirected edge identity.
pub use graph_metrics::EdgeIdentity;
/// Edge recovery precision.
pub use graph_metrics::edge_precision;
/// Edge recovery recall.
pub use graph_metrics::edge_recall;
/// Absolute residual vector.
pub use matching::absolute_residuals;
/// Tolerance match counts.
pub use matching::match_count;
/// Monte Carlo replication summary.
pub use monte_carlo::MonteCarloSummary;
/// SE-aware acceptance gate.
pub use monte_carlo::accept_within_standard_errors;
/// Aggregate Monte Carlo replications.
pub use monte_carlo::summarize_replications;
/// Machine-readable validation report.
pub use report::ValidationReport;
/// RMSE standard error.
pub use rmse::rmse_standard_error;
/// Root-mean-square error.
pub use rmse::root_mean_square_error;
/// Pairwise temporal-order accuracy.
pub use temporal_order::temporal_order_accuracy;
