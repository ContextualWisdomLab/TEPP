#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Untrusted payloads fail closed without identity, provenance, size, and depth.
//!
//! Documents, serialized records, model checkpoints, and LLM outputs stay
//! untrusted until the owning boundary validates those four gates
//! (AGENTS.md; ADR 0008/0013).

mod bound;
mod error;

/// Fraction of recovered accept/reject flags that match known truth.
pub use bound::identity_recovery_rate;
/// Refuse an untrusted payload that fails identity, provenance, size, or depth.
pub use bound::refuse_untrusted_payload;
/// Positive byte and nesting-depth limits for one untrusted payload.
pub use bound::PayloadBound;
/// Closed vocabulary of untrusted inbound payload kinds.
pub use bound::PayloadKind;
/// Fail-closed payload-bound errors.
pub use error::PayloadBoundError;
