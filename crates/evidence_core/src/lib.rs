#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Immutable source-evidence identifiers, records, and exact spans.
//!
//! The crate owns fail-closed evidence-domain invariants. It provides RFC 9562
//! `UUIDv7` identifiers, canonical `SHA-256` digests, immutable byte and text
//! records, source spans whose byte, Unicode-scalar, page, and layout
//! coordinates are validated before entering later temporal or psychometric
//! layers, versioned source-snapshot evidence bindings, Evidence-owned source
//! observation/availability records, and strict versioned JSON wire contracts
//! that validate external records without promoting caller-supplied identity
//! into owner-issued Evidence state. Embedded `data:image` units keep their
//! original offsets and are not lexical inference text.

mod artifact;
mod digest;
mod document;
mod error;
mod identifier;
mod image_unit;
mod source_observation;
mod source_snapshot_receipt;
mod span;
mod wire;

/// An immutable source artifact with an Evidence-issued identity and verified content digest.
pub use artifact::SourceArtifact;
/// Evidence-issued nominal identity handle for one trusted source artifact.
pub use artifact::SourceArtifactId;
/// Canonical validated source-artifact wire input that is not owner-authenticated Evidence state.
pub use artifact::ValidatedSourceArtifactWire;
/// A canonical `SHA-256` content digest.
pub use digest::ContentDigest;
/// An immutable UTF-8 document linked to its source artifact.
pub use document::DocumentRecord;
/// Canonical validated document wire input that is not owner-authenticated Evidence state.
pub use document::ValidatedDocumentRecordWire;
/// Fail-closed evidence-domain validation errors.
pub use error::EvidenceError;
/// A validated RFC 9562 `UUIDv7` evidence identifier.
pub use identifier::EvidenceId;
/// One embedded image located in a document body.
pub use image_unit::EmbeddedImageUnit;
/// Locate `data:image` base64 units with exact source spans.
pub use image_unit::embedded_image_units;
/// Refuse treating an embedded image URI as lexical inference text.
pub use image_unit::refuse_base64_image_as_lexical_text;
/// Evidence-owned availability of a previously observed source artifact.
pub use source_observation::SourceAvailability;
/// Evidence-owned observation of an immutable source artifact entering TEPP.
pub use source_observation::SourceObservation;
/// Maximum canonical JSON size accepted for one source-snapshot receipt.
pub use source_snapshot_receipt::SOURCE_SNAPSHOT_RECEIPT_BYTE_LIMIT;
/// Versioned wire schema for immutable source-snapshot receipts.
pub use source_snapshot_receipt::SOURCE_SNAPSHOT_RECEIPT_SCHEMA_VERSION;
/// Evidence-owned immutable source-snapshot binding receipt.
pub use source_snapshot_receipt::SourceSnapshotReceiptV1;
/// Canonical validated receipt wire that is not owner-authenticated Evidence state.
pub use source_snapshot_receipt::ValidatedSourceSnapshotReceiptWireV1;
/// A validated page-relative location for source evidence.
pub use span::PageLocation;
/// An exact byte, Unicode-scalar, and optional page/layout span.
pub use span::SourceSpan;
/// The only JSON wire-schema version accepted by this crate.
pub use wire::WIRE_SCHEMA_VERSION;
