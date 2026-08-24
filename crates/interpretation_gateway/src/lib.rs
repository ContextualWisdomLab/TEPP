#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Evidence-bounded LLM interpretation that cannot become a scientific result.
//!
//! LLM output is an untrusted hypothetical proposal. It must cite at least one
//! evidence span and cannot define an estimator result or observed fact
//! (ADR 0010). Unsupported-claim rates are computed from known truth.

mod error;
mod identity;
mod interpretation;
mod support;

/// Fail-closed interpretation errors.
pub use error::InterpretationError;
/// Opaque interpretation identity.
pub use identity::InterpretationId;
/// Evidence-bounded interpretation proposal.
pub use interpretation::EvidenceBoundInterpretation;
/// Hypothetical-only interpretation status.
pub use interpretation::InterpretationStatus;
/// Refuse to treat an interpretation as an estimator result.
pub use interpretation::refuse_interpretation_as_estimator_result;
/// Refuse to treat an interpretation as an observed fact.
pub use interpretation::refuse_interpretation_as_observed_fact;
/// Known-truth claim support label.
pub use support::ClaimSupport;
/// False-support rate over unsupported known truth.
pub use support::unsupported_claim_rate;
