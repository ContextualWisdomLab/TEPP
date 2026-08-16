#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Predicted intervals that contradict observations stay hypothetical.
//!
//! A forecast cannot be promoted to an observed event when
//! [`temporal_core::classify_interval_relation`] returns Allen `before` or
//! `after`, or when the pair only `meets` / is `met_by`. Partial overlap is
//! not a contradiction, but it also does not cover unmatched predicted
//! mass. [`require_observed_coverage`] is the promotion-authority gate.
//! Evidence whose availability time exceeds the analysis knowledge cutoff
//! is ineligible (ADR 0002, ADR 0016). This crate does not run the
//! path-consistency reasoner.

mod error;
mod interval;

/// Fail-closed prediction-contradiction errors.
pub use error::PredictionContradictionError;
/// How later-observed evidence relates to a predicted event-time interval.
pub use interval::PromotionSupport;
/// Classify predicted-versus-observed support without applying cutoff policy.
pub use interval::classify_promotion_support;
/// Fraction of contradiction flags that match independently supplied labels.
pub use interval::contradiction_agreement_rate;
/// Return whether two closed proper intervals are Allen `before` or `after`.
pub use interval::intervals_contradict;
/// Refuse promotion when evidence is ineligible, disjoint, or only adjacent.
pub use interval::refuse_promotion;
/// Refuse promotion unless later-observed evidence covers the prediction.
pub use interval::require_observed_coverage;
