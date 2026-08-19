#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Predicted intervals that contradict observations stay hypothetical.
//!
//! A forecast cannot be promoted to an observed event when
//! [`temporal_core::classify_interval_relation`] returns Allen `before` or
//! `after`, when the pair only `meets` / is `met_by`, or when observed evidence
//! covers only part of the prediction. Evidence whose availability time exceeds
//! the analysis knowledge cutoff is ineligible (ADR 0002, ADR 0016). This crate
//! does not run the path-consistency reasoner.

mod error;
mod interval;

/// Fail-closed prediction-contradiction errors.
pub use error::PredictionContradictionError;
/// Fraction of contradiction flags that match independently supplied labels.
pub use interval::contradiction_agreement_rate;
/// Return whether two closed proper intervals are Allen `before` or `after`.
pub use interval::intervals_contradict;
/// Refuse promotion when evidence is ineligible, disjoint, or only adjacent.
pub use interval::refuse_promotion;
