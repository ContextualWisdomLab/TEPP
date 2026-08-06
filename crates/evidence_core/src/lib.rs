#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Immutable source-evidence identifiers, records, and exact spans.
//!
//! The crate owns fail-closed evidence-domain invariants. It provides RFC 9562
//! `UUIDv7` identifiers, canonical `SHA-256` digests, immutable byte and text
//! records, source spans whose byte, Unicode-scalar, page, and layout
//! coordinates are validated before entering later temporal or psychometric
//! layers, and strict versioned JSON wire contracts that reconstruct records
//! only through the same domain validation boundary.

mod artifact;
mod digest;
mod document;
mod error;
mod identifier;
mod span;
mod wire;

/// An immutable source artifact with a verified content digest.
pub use artifact::SourceArtifact;
/// A canonical `SHA-256` content digest.
pub use digest::ContentDigest;
/// An immutable UTF-8 document linked to its source artifact.
pub use document::DocumentRecord;
/// Fail-closed evidence-domain validation errors.
pub use error::EvidenceError;
/// A validated RFC 9562 `UUIDv7` evidence identifier.
pub use identifier::EvidenceId;
/// A validated page-relative location for source evidence.
pub use span::PageLocation;
/// An exact byte, Unicode-scalar, and optional page/layout span.
pub use span::SourceSpan;
/// The only JSON wire-schema version accepted by this crate.
pub use wire::WIRE_SCHEMA_VERSION;
