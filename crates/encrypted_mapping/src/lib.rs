#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Purpose-bound encrypted identity mappings without plaintext persistence.
//!
//! Opaque analytical identifiers stay in the clear. Source identity is sealed
//! with AES-256-GCM and opens only under an explicit re-identification purpose
//! (ADR 0009). Persistence and KMS wait for a later migration; this crate does
//! not allocate `0008`.

mod envelope;
mod error;

/// Sealed analytical-id to source-identity envelope.
pub use envelope::EncryptedIdentityMapping;
/// Caller-held mapping key identity and bytes.
pub use envelope::MappingKey;
/// Closed purpose vocabulary for opening a sealed mapping.
pub use envelope::MappingPurpose;
/// Fraction of recovered identities that match known truth.
pub use envelope::identity_recovery_rate;
/// Open a sealed mapping only under re-identification purpose.
pub use envelope::open_identity;
/// Refuse to treat a blanket PII mask as encryption.
pub use envelope::refuse_blanket_mask_as_encryption;
/// Refuse to persist the mapping until a later migration exists.
pub use envelope::refuse_persistence_without_later_migration;
/// Seal a source identity so analytical artifacts cannot read it.
pub use envelope::seal_identity;
/// Fail-closed encrypted-mapping errors.
pub use error::EncryptedMappingError;
