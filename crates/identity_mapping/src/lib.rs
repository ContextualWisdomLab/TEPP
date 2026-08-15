#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(clippy::cast_precision_loss)]
//! Opaque analytical identifiers with separately authorized re-identification.
//!
//! Ordinary compute uses opaque analytical identifiers. Exporting the protected
//! source-identity mapping requires an explicit re-identification purpose.
//! Analytical purpose and blanket PII masking cannot unlock the mapping
//! (ADR 0009).

mod error;
mod mapping;

/// Fail-closed identity-mapping errors.
pub use error::IdentityMappingError;
/// One opaque analytical identifier paired with a source identity.
pub use mapping::IdentityMapRecord;
/// Closed purpose vocabulary for mapping export.
pub use mapping::MappingPurpose;
/// Export source identities only under re-identification purpose.
pub use mapping::export_reidentification;
/// Fraction of recovered mapping pairs that match known truth.
pub use mapping::mapping_recovery_rate;
/// Refuse to treat blanket PII masking as re-identification authorization.
pub use mapping::refuse_blanket_mask_as_reidentification;
