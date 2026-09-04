#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Recovery metrics intentionally cast small finite sample sizes to `f64`.
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
//! Recovery, calibration, graph, Monte Carlo, and claim-promotion metrics.
//!
//! TEPP scientific acceptance requires realistic synthetic truth recovery:
//! parameter match counts, RMSE, bias, interval coverage with Wilson bounds,
//! temporal-order accuracy, relation precision/recall, and SE-aware Monte Carlo
//! acceptance gates. ADR 0014 claim authorities are promoted only by exact-head
//! evidence; queued, predecessor, skipped, and LLM judgments fail closed.
//! Metrics are pure `f64` CPU reference implementations.

mod bias;
mod claim;
mod coverage;
mod coverage_evidence;
mod error;
mod graph_metrics;
mod input;
mod matching;
mod monte_carlo;
mod numeric;
mod report;
mod rmse;
mod temporal_order;

/// Standard error of mean signed bias.
pub use bias::bias_standard_error;
/// Mean signed bias.
pub use bias::mean_bias;
/// Four ADR 0014 claim authorities.
pub use claim::ClaimAuthority;
/// One evidence item offered for promotion.
pub use claim::ClaimEvidence;
/// Kind of evidence offered for a promotion request.
pub use claim::ClaimEvidenceKind;
/// A claim bound to one exact commit after every required gate passed.
pub use claim::PromotedClaim;
/// Exact-head promotion request.
pub use claim::PromotionRequest;
/// Parse a forty-character hexadecimal Git commit SHA.
pub use claim::parse_commit_head;
/// Promote a claim only when exact-head evidence satisfies ADR 0014.
pub use claim::promote_claim;
/// Promote a scientific claim from computed RMSE, not a hardcoded threshold.
pub use claim::promote_scientific_recovery;
/// Empirical interval coverage.
pub use coverage::interval_coverage;
/// Wilson bounds for coverage proportions.
pub use coverage::wilson_coverage_interval;
/// Versioned Wilson coverage evidence with denominator and critical-value provenance.
pub use coverage_evidence::WilsonCoverageEvidenceV1;
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
