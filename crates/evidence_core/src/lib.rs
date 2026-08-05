#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Immutable source-evidence identifiers, records, and exact spans.
//!
//! The crate owns fail-closed evidence-domain invariants. It starts with an
//! RFC 9562 `UUIDv7` identifier so later source artifacts, spans, events, and
//! audit records cannot accept arbitrary UUID versions by accident.

mod error;
mod identifier;

/// Fail-closed evidence-domain validation errors.
pub use error::EvidenceError;
/// A validated RFC 9562 `UUIDv7` evidence identifier.
pub use identifier::EvidenceId;
